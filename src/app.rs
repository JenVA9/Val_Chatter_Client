use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::api::admin::{AdminUser, StorageInfo};
use crate::api::client::ApiClient;
use crate::models::{Message, Node, Thread, WbObject, WhiteboardState};
use crate::state::{AuthState, ChatState, NavState};
use crate::utils::image_cache::{self, ImageCache};
use crate::ws;

// ── Enums ─────────────────────────────────────────────────────────────────────

#[derive(PartialEq)]
pub enum Screen { Login, Main }

#[derive(Debug, Clone, PartialEq)]
pub enum Theme { Dark, Light, Nebula, OledBlack }
impl Default for Theme { fn default() -> Self { Self::Dark } }

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AdminTab { #[default] Users, Nodes, Config, Storage, IpBans }

#[derive(Debug, Clone, PartialEq)]
pub enum ThreadMode { Chat, Whiteboard }

// ── Supporting structs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ServerProfile {
    pub url: String,
    pub username: String,
    pub user_id: i64,
    pub token: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone)]
pub struct PinDialog {
    pub message_id: i64,
    pub duration_str: String,
    pub is_permanent: bool,
}

// ── AppEvent ──────────────────────────────────────────────────────────────────

pub enum AppEvent {
    LoginSuccess { token: String, user_id: i64, username: String, is_admin: bool, is_guest: bool },
    NodesLoaded(Vec<Node>),
    ActiveNodesLoaded(Vec<i64>),
    ThreadResolved(Thread),
    MessagesLoaded { thread_id: i64, messages: Vec<Message> },
    PinnedLoaded { thread_id: i64, messages: Vec<Message> },
    MessageReceived(Message),
    UploadComplete { url: String, whiteboard_world_pos: Option<[f32; 2]> },
    ImageLoaded { url: String, image: Option<egui::ColorImage> },
    WhiteboardLoaded { thread_id: i64, objects: Vec<WbObject> },
    AdminUsersLoaded(Vec<AdminUser>),
    AdminConfigLoaded(Vec<(String, String)>),
    AdminNodesLoaded(Vec<Node>),
    StorageInfoLoaded(StorageInfo),
    IpBansLoaded(Vec<String>),
    InputLocked,
    InputUnlocked,
    Error(String),
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    egui_ctx: egui::Context,

    // Login state
    pub login_username: String,
    pub login_password: String,
    pub login_error: Option<String>,
    pub login_register: bool,
    pub login_server_url: String,

    // Auth
    pub auth: AuthState,
    pub is_admin: bool,
    pub is_guest: bool,

    // Navigation
    pub nav: NavState,
    nav_snapshot: NavState,
    pub nodes: Vec<Node>,
    pub active_nodes: HashSet<i64>,

    // Chat
    pub chat: ChatState,
    pub scroll_to_message: Option<i64>,
    pub pin_dialog: Option<PinDialog>,

    // Thread modes (chat vs whiteboard per thread)
    pub thread_mode: HashMap<i64, ThreadMode>,

    // Whiteboard
    pub whiteboard: WhiteboardState,

    // API / WS
    pub api: ApiClient,
    pub server_url: String,
    pub ws_url: String,
    pub servers: Vec<ServerProfile>,
    pub new_server_url_input: String,

    rt: tokio::runtime::Runtime,
    event_tx: mpsc::SyncSender<AppEvent>,
    event_rx: mpsc::Receiver<AppEvent>,
    ws_abort: Option<tokio::task::AbortHandle>,

    // Images
    pub image_cache: ImageCache,
    pub viewing_image: Option<String>,
    pub pending_upload: bool,

    // Connectivity
    pub input_locked: bool,

    // UI state
    pub settings_open: bool,
    pub admin_panel_open: bool,
    pub admin_tab: AdminTab,
    pub theme: Theme,
    pub font_size: f32,

