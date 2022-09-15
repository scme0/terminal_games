use minesweeper_engine::can_be_engine::CanBeEngine;
use minesweeper_engine::game_stats::GameStats;
use std::collections::HashMap;
use minesweeper_engine::cell::Cell;
use minesweeper_engine::cell_state::CellState;
use minesweeper_engine::cell_state::CellState::{Bomb, Checked, Flagged, Unchecked};
use minesweeper_engine::game_state::GameState;
use minesweeper_engine::game_state::GameState::Playing;
use minesweeper_engine::move_type::MoveType;
use minesweeper_engine::zero_to_eight::ZeroToEight;
use minesweeper_engine::zero_to_eight::ZeroToEight::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};

pub struct TestEngine {
    updated: bool
}

impl TestEngine {
    pub fn new() -> Self {
        TestEngine { updated: false }
    }
}

impl CanBeEngine for TestEngine {
    fn get_size(&self) -> (i32, i32) {
        return (4,3);
    }

    fn get_game_stats(&self) -> GameStats {
        return GameStats{game_state: Playing, flags_remaining:33, game_run_time: 999};
    }

    fn get_board_updates(&mut self) -> HashMap<Cell, CellState> {
        self.get_board_state()
    }

    fn get_board_state(&mut self) -> HashMap<Cell, CellState> {
        let mut map = HashMap::new();
        if !self.updated {
            map.insert(Cell{y: 0, x: 0}, Checked(Zero));
            map.insert(Cell{y: 0, x: 1}, Checked(One));
            map.insert(Cell{y: 0, x: 2}, Checked(Two));
            map.insert(Cell{y: 1, x: 0}, Checked(Three));
            map.insert(Cell{y: 1, x: 1}, Checked(Four));
            map.insert(Cell{y: 1, x: 2}, Checked(Five));
            map.insert(Cell{y: 2, x: 0}, Checked(Six));
            map.insert(Cell{y: 2, x: 1}, Checked(Seven));
            map.insert(Cell{y: 2, x: 2}, Checked(Eight));
            map.insert(Cell{y: 3, x: 0}, Unchecked);
            map.insert(Cell{y: 3, x: 1}, Flagged);
            map.insert(Cell{y: 3, x: 2}, Bomb);
        }
        return map;
    }

    fn play_move(&mut self, _: MoveType, _: Cell) -> crossterm::Result<GameState> {
        self.updated = true;
        Ok(Playing)
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(TestEngine::new())
    }

    fn get_chill_factor(&mut self, _: Cell) -> crossterm::Result<ZeroToEight> {
        Ok(Eight)
    }
}
