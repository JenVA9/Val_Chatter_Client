use crate::app::{App, AdminTab};
use crate::models::NodeType;

pub fn show(ctx: &egui::Context, app: &mut App) {
    if !app.admin_panel_open { return; }

    let mut open = app.admin_panel_open;

    egui::Window::new("Admin Panel")
        .open(&mut open)
        .resizable(true)
        .min_size([640.0, 480.0])
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                tab_btn(ui, "Users",      AdminTab::Users,      &mut app.admin_tab);
                tab_btn(ui, "Categories", AdminTab::Categories, &mut app.admin_tab);
                tab_btn(ui, "Config",     AdminTab::Config,     &mut app.admin_tab);
                tab_btn(ui, "Storage",    AdminTab::Storage,    &mut app.admin_tab);
                tab_btn(ui, "IP Bans",    AdminTab::IpBans,     &mut app.admin_tab);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Refresh").clicked() {
                        app.spawn_load_admin_data();
                    }
                });
            });

            ui.separator();

            match app.admin_tab {
                AdminTab::Users      => show_users(ui, app),
                AdminTab::Categories => show_categories(ui, app),
                AdminTab::Config     => show_config(ui, app),
                AdminTab::Storage    => show_storage(ui, app),
                AdminTab::IpBans     => show_ip_bans(ui, app),
            }
        });

    app.admin_panel_open = open;
}

// ── Users tab ─────────────────────────────────────────────────────────────────

fn show_users(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if app.admin_users.is_empty() {
            ui.label(egui::RichText::new("No users loaded. Click Refresh.").weak().italics());
            return;
        }

        let users_copy = app.admin_users.clone();
        for user in &users_copy {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&user.username).strong());
                if user.is_admin { ui.label(egui::RichText::new("[Admin]").color(egui::Color32::from_rgb(255, 180, 0)).small()); }
                if user.is_guest { ui.label(egui::RichText::new("[Guest]").color(egui::Color32::from_gray(150)).small()); }
                if user.is_banned { ui.label(egui::RichText::new("[Banned]").color(egui::Color32::RED).small()); }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let uid = user.id;
                    let is_me = uid == app.auth.user_id;
                    if !is_me {
                        if ui.small_button("Delete").clicked() { app.spawn_admin_delete_user(uid); }
                    }
                    if !user.is_guest && !is_me {
                        if ui.small_button(if user.is_banned { "Unban" } else { "Ban" }).clicked() {
                            app.spawn_admin_ban_user(uid, !user.is_banned);
                        }
                        if ui.small_button(if user.is_admin { "Revoke Admin" } else { "Make Admin" }).clicked() {
                            app.spawn_admin_set_admin(uid, !user.is_admin);
                        }
                    }
                });
            });
            ui.separator();
        }
    });
}

// ── Categories tab ────────────────────────────────────────────────────────────

fn show_categories(ui: &mut egui::Ui, app: &mut App) {
    // ── Add item form ──────────────────────────────────────────────────────
    ui.group(|ui| {
        ui.label(egui::RichText::new("Add item to category").strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut app.admin_new_node_name)
                .hint_text("Name").desired_width(120.0));

            egui::ComboBox::from_id_source("admin_node_type")
                .selected_text(type_display_name(&app.admin_new_node_type))
                .show_ui(ui, |ui| {
                    for t in &["map", "agent", "site", "tactic_type", "agent_combo"] {
                        ui.selectable_value(
                            &mut app.admin_new_node_type,
                            t.to_string(),
                            type_display_name(t),
                        );
                    }
                });

            if app.admin_new_node_type == "site" {
                let parent_label = app.admin_new_node_parent_id
                    .and_then(|pid| app.admin_nodes.iter().find(|n| n.id == pid))
                    .map(|n| n.name.as_str())
                    .unwrap_or("-- select map --");

                egui::ComboBox::from_id_source("admin_node_parent")
                    .selected_text(parent_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.admin_new_node_parent_id, None, "-- none --");
                        let maps: Vec<_> = app.admin_nodes.iter()
                            .filter(|n| n.node_type == NodeType::Map)
                            .map(|n| (n.id, n.name.clone()))
                            .collect();
                        for (mid, mname) in maps {
                            ui.selectable_value(&mut app.admin_new_node_parent_id, Some(mid), mname);
                        }
                    });
            } else {
                app.admin_new_node_parent_id = None;
            }

            if ui.button("Add").clicked() && !app.admin_new_node_name.trim().is_empty() {
                let name = std::mem::take(&mut app.admin_new_node_name).trim().to_string();
                let t = app.admin_new_node_type.clone();
                let pid = app.admin_new_node_parent_id;
                app.spawn_admin_create_node(t, name, pid);
            }
        });
    });

    ui.add_space(4.0);

    // ── Items grouped by category ──────────────────────────────────────────
    egui::ScrollArea::vertical().show(ui, |ui| {
        let nodes_copy = app.admin_nodes.clone();

        for (type_key, type_label) in &[
            ("map",         "Maps"),
            ("agent",       "Agents"),
            ("site",        "Sites"),
            ("tactic_type", "Tactic Types"),
            ("agent_combo", "Agent Combos"),
        ] {
            let group: Vec<_> = nodes_copy.iter()
                .filter(|n| n.node_type_str() == *type_key)
                .collect();
            if group.is_empty() { continue; }

            ui.add_space(6.0);
            ui.label(egui::RichText::new(*type_label).strong()
                .color(egui::Color32::from_rgb(120, 180, 230)));
            ui.separator();

            for node in group {
                let nid = node.id;
                let is_renaming = app.admin_rename_node_id == Some(nid);

                ui.horizontal(|ui| {
                    if is_renaming {
                        ui.add(egui::TextEdit::singleline(&mut app.admin_rename_name)
                            .desired_width(150.0));
                        if ui.small_button("Save").clicked() {
                            let new_name = std::mem::take(&mut app.admin_rename_name).trim().to_string();
                            if !new_name.is_empty() {
                                app.spawn_admin_rename_node(nid, new_name);
                            }
                            app.admin_rename_node_id = None;
                        }
                        if ui.small_button("Cancel").clicked() {
                            app.admin_rename_node_id = None;
                        }
                    } else {
                        if node.node_type == NodeType::Site {
                            if let Some(pid) = node.parent_id {
                                if let Some(parent) = nodes_copy.iter().find(|n| n.id == pid) {
                                    ui.label(egui::RichText::new(format!("[{}]", parent.name)).small().weak());
                                }
                            }
                        }
                        ui.label(&node.name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Del").clicked() {
                                app.spawn_admin_delete_node(nid);
                            }
                            if ui.small_button("Rename").clicked() {
                                app.admin_rename_node_id = Some(nid);
                                app.admin_rename_name = node.name.clone();
                            }
                        });
                    }
                });
            }
        }
    });
}

