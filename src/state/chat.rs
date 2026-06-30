use crate::models::Message;

#[derive(Debug, Default)]
pub struct ChatState {
    pub thread_id: Option<i64>,
    pub messages: Vec<Message>,
    pub pinned: Vec<Message>,
    pub input_buffer: String,
    pub pending_image_url: Option<String>,
    pub scroll_to_bottom: bool,
}
