#[derive(Debug, Clone, Default)]
pub struct AuthState {
    pub token: String,
    pub username: String,
    pub user_id: i64,
}