// ── Config tab ────────────────────────────────────────────────────────────────

fn show_config(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if app.admin_config.is_empty() {
            ui.label(egui::RichText::new("No config loaded. Click Refresh.").weak().italics());
            return;
        }
        let config_copy = app.admin_config.clone();
        for (i, (key, _value)) in config_copy.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(key);
                let mut v = app.admin_config[i].1.clone();
                if ui.add(egui::TextEdit::singleline(&mut v).desired_width(160.0)).changed() {
                    app.admin_config[i].1 = v.clone();
                }
                if ui.small_button("Save").clicked() {
                    let k = key.clone();
                    let val = v.clone();
                    app.spawn_admin_set_config(k, val);
                }
            });
        }
    });
}

// ── Storage tab ───────────────────────────────────────────────────────────────

fn show_storage(ui: &mut egui::Ui, app: &mut App) {
    let used_gb = app.admin_storage.used_bytes as f64 / 1024.0_f64.powi(3);
    let limit_gb = app.admin_storage.limit_bytes as f64 / 1024.0_f64.powi(3);

    ui.label(format!("Used: {:.2} GB", used_gb));

    if app.admin_storage.limit_bytes > 0 {
        let fraction = (app.admin_storage.used_bytes as f32 / app.admin_storage.limit_bytes as f32).clamp(0.0, 1.0);
        let bar_color = if fraction > 0.9 { egui::Color32::RED } else { egui::Color32::from_rgb(70, 170, 70) };
        let rect = ui.available_rect_before_wrap();
        let bar_rect = egui::Rect::from_min_size(rect.min, egui::Vec2::new(rect.width(), 16.0));
        ui.allocate_rect(bar_rect, egui::Sense::hover());
        ui.painter().rect_filled(bar_rect, 4.0, egui::Color32::from_gray(40));
        ui.painter().rect_filled(
            egui::Rect::from_min_size(bar_rect.min, egui::Vec2::new(bar_rect.width() * fraction, 16.0)),
            4.0, bar_color,
        );
        ui.label(format!("Limit: {:.2} GB", limit_gb));
    } else {
        ui.label("No storage limit set.");
    }

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Set limit (GB, 0 = unlimited):");
        if app.admin_storage_limit_input.is_empty() {
            app.admin_storage_limit_input = format!("{:.0}", limit_gb);
        }
        ui.add(egui::TextEdit::singleline(&mut app.admin_storage_limit_input).desired_width(60.0));
        if ui.small_button("Set").clicked() {
            let val = app.admin_storage_limit_input.clone();
            app.spawn_admin_set_config("storage_limit_gb".to_string(), val);
        }
    });

    ui.separator();
    ui.label(egui::RichText::new("Danger zone:").color(egui::Color32::RED));
    if ui.add(egui::Button::new("Purge ALL Uploads").fill(egui::Color32::from_rgb(120, 20, 20))).clicked() {
        app.spawn_admin_purge_storage();
    }
}

// ── IP Bans tab ───────────────────────────────────────────────────────────────

fn show_ip_bans(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut app.admin_new_ip)
            .hint_text("IP address").desired_width(160.0));
        if ui.small_button("Ban IP").clicked() && !app.admin_new_ip.trim().is_empty() {
            let ip = std::mem::take(&mut app.admin_new_ip).trim().to_string();
            app.spawn_admin_add_ip_ban(ip);
        }
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        let bans_copy = app.admin_ip_bans.clone();
        for ip in &bans_copy {
            ui.horizontal(|ui| {
                ui.label(ip);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Remove").clicked() {
                        app.spawn_admin_remove_ip_ban(ip.clone());
                    }
                });
            });
        }
    });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn tab_btn(ui: &mut egui::Ui, label: &str, tab: AdminTab, current: &mut AdminTab) {
    let selected = *current == tab;
    let btn = egui::Button::new(label)
        .fill(if selected { egui::Color32::from_rgb(40, 80, 130) } else { egui::Color32::TRANSPARENT });
    if ui.add(btn).clicked() {
        *current = tab;
    }
}

fn type_display_name(t: &str) -> &str {
    match t {
        "map"         => "Map",
        "agent"       => "Agent",
        "site"        => "Site",
        "tactic_type" => "Tactic Type",
        "agent_combo" => "Agent Combo",
        other         => other,
    }
}
