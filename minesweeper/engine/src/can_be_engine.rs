use std::collections::HashMap;
use crate::cell::Cell;
use crate::cell_state::CellState;
use crate::game_state::GameState;
use crate::game_stats::GameStats;
use crate::move_type::MoveType;
use crate::zero_to_eight::ZeroToEight;

pub trait CanBeEngine {
    fn get_size(&self) -> (i32, i32);
    fn get_board_state(&self) -> (GameStats, HashMap<Cell,CellState>);
    fn play_move(&mut self, move_type: MoveType, cell: Cell) -> crossterm::Result<GameState>;
    fn make_clone(&self) -> Box<dyn CanBeEngine>;
    fn get_chill_factor(&mut self, cell: Cell) -> crossterm::Result<ZeroToEight>;
}
