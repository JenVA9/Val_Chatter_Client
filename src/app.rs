use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::api::client::ApiClient;
use crate::models::{Message, Node, Thread};
use crate::state::{AuthState, ChatState, NavState};
use crate::utils::image_cache::{self, ImageCache};
use crate::ws;

#[derive(PartialEq)]
pub enum Screen {
    Login,
    Main,
}

pub enum AppEvent {
    LoginSuccess { token: String, user_id: i64, username: String },
    NodesLoaded(Vec<Node>),
    ThreadResolved(Thread),
    MessagesLoaded { thread_id: i64, messages: Vec<Message> },
    PinnedLoaded { thread_id: i64, messages: Vec<Message> },
    MessageReceived(Message),
    UploadComplete(String),
    ImageLoaded { url: String, image: Option<egui::ColorImage> },
    Error(String),
}

pub struct App {
    pub screen: Screen,
    egui_ctx: egui::Context,

    pub login_username: String,
    pub login_password: String,
    pub login_error: Option<String>,
    pub login_register: bool,
    pub login_server_url: String,

    pub auth: AuthState,
    pub nav: NavState,
    nav_snapshot: NavState,
    pub chat: ChatState,
    pub nodes: Vec<Node>,

    pub api: ApiClient,
    pub server_url: String,
    pub ws_url: String,

    rt: tokio::runtime::Runtime,
    event_tx: mpsc::SyncSender<AppEvent>,
    event_rx: mpsc::Receiver<AppEvent>,
    ws_abort: Option<tokio::task::AbortHandle>,

    pub image_cache: ImageCache,
    pub viewing_image: Option<String>,
    pub pending_upload: bool,

    toast: Option<(String, Instant)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let server_url = cc.storage
            .and_then(|s| eframe::get_value::<String>(s, "server_url"))
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let ws_url = derive_ws_url(&server_url);
        let api = ApiClient::new(&server_url);
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let (event_tx, event_rx) = mpsc::sync_channel::<AppEvent>(256);

        let mut app = Self {
            screen: Screen::Login,
            egui_ctx: cc.egui_ctx.clone(),
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            login_register: false,
            login_server_url: server_url.clone(),
            auth: AuthState::default(),
            nav: NavState::default(),
            nav_snapshot: NavState::default(),
            chat: ChatState::default(),
            nodes: Vec::new(),
            api,
            server_url,
            ws_url,
            rt,
            event_tx,
            event_rx,
            ws_abort: None,
            image_cache: ImageCache::new(),
            viewing_image: None,
            pending_upload: false,
            toast: None,
        };

        // Try restoring saved session from OS keychain
        if let Ok(token) = try_load_token() {
            if let Ok(uid) = try_load_user_id() {
                if let Ok(uname) = try_load_username() {
                    app.auth = AuthState { token: token.clone(), user_id: uid, username: uname };
                    app.screen = Screen::Main;
                    app.spawn_load_nodes();
                }
            }
        }

