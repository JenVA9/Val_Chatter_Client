use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WbObject {
    Text {
        x: f32,
        y: f32,
        content: String,
        color: [u8; 4],
        font_size: f32,
    },
    Stroke {
        points: Vec<[f32; 2]>,
        color: [u8; 4],
        width: f32,
    },
    Image {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        url: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WbTool {
    Select,
    Text,
    Draw,
    Image,
}

impl Default for WbTool {
    fn default() -> Self { Self::Draw }
}

#[derive(Debug, Clone)]
pub struct WhiteboardState {
    pub objects: Vec<WbObject>,
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub tool: WbTool,
    pub color: egui::Color32,
    pub stroke_width: f32,
    pub current_stroke: Option<Vec<egui::Pos2>>,
    pub text_input: Option<(egui::Pos2, String)>,
    pub selected_idx: Option<usize>,
    pub dirty: bool,
    // When uploading an image while in whiteboard mode, store target world position
    pub pending_image_world_pos: Option<[f32; 2]>,
}

impl Default for WhiteboardState {
    fn default() -> Self { Self::new() }
}

impl WhiteboardState {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            tool: WbTool::Draw,
            color: egui::Color32::from_rgb(255, 80, 80),
            stroke_width: 2.0,
            current_stroke: None,
            text_input: None,
            selected_idx: None,
            dirty: false,
            pending_image_world_pos: None,
        }
    }

    pub fn world_to_screen(&self, world: egui::Pos2, canvas_min: egui::Pos2) -> egui::Pos2 {
        egui::Pos2::new(
            canvas_min.x + world.x * self.zoom + self.pan.x,
            canvas_min.y + world.y * self.zoom + self.pan.y,
        )
    }

    pub fn screen_to_world(&self, screen: egui::Pos2, canvas_min: egui::Pos2) -> egui::Pos2 {
        egui::Pos2::new(
            (screen.x - canvas_min.x - self.pan.x) / self.zoom,
            (screen.y - canvas_min.y - self.pan.y) / self.zoom,
        )
    }
}
