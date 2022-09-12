use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GameType {
    Easy,
    Medium,
    Hard,
}
