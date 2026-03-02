#[derive(Debug)]
pub enum GameError {
    InvalidChoice,
    MissingItem(String),
    SceneNotFound(String),
}