use crate::GameState;

#[derive(Debug, Copy, Clone)]
pub struct GameStats {
    pub game_state: GameState,
    pub flags_remaining: i32,
    pub game_run_time: u64,
}
