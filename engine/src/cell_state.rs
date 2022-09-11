use crate::ZeroToEight;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CellState {
    Unchecked,
    Checked(ZeroToEight),
    Flagged,
    Bomb,
    Cross,
    Exploded
}