    // Admin panel data
    pub admin_users: Vec<AdminUser>,
    pub admin_config: Vec<(String, String)>,
    pub admin_nodes: Vec<Node>,
    pub admin_storage: StorageInfo,
    pub admin_ip_bans: Vec<String>,
    pub admin_new_node_name: String,
    pub admin_new_node_type: String,
    pub admin_new_ip: String,
    pub admin_storage_limit_input: String,

    toast: Option<(String, Instant)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let server_url = cc.storage
            .and_then(|s| eframe::get_value::<String>(s, "server_url"))
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let servers: Vec<ServerProfile> = cc.storage
            .and_then(|s| eframe::get_value::<Vec<ServerProfile>>(s, "servers"))
            .unwrap_or_default();

        let theme: Theme = cc.storage
            .and_then(|s| eframe::get_value::<String>(s, "theme"))
            .and_then(|t| match t.as_str() {
                "light" => Some(Theme::Light),
                "nebula" => Some(Theme::Nebula),
                "oled" => Some(Theme::OledBlack),
                _ => None,
            })
            .unwrap_or_default();

        let font_size: f32 = cc.storage
            .and_then(|s| eframe::get_value::<f32>(s, "font_size"))
            .unwrap_or(14.0);

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
            is_admin: false,
            is_guest: false,
            nav: NavState::default(),
            nav_snapshot: NavState::default(),
            nodes: Vec::new(),
            active_nodes: HashSet::new(),
            chat: ChatState::default(),
            scroll_to_message: None,
            pin_dialog: None,
            thread_mode: HashMap::new(),
            whiteboard: WhiteboardState::new(),
            api,
            server_url,
            ws_url,
            servers,
            new_server_url_input: String::new(),
            rt,
            event_tx,
            event_rx,
            ws_abort: None,
            image_cache: ImageCache::new(),
            viewing_image: None,
            pending_upload: false,
            input_locked: false,
            settings_open: false,
            admin_panel_open: false,
            admin_tab: AdminTab::default(),
            theme,
            font_size,
            admin_users: Vec::new(),
            admin_config: Vec::new(),
            admin_nodes: Vec::new(),
            admin_storage: StorageInfo::default(),
            admin_ip_bans: Vec::new(),
            admin_new_node_name: String::new(),
            admin_new_node_type: "map".to_string(),
            admin_new_ip: String::new(),
            admin_storage_limit_input: String::new(),
            toast: None,
        };

        app.apply_theme(&cc.egui_ctx);

        if let Ok(token) = try_load_token() {
            if let Ok(uid) = try_load_user_id() {
                if let Ok(uname) = try_load_username() {
                    app.auth = AuthState { token: token.clone(), user_id: uid, username: uname };
                    app.screen = Screen::Main;
                    app.spawn_load_nodes();
                    app.spawn_load_active_nodes();
                }
            }
        }

        app
    }

    // ── Theme ─────────────────────────────────────────────────────────────────

    pub fn apply_theme(&self, ctx: &egui::Context) {
        use egui::{Color32, Visuals, TextStyle, FontId};

        let vis = match self.theme {
            Theme::Dark => Visuals::dark(),
            Theme::Light => Visuals::light(),
            Theme::Nebula => {
                let mut v = Visuals::dark();
                v.panel_fill = Color32::from_rgb(14, 6, 25);
                v.window_fill = Color32::from_rgb(20, 10, 35);
                v.faint_bg_color = Color32::from_rgb(30, 15, 50);
                v.override_text_color = Some(Color32::from_rgb(220, 200, 255));
                v
            }
            Theme::OledBlack => {
                let mut v = Visuals::dark();
                v.panel_fill = Color32::BLACK;
                v.window_fill = Color32::from_gray(5);
                v.faint_bg_color = Color32::from_gray(8);
                v
            }
        };

        ctx.set_visuals(vis);

        let fs = self.font_size;
        use std::collections::BTreeMap;
        let mut style = (*ctx.style()).clone();
        style.text_styles = BTreeMap::from([
            (TextStyle::Small,   FontId::proportional(fs * 0.8)),
            (TextStyle::Body,    FontId::proportional(fs)),
            (TextStyle::Button,  FontId::proportional(fs)),
            (TextStyle::Heading, FontId::proportional(fs * 1.4)),
            (TextStyle::Monospace, FontId::monospace(fs)),
        ]);
        ctx.set_style(style);
    }

    // ── Event handling ────────────────────────────────────────────────────────

    fn drain_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.event_rx.try_recv() {
            self.handle_event(event, ctx);
        }
    }

    fn handle_event(&mut self, event: AppEvent, ctx: &egui::Context) {
        match event {
            AppEvent::LoginSuccess { token, user_id, username, is_admin, is_guest } => {
                let _ = save_token(&token);
                let _ = save_user_id(user_id);
                let _ = save_username(&username);
                self.auth = AuthState { token, user_id, username };
                self.is_admin = is_admin;
                self.is_guest = is_guest;
                self.screen = Screen::Main;
                self.spawn_load_nodes();
                self.spawn_load_active_nodes();
            }
            AppEvent::NodesLoaded(nodes) => {
                self.nodes = nodes;
            }
            AppEvent::ActiveNodesLoaded(ids) => {
                self.active_nodes = ids.into_iter().collect();
            }
            AppEvent::ThreadResolved(thread) => {
                let tid = thread.id;
                let mode = if thread.mode == "whiteboard" { ThreadMode::Whiteboard } else { ThreadMode::Chat };
                self.thread_mode.insert(tid, mode.clone());
                self.chat.thread_id = Some(tid);
                self.chat.messages.clear();
                self.chat.pinned.clear();
                self.chat.scroll_to_bottom = true;
                if mode == ThreadMode::Chat {
                    self.spawn_load_messages(tid, None);
                    self.spawn_load_pinned(tid);
                } else {
                    self.spawn_load_whiteboard(tid);
                }
                self.start_ws(tid);
            }
            AppEvent::MessagesLoaded { thread_id, messages } => {
                if self.chat.thread_id == Some(thread_id) {
                    self.chat.messages = messages;
                    self.chat.scroll_to_bottom = true;
                    self.spawn_load_active_nodes();
                }
            }
            AppEvent::PinnedLoaded { thread_id, messages } => {
                if self.chat.thread_id == Some(thread_id) {
                    self.chat.pinned = messages;
                }
            }
            AppEvent::MessageReceived(msg) => {
                if self.chat.thread_id == Some(msg.thread_id) {
                    if !self.chat.messages.iter().any(|m| m.id == msg.id) {
                        self.chat.messages.push(msg);
                        self.chat.scroll_to_bottom = true;
                    }
                    self.spawn_load_active_nodes();
                }
            }
            AppEvent::UploadComplete { url, whiteboard_world_pos } => {
                self.pending_upload = false;
                if let Some(wpos) = whiteboard_world_pos {
                    // Add image to whiteboard
                    self.whiteboard.objects.push(WbObject::Image {
                        x: wpos[0], y: wpos[1],
                        width: 200.0, height: 150.0,
                        url: url.clone(),
                    });
                    self.whiteboard.dirty = true;
                    self.whiteboard.pending_image_world_pos = None;
                } else {
                    self.chat.pending_image_url = Some(url);
                }
            }
            AppEvent::ImageLoaded { url, image } => {
                if let Some(color_image) = image {
                    let tex = ctx.load_texture(&url, color_image, egui::TextureOptions::default());
                    self.image_cache.insert(url, tex);
                } else {
                    self.image_cache.mark_failed(&url);
                }
            }
            AppEvent::WhiteboardLoaded { thread_id, objects } => {
                if self.chat.thread_id == Some(thread_id) {
                    self.whiteboard.objects = objects;
                    self.whiteboard.dirty = false;
                }
            }
            AppEvent::AdminUsersLoaded(users) => { self.admin_users = users; }
            AppEvent::AdminConfigLoaded(cfg) => { self.admin_config = cfg; }
            AppEvent::AdminNodesLoaded(nodes) => { self.admin_nodes = nodes; }
            AppEvent::StorageInfoLoaded(info) => { self.admin_storage = info; }
            AppEvent::IpBansLoaded(ips) => { self.admin_ip_bans = ips; }
            AppEvent::InputLocked => { self.input_locked = true; }
            AppEvent::InputUnlocked => { self.input_locked = false; }
            AppEvent::Error(msg) => {
                self.toast = Some((msg.clone(), Instant::now()));
                self.login_error = Some(msg);
                self.pending_upload = false;
            }
        }
    }

    // ── Async spawns ──────────────────────────────────────────────────────────

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
                Ok(r) => AppEvent::LoginSuccess {
                    token: r.token, user_id: r.user_id,
                    username: r.username, is_admin: r.is_admin, is_guest: r.is_guest,
                },
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

    pub fn spawn_load_active_nodes(&mut self) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            if let Ok(ids) = crate::api::nodes::get_active(&client, &token).await {
                let _ = tx.send(AppEvent::ActiveNodesLoaded(ids));
                ctx.request_repaint();
            }
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

    pub fn spawn_pin_message_timed(&mut self, message_id: i64, pin: bool, duration_minutes: Option<u32>) {
        let Some(thread_id) = self.chat.thread_id else { return };
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let result = if pin {
                crate::api::pins::pin_timed(&client, &token, message_id, duration_minutes).await
            } else {
                crate::api::pins::unpin(&client, &token, message_id).await
            };
            let ev = match result {
                Ok(_) => {
                    match crate::api::pins::get_pinned(&client, &token, thread_id).await {
                        Ok(messages) => AppEvent::PinnedLoaded { thread_id, messages },
                        Err(e) => AppEvent::Error(e.to_string()),
                    }
                }
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    // Keep backward compat for existing callers
    pub fn spawn_pin_message(&mut self, message_id: i64, pin: bool) {
        self.spawn_pin_message_timed(message_id, pin, None);
    }

    pub fn spawn_upload_image(&mut self, path: std::path::PathBuf) {
        self.pending_upload = true;
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        let wb_pos = self.whiteboard.pending_image_world_pos;
        self.rt.spawn(async move {
            let ev = match tokio::fs::read(&path).await {
                Ok(bytes) => {
                    let filename = path.file_name()
                        .and_then(|n| n.to_str()).unwrap_or("image.png").to_string();
                    let mime = if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                        "image/jpeg"
                    } else { "image/png" };
                    match crate::api::uploads::upload_image(&client, &token, bytes, filename, mime).await {
                        Ok(url) => AppEvent::UploadComplete { url, whiteboard_world_pos: wb_pos },
                        Err(e) => AppEvent::Error(e.to_string()),
                    }
                }
                Err(e) => AppEvent::Error(e.to_string()),
            };
            let _ = tx.send(ev);
            ctx.request_repaint();
        });
    }

    pub fn spawn_load_whiteboard(&mut self, thread_id: i64) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let objects = crate::api::whiteboard::get(&client, &token, thread_id)
                .await
                .unwrap_or(None)
                .unwrap_or_default();
            let _ = tx.send(AppEvent::WhiteboardLoaded { thread_id, objects });
            ctx.request_repaint();
        });
    }

    pub fn spawn_save_whiteboard(&mut self, thread_id: i64) {
        self.whiteboard.dirty = false;
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let objects = self.whiteboard.objects.clone();
        self.rt.spawn(async move {
            if let Err(e) = crate::api::whiteboard::save(&client, &token, thread_id, &objects).await {
                let _ = tx.send(AppEvent::Error(e.to_string()));
            }
        });
    }

    // ── Admin spawns ──────────────────────────────────────────────────────────

    pub fn spawn_load_admin_data(&mut self) {
        let client = self.api.clone();
        let token = self.auth.token.clone();
        let tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            if let Ok(users) = crate::api::admin::get_users(&client, &token).await {
                let _ = tx.send(AppEvent::AdminUsersLoaded(users));
            }
            if let Ok(cfg) = crate::api::admin::get_config(&client, &token).await {
                let _ = tx.send(AppEvent::AdminConfigLoaded(cfg));
            }
            if let Ok(nodes) = crate::api::admin::get_nodes(&client, &token).await {
                let _ = tx.send(AppEvent::AdminNodesLoaded(nodes));
            }
            if let Ok(info) = crate::api::admin::get_storage(&client, &token).await {
                let _ = tx.send(AppEvent::StorageInfoLoaded(info));
            }
            if let Ok(ips) = crate::api::admin::get_ip_bans(&client, &token).await {
                let _ = tx.send(AppEvent::IpBansLoaded(ips));
            }
            ctx.request_repaint();
        });
    }

    pub fn spawn_admin_ban_user(&mut self, user_id: i64, ban: bool) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::set_user_banned(&client, &token, user_id, ban).await;
            if let Ok(users) = crate::api::admin::get_users(&client, &token).await {
                let _ = tx.send(AppEvent::AdminUsersLoaded(users));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_set_admin(&mut self, user_id: i64, admin: bool) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::set_user_admin(&client, &token, user_id, admin).await;
            if let Ok(users) = crate::api::admin::get_users(&client, &token).await {
                let _ = tx.send(AppEvent::AdminUsersLoaded(users));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_delete_user(&mut self, user_id: i64) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::delete_user(&client, &token, user_id).await;
            if let Ok(users) = crate::api::admin::get_users(&client, &token).await {
                let _ = tx.send(AppEvent::AdminUsersLoaded(users));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_create_node(&mut self, t: String, name: String, parent_id: Option<i64>) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::create_node(&client, &token, &t, &name, parent_id).await;
            if let Ok(nodes) = crate::api::admin::get_nodes(&client, &token).await {
                let _ = tx.send(AppEvent::AdminNodesLoaded(nodes));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_delete_node(&mut self, node_id: i64) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::delete_node(&client, &token, node_id).await;
            if let Ok(nodes) = crate::api::admin::get_nodes(&client, &token).await {
                let _ = tx.send(AppEvent::AdminNodesLoaded(nodes));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_set_config(&mut self, key: String, value: String) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::set_config(&client, &token, &key, &value).await;
            if let Ok(cfg) = crate::api::admin::get_config(&client, &token).await {
                let _ = tx.send(AppEvent::AdminConfigLoaded(cfg));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_purge_storage(&mut self) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::purge_storage(&client, &token).await;
            if let Ok(info) = crate::api::admin::get_storage(&client, &token).await {
                let _ = tx.send(AppEvent::StorageInfoLoaded(info));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_add_ip_ban(&mut self, ip: String) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::add_ip_ban(&client, &token, &ip).await;
            if let Ok(ips) = crate::api::admin::get_ip_bans(&client, &token).await {
                let _ = tx.send(AppEvent::IpBansLoaded(ips));
                ctx.request_repaint();
            }
        });
    }

    pub fn spawn_admin_remove_ip_ban(&mut self, ip: String) {
        let client = self.api.clone(); let token = self.auth.token.clone();
        let tx = self.event_tx.clone(); let ctx = self.egui_ctx.clone();
        self.rt.spawn(async move {
            let _ = crate::api::admin::remove_ip_ban(&client, &token, &ip).await;
            if let Ok(ips) = crate::api::admin::get_ip_bans(&client, &token).await {
                let _ = tx.send(AppEvent::IpBansLoaded(ips));
                ctx.request_repaint();
            }
        });
    }

    // ── WebSocket ─────────────────────────────────────────────────────────────

    fn start_ws(&mut self, thread_id: i64) {
        if let Some(h) = self.ws_abort.take() { h.abort(); }

        let (ws_tx, ws_rx) = mpsc::sync_channel::<ws::WsEvent>(256);
        let event_tx = self.event_tx.clone();
        let ctx = self.egui_ctx.clone();

        std::thread::spawn(move || {
            for ev in ws_rx {
                let app_ev = match ev {
                    ws::WsEvent::NewMessage(m) => AppEvent::MessageReceived(m),
                    ws::WsEvent::InputLocked => AppEvent::InputLocked,
                    ws::WsEvent::InputUnlocked => AppEvent::InputUnlocked,
                    ws::WsEvent::WhiteboardUpdate(objects) => {
                        AppEvent::WhiteboardLoaded { thread_id, objects }
                    }
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

    // ── Nav change ────────────────────────────────────────────────────────────

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

    // ── Image prefetch ────────────────────────────────────────────────────────

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

    // ── Toast ─────────────────────────────────────────────────────────────────

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

    // ── Server switching ──────────────────────────────────────────────────────

    pub fn switch_server(&mut self, idx: usize) {
        let profile = self.servers[idx].clone();
        self.logout();
        self.login_server_url = profile.url.clone();
        self.server_url = profile.url.clone();
        self.ws_url = derive_ws_url(&profile.url);
        self.api = ApiClient::new(&profile.url);
    }

    pub fn logout(&mut self) {
        let _ = clear_keyring();
        self.auth = AuthState::default();
        self.is_admin = false;
        self.is_guest = false;
        self.chat = ChatState::default();
        self.nav = NavState::default();
        self.nav_snapshot = NavState::default();
        self.nodes.clear();
        self.active_nodes.clear();
        self.thread_mode.clear();
        self.whiteboard = WhiteboardState::new();
        self.input_locked = false;
        if let Some(h) = self.ws_abort.take() { h.abort(); }
        self.screen = Screen::Login;
    }

    pub fn update_server_url(&mut self, url: String) {
        self.server_url = url.clone();
        self.ws_url = derive_ws_url(&url);
        self.api = ApiClient::new(&url);
    }
}

// ── eframe::App impl ──────────────────────────────────────────────────────────

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
        eframe::set_value(storage, "servers", &self.servers);
        let theme_str = match self.theme {
            Theme::Dark => "dark", Theme::Light => "light",
            Theme::Nebula => "nebula", Theme::OledBlack => "oled",
        };
        eframe::set_value(storage, "theme", &theme_str.to_string());
        eframe::set_value(storage, "font_size", &self.font_size);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn derive_ws_url(http_url: &str) -> String {
    http_url.replace("https://", "wss://").replace("http://", "ws://")
}

fn keyring_entry(key: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new("val-tactics", key)
}

fn try_load_token() -> Result<String, keyring::Error> { keyring_entry("token")?.get_password() }
fn try_load_user_id() -> Result<i64, Box<dyn std::error::Error>> {
    Ok(keyring_entry("user_id")?.get_password()?.parse()?)
}
fn try_load_username() -> Result<String, keyring::Error> { keyring_entry("username")?.get_password() }
fn save_token(token: &str) -> Result<(), keyring::Error> { keyring_entry("token")?.set_password(token) }
fn save_user_id(id: i64) -> Result<(), keyring::Error> { keyring_entry("user_id")?.set_password(&id.to_string()) }
fn save_username(name: &str) -> Result<(), keyring::Error> { keyring_entry("username")?.set_password(name) }
fn clear_keyring() -> Result<(), keyring::Error> {
    let _ = keyring_entry("token")?.delete_password();
    let _ = keyring_entry("user_id")?.delete_password();
    let _ = keyring_entry("username")?.delete_password();
    Ok(())
}
