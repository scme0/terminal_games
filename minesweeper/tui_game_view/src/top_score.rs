use std::collections::HashMap;
use tui::screen::GameType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopScore {
    pub scores: HashMap<GameType,u64>
}
