use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tui::screen::GameType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopScore {
    pub scores: HashMap<GameType,u64>
}
