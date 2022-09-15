use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use game_actions::game_type::GameType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopScore {
    pub scores: HashMap<GameType,u64>
}
