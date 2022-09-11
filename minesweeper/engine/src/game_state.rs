use crate::CompleteState;

#[derive(Debug, Copy, Clone, Default)]
pub enum GameState {
    Initialised,
    #[default]
    Playing,
    Complete(CompleteState),
}
