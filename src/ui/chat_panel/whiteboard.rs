use egui::{Color32, Pos2, Sense, Stroke, Vec2};
use crate::app::App;
use crate::models::{WbObject, WbTool, WhiteboardState};

pub fn show(ctx: &egui::Context, ui: &mut egui::Ui, app: &mut App) {
    let thread_id = match app.chat.thread_id {
        Some(id) => id,
        None => return,
    };

    // ── Toolbar ───────────────────────────────────────────────────────────
    // Use separate block so wb borrow ends before canvas section
    let (tool_clone, switch_to_chat, do_clear, do_save) = {
        let wb = &mut app.whiteboard;
        let mut switch_to_chat = false;
        let mut do_clear = false;
        let mut do_save = false;

        ui.horizontal(|ui| {
            tool_btn(ui, "🔲 Select", WbTool::Select, &mut wb.tool);
            tool_btn(ui, "T Text",   WbTool::Text,   &mut wb.tool);
            tool_btn(ui, "✏ Draw",  WbTool::Draw,   &mut wb.tool);
            tool_btn(ui, "🖼 Image", WbTool::Image,  &mut wb.tool);

            ui.separator();
            ui.color_edit_button_srgba(&mut wb.color);
            ui.add(egui::Slider::new(&mut wb.stroke_width, 1.0..=20.0).text("W"));

            ui.separator();
            if ui.button("Clear").clicked() { do_clear = true; }
            if ui.add_enabled(wb.dirty, egui::Button::new("💾 Save")).clicked() { do_save = true; }
            ui.separator();
            if ui.button("💬 Chat").clicked() { switch_to_chat = true; }
        });

        (wb.tool.clone(), switch_to_chat, do_clear, do_save)
    };

    if switch_to_chat {
        app.thread_mode.insert(thread_id, crate::app::ThreadMode::Chat);
    }
    if do_clear {
        app.whiteboard.objects.clear();
        app.whiteboard.dirty = true;
    }
    if do_save {
        app.spawn_save_whiteboard(thread_id);
    }

    ui.separator();

    // ── Canvas ────────────────────────────────────────────────────────────
    let available = ui.available_size();
    let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());
    let canvas_min = response.rect.min;

    // Pan: right-drag or Space + left-drag
    let space_held = ctx.input(|i| i.key_down(egui::Key::Space));
    if response.dragged()
        && (response.dragged_by(egui::PointerButton::Middle) || space_held)
    {
        app.whiteboard.pan += response.drag_delta();
        ctx.request_repaint();
    }

    // Zoom: scroll wheel
    let scroll_y = ctx.input(|i| i.raw_scroll_delta.y);
    if scroll_y != 0.0 && response.hovered() {
        let factor = if scroll_y > 0.0 { 1.1_f32 } else { 0.9_f32 };
        app.whiteboard.zoom = (app.whiteboard.zoom * factor).clamp(0.05, 10.0);
        ctx.request_repaint();
    }

    // Background
    painter.rect_filled(response.rect, 0.0, Color32::from_rgb(28, 28, 32));

    // Grid dots
    {
        let grid = 40.0 * app.whiteboard.zoom;
        if grid > 8.0 {
            let off_x = app.whiteboard.pan.x.rem_euclid(grid);
            let off_y = app.whiteboard.pan.y.rem_euclid(grid);
            let mut x = canvas_min.x + off_x;
            while x < response.rect.max.x {
                let mut y = canvas_min.y + off_y;
                while y < response.rect.max.y {
                    painter.circle_filled(Pos2::new(x, y), 1.0, Color32::from_gray(55));
                    y += grid;
                }
                x += grid;
            }
        }
    }

    // Render objects
    for obj in &app.whiteboard.objects {
        render_object(&painter, obj, &app.whiteboard, canvas_min);
    }

    // Render in-progress stroke
    if let Some(pts) = &app.whiteboard.current_stroke {
        if pts.len() >= 2 {
            let screen_pts: Vec<Pos2> = pts.iter()
                .map(|p| app.whiteboard.world_to_screen(*p, canvas_min))
                .collect();
            painter.add(egui::Shape::line(
                screen_pts,
                Stroke::new(app.whiteboard.stroke_width * app.whiteboard.zoom, app.whiteboard.color),
            ));
        }
    }

    // ── Tool interactions ─────────────────────────────────────────────────
    match tool_clone {
        WbTool::Draw  => handle_draw(ctx, &response, &mut app.whiteboard, canvas_min),
        WbTool::Text  => handle_text(&response, &mut app.whiteboard, canvas_min),
        WbTool::Image => handle_image(&response, app, canvas_min),
        WbTool::Select => {}
    }

    // ── Text input popup ──────────────────────────────────────────────────
    let text_pos = app.whiteboard.text_input.as_ref().map(|(p, _)| *p);
    if let Some(world_pos) = text_pos {
        let screen_pos = app.whiteboard.world_to_screen(world_pos, canvas_min);
        let mut text = app.whiteboard.text_input.as_ref().unwrap().1.clone();
        let committed = egui::Area::new(egui::Id::new("wb_text_area"))
            .fixed_pos(screen_pos)
            .show(ctx, |ui| {
                ui.set_min_width(140.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .hint_text("Type, press Enter"),
                );
                resp.request_focus();
                let committed = resp.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter));
                let cancelled = ctx.input(|i| i.key_pressed(egui::Key::Escape));
                if !committed && !cancelled {
                    if let Some(ti) = &mut app.whiteboard.text_input {
                        ti.1 = text.clone();
                    }
                }
                committed || cancelled
            })
            .inner;

        if committed {
            let content = app.whiteboard.text_input.take().map(|(_, t)| t).unwrap_or_default();
            if !content.is_empty() {
                let c = app.whiteboard.color;
                app.whiteboard.objects.push(WbObject::Text {
                    x: world_pos.x, y: world_pos.y,
                    content,
                    color: [c.r(), c.g(), c.b(), c.a()],
                    font_size: 16.0,
                });
                app.whiteboard.dirty = true;
            }
        }
    }
}