        app
    }

    // ──────────────────────────── event handling ────────────────────────────

    fn drain_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.handle_event(event, ctx);
        }
    }

    fn handle_event(&mut self, event: AppEvent, ctx: &egui::Context) {
        match event {
            AppEvent::LoginSuccess { token, user_id, username } => {
                let _ = save_token(&token);
                let _ = save_user_id(user_id);
                let _ = save_username(&username);
                self.auth = AuthState { token, user_id, username };
                self.screen = Screen::Main;
                self.spawn_load_nodes();
            }
            AppEvent::NodesLoaded(nodes) => {
                self.nodes = nodes;
            }
            AppEvent::ThreadResolved(thread) => {
                let tid = thread.id;
                self.chat.thread_id = Some(tid);
                self.chat.messages.clear();
                self.chat.pinned.clear();
                self.chat.scroll_to_bottom = true;
                self.spawn_load_messages(tid, None);
                self.spawn_load_pinned(tid);
                self.start_ws(tid);
            }
            AppEvent::MessagesLoaded { thread_id, messages } => {
                if self.chat.thread_id == Some(thread_id) {
                    self.chat.messages = messages;
                    self.chat.scroll_to_bottom = true;
                }
            }
            AppEvent::PinnedLoaded { thread_id, messages } => {
                if self.chat.thread_id == Some(thread_id) {
                    self.chat.pinned = messages;
                }
            }
            AppEvent::MessageReceived(msg) => {
                if self.chat.thread_id == Some(msg.thread_id) {
                    // Avoid duplicates (msg might already be added by HTTP send response)
                    if !self.chat.messages.iter().any(|m| m.id == msg.id) {
                        self.chat.messages.push(msg);
                        self.chat.scroll_to_bottom = true;
                    }
                }
            }
            AppEvent::UploadComplete(url) => {
                self.pending_upload = false;
                self.chat.pending_image_url = Some(url);
            }
            AppEvent::ImageLoaded { url, image } => {
                if let Some(color_image) = image {
                    let tex = ctx.load_texture(&url, color_image, egui::TextureOptions::default());
                    self.image_cache.insert(url, tex);
                } else {
                    self.image_cache.mark_failed(&url);
                }
            }
            AppEvent::Error(msg) => {
                self.toast = Some((msg, Instant::now()));
                self.login_error = Some(self.toast.as_ref().unwrap().0.clone());
                self.pending_upload = false;
            }
        }
    }

    // ──────────────────────────── async spawns ──────────────────────────────

    pub fn spawn_login(&mut self, username: String, password: String, register: bool) {
        let client = self.api.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let result = if register {
                crate::api::auth::register(&client, &username, &password).await
            } else {
                crate::api::auth::login(&client, &username, &password).await
            };
            let ev = match result {
                Ok(r) => AppEvent::LoginSuccess { token: r.token, user_id: r.user_id, username: r.username },
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_load_nodes(&mut self) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let ev = match crate::api::nodes::get_all(&client, &token).await {
                Ok(nodes) => AppEvent::NodesLoaded(nodes),
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_resolve_thread(&mut self, node_ids: Vec<i64>) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let ev = match crate::api::threads::resolve(&client, &token, node_ids).await {
                Ok(thread) => AppEvent::ThreadResolved(thread),
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_load_messages(&mut self, thread_id: i64, before: Option<i64>) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let ev = match crate::api::messages::get(&client, &token, thread_id, before).await {
                Ok(messages) => AppEvent::MessagesLoaded { thread_id, messages },
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_load_pinned(&mut self, thread_id: i64) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let ev = match crate::api::pins::get_pinned(&client, &token, thread_id).await {
                Ok(messages) => AppEvent::PinnedLoaded { thread_id, messages },
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_send_message(&mut self, content: String, image_url: Option<String>) {
        let Some(thread_id) = self.chat.thread_id else { return };
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let ev = match crate::api::messages::send(&client, &token, thread_id, Some(content), image_url).await {
                Ok(msg) => AppEvent::MessageReceived(msg),
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_delete_message(&mut self, message_id: i64) {
        self.chat.messages.retain(|m| m.id != message_id);
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        self.rt.spawn(async move {
            if let Err(e) = crate::api::messages::delete(&client, &token, message_id).await {
                let _ = tx.send(AppEvent::Error(e.to_string()));
            }
        });
    }

    pub fn spawn_pin_message(&mut self, message_id: i64, pin: bool) {
        let Some(thread_id) = self.chat.thread_id else { return };
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let result = if pin {
                crate::api::pins::pin(&client, &token, message_id).await
            } else {
                crate::api::pins::unpin(&client, &token, message_id).await
            };
            let ev = match result {
                Ok(_) => AppEvent::PinnedLoaded {
                    thread_id,
                    messages: crate::api::pins::get_pinned(&client, &token, thread_id)
                        .await
                        .unwrap_or_default(),
                },
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_upload_image(&mut self, path: std::path::PathBuf) {
        self.pending_upload = true;
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let ev = match tokio::fs::read(&path).await {
                Ok(bytes) => {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("image.png")
                        .to_string();
                    let mime = if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                        "image/jpeg"
                    } else {
                        "image/png"
                    };
                    match crate::api::uploads::upload_image(&client, &token, bytes, filename, mime).await {
                        Ok(url) => AppEvent::UploadComplete(url),
                        Err(e) => AppEvent::Error(e.to_string()),
                    }
                }
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    // ──────────────────────────── WebSocket ─────────────────────────────────

    fn start_ws(&mut self, thread_id: i64) {
        if let Some(h) = self.ws_abort.take() { h.abort(); }

        let (ws_tx, ws_rx) = mpsc::sync_channel::<ws::WsEvent>(256);
        let event_tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();

        std::thread::spawn(move || {
            for ev in ws_rx {
                let app_ev = match ev {
                    ws::WsEvent::NewMessage(m) => AppEvent::MessageReceived(m),
                    ws::WsEvent::Error(e) => AppEvent::Error(e),
                    _ => continue,
                };
                if event_tx.send(app_ev).is_err() { break; }
                ctx.request_repaint();
            }
        });

        let ws_url = self.ws_url.clone();
        let token = self.auth.token.clone();
        let task = self.rt.spawn(async move {
            ws::socket::connect_and_listen(ws_url, token, thread_id, ws_tx).await;
        });
        self.ws_abort = Some(task.abort_handle());
    }

    // ──────────────────────────── nav change ────────────────────────────────

    fn on_nav_changed(&mut self) {
        let ids = self.nav.selected_node_ids();
        if ids.is_empty() {
            if let Some(h) = self.ws_abort.take() { h.abort(); }
            self.chat = ChatState::default();
        } else {
            self.chat.thread_id = None;
            self.chat.messages.clear();
            self.chat.pinned.clear();
            self.spawn_resolve_thread(ids);
        }
    }

    // ──────────────────────────── image prefetch ────────────────────────────

    fn prefetch_visible_images(&mut self) {
        let urls: Vec<String> = self.chat.messages.iter()
            .filter_map(|m| m.image_url.as_ref())
            .filter(|u| !self.image_cache.contains(u) && !self.image_cache.is_pending(u))
            .cloned()
            .collect();

        for url in urls {
            self.image_cache.mark_pending(url.clone());
            let http = self.api.http.clone();
            let base = self.server_url.clone();
            let tx = self.event_tx.clone();
            let ctx = self.egui_ctx.clone();
            let key = url.clone();
            self.rt.spawn(async move {
                let full = if url.starts_with("http") { url.clone() } else { format!("{}{}", base, url) };
                let image = image_cache::fetch_image(&http, &full).await.ok();
                let _ = tx.send(AppEvent::ImageLoaded { url: key, image });
                ctx.request_repaint();
            });
        }
    }

    // ──────────────────────────── toast ─────────────────────────────────────

    fn show_toast(&self, ctx: &egui::Context) {
        if let Some((msg, when)) = &self.toast {
            if when.elapsed() < Duration::from_secs(4) {
                egui::Window::new("##toast")
                    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
                    .collapsible(false)
                    .resizable(false)
                    .title_bar(false)
                    .show(ctx, |ui| {
                        ui.colored_label(egui::Color32::RED, msg);
                    });
                ctx.request_repaint();
            }
        }
    }

    pub fn logout(&mut self) {
        let _ = clear_keyring();
        self.auth = AuthState::default();
        self.chat = ChatState::default();
        self.nav = NavState::default();
        self.nodes.clear();
        if let Some(h) = self.ws_abort.take() { h.abort(); }
        self.screen = Screen::Login;
    }

    pub fn update_server_url(&mut self, url: String) {
        self.server_url = url.clone();
        self.ws_url = derive_ws_url(&url);
        self.api = ApiClient::new(&url);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events(ctx);

        if self.nav != self.nav_snapshot {
            self.nav_snapshot = self.nav.clone();
            self.on_nav_changed();
        }

        self.prefetch_visible_images();
        self.show_toast(ctx);

        match self.screen {
            Screen::Login => crate::ui::login::show(ctx, self),
            Screen::Main => crate::ui::main_layout::show(ctx, self),
        }

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "server_url", &self.server_url);
    }
}

// ──────────────────────────────── helpers ───────────────────────────────────

fn derive_ws_url(http_url: &str) -> String {
    http_url
        .replace("https://", "wss://")
        .replace("http://", "ws://")
}

fn keyring_entry(key: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new("val-tactics", key)
}

fn try_load_token() -> Result<String, keyring::Error> {
    keyring_entry("token")?.get_password()
}

fn try_load_user_id() -> Result<i64, Box<dyn std::error::Error>> {
    let s = keyring_entry("user_id")?.get_password()?;
    Ok(s.parse()?)
}

fn try_load_username() -> Result<String, keyring::Error> {
    keyring_entry("username")?.get_password()
}

fn save_token(token: &str) -> Result<(), keyring::Error> {
    keyring_entry("token")?.set_password(token)
}

fn save_user_id(id: i64) -> Result<(), keyring::Error> {
    keyring_entry("user_id")?.set_password(&id.to_string())
}

fn save_username(name: &str) -> Result<(), keyring::Error> {
    keyring_entry("username")?.set_password(name)
}

fn clear_keyring() -> Result<(), keyring::Error> {
    let _ = keyring_entry("token")?.delete_password();
    let _ = keyring_entry("user_id")?.delete_password();
    let _ = keyring_entry("username")?.delete_password();
    Ok(())
}