fn render_object(painter: &egui::Painter, obj: &WbObject, wb: &WhiteboardState, canvas_min: Pos2) {
    match obj {
        WbObject::Text { x, y, content, color, font_size } => {
            painter.text(
                wb.world_to_screen(Pos2::new(*x, *y), canvas_min),
                egui::Align2::LEFT_TOP,
                content,
                egui::FontId::proportional(font_size * wb.zoom),
                Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]),
            );
        }
        WbObject::Stroke { points, color, width } => {
            if points.len() < 2 { return; }
            let pts: Vec<Pos2> = points.iter()
                .map(|p| wb.world_to_screen(Pos2::new(p[0], p[1]), canvas_min))
                .collect();
            painter.add(egui::Shape::line(
                pts,
                Stroke::new(width * wb.zoom, Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])),
            ));
        }
        WbObject::Image { x, y, width, height, .. } => {
            let tl = wb.world_to_screen(Pos2::new(*x, *y), canvas_min);
            let br = wb.world_to_screen(Pos2::new(x + width, y + height), canvas_min);
            painter.rect_stroke(
                egui::Rect::from_min_max(tl, br), 4.0,
                Stroke::new(1.0, Color32::from_gray(120)),
            );
            painter.text(tl + Vec2::new(4.0, 4.0), egui::Align2::LEFT_TOP, "🖼",
                egui::FontId::proportional(14.0), Color32::from_gray(180));
        }
    }
}

fn handle_draw(ctx: &egui::Context, response: &egui::Response, wb: &mut WhiteboardState, canvas_min: Pos2) {
    if response.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            let world = wb.screen_to_world(pos, canvas_min);
            if let Some(stroke) = &mut wb.current_stroke {
                stroke.push(world);
            } else {
                wb.current_stroke = Some(vec![world]);
            }
            ctx.request_repaint();
        }
    } else if let Some(stroke) = wb.current_stroke.take() {
        if stroke.len() >= 2 {
            let c = wb.color;
            wb.objects.push(WbObject::Stroke {
                points: stroke.iter().map(|p| [p.x, p.y]).collect(),
                color: [c.r(), c.g(), c.b(), c.a()],
                width: wb.stroke_width,
            });
            wb.dirty = true;
        }
    }
}

fn handle_text(response: &egui::Response, wb: &mut WhiteboardState, canvas_min: Pos2) {
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let world = wb.screen_to_world(pos, canvas_min);
            wb.text_input = Some((world, String::new()));
        }
    }
}

fn handle_image(response: &egui::Response, app: &mut App, canvas_min: Pos2) {
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let world = app.whiteboard.screen_to_world(pos, canvas_min);
            let path = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .pick_file();
            if let Some(p) = path {
                app.whiteboard.pending_image_world_pos = Some([world.x, world.y]);
                app.spawn_upload_image(p);
            }
        }
    }
}

fn tool_btn(ui: &mut egui::Ui, label: &str, tool: WbTool, current: &mut WbTool) {
    let selected = *current == tool;
    let btn = egui::Button::new(label)
        .fill(if selected { Color32::from_rgb(50, 100, 160) } else { Color32::TRANSPARENT });
    if ui.add(btn).clicked() {
        *current = tool;
    }
}
