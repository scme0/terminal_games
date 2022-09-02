use std::collections::{HashMap, HashSet};
use crate::CellState::{Bomb, Checked, Flagged, Unchecked};
use crate::CompleteState::{Lose, Win};
use crate::GameState::{Complete, Playing};
use crossterm::{ErrorKind, Result};
use rand::Rng;
use std::io;
use queues::{IsQueue, Queue, queue};
use crate::AdjacentBombs::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};

pub trait CanBeEngine {
    fn get_size(&self) -> (usize, usize);
    fn get_board_state(&self) -> (GameState, HashMap<Cell,CellState>);
    fn play_move(&mut self, move_type: MoveType, cell: Cell) -> Result<()>;
    fn make_clone(&self) -> Box<dyn CanBeEngine>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cell {
    pub x: usize,
    pub y: usize
}

impl From<Cell> for (usize, usize) {
    fn from(c: Cell) -> (usize, usize) {
        let Cell {x, y} = c;
        return (x, y);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AdjacentBombs {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

impl AdjacentBombs {
    pub fn from_u8(number: u8) -> Result<AdjacentBombs> {
        let bombs = match number {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            _ => Err(ErrorKind::new(io::ErrorKind::Other, "This number cannot be an adjacent number of bombs!"))?
        };
        Ok(bombs)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CellState {
    Unchecked,
    Checked(AdjacentBombs),
    Flagged,
    Bomb,
}

#[derive(Debug, Copy, Clone)]
pub enum CompleteState {
    Win,
    Lose,
}

#[derive(Debug, Copy, Clone)]
pub enum GameState {
    Initialised,
    Playing,
    Complete(CompleteState),
}

#[derive(Debug, Clone)]
pub struct Engine {
    game_state: GameState,
    board_play_state: HashMap<Cell,CellState>,
    board_state: HashMap<Cell,CellState>,
    board_initialised: bool,
    width: usize,
    height: usize,
    bomb_count: usize,
    checked_cells: usize,
    flagged_cells: usize,
    total_cells: usize,
}

#[derive(Debug, Clone)]
pub enum MoveType {
    Dig,
    Flag,
}

impl Engine {
    pub fn new(width: usize, height: usize, bomb_count: usize) -> Self {
        let total_cells = width * height;
        let mut board_play_state = HashMap::new();
        let mut board_state = HashMap::new();
        for x in 0..height {
            for y in 0..width {
                board_play_state.insert(Cell{x,y}, Unchecked);
                board_state.insert(Cell{x,y}, Checked(Zero));
            }
        }
        Engine {
            board_state,
            board_play_state,
            game_state: GameState::Initialised,
            width,
            height,
            bomb_count,
            checked_cells: 0,
            flagged_cells: 0,
            total_cells,
            board_initialised: false
        }
    }

    fn increment_bomb_count_of_surrounding_cells(&mut self, cell: Cell) -> Result<()> {
        self.get_surrounding_cells(cell, Some(|s, c| {
            if let Checked(bombs) = s.board_state[&c] {
                let mut bombs_as_byte = bombs as u8;
                bombs_as_byte += 1;
                let new_bombs = AdjacentBombs::from_u8(bombs_as_byte).expect("");
                s.board_state.insert(c, Checked(new_bombs));
            }
        }));
        Ok(())
    }

    fn initialise_board(&mut self, clicked_cell: Cell) -> Result<()> {
        if self.board_initialised {
            return Ok(());
        }

        let mut rng = rand::thread_rng();
        let bomb_probability = self.bomb_count as f64 / self.total_cells as f64;
        let mut get_bomb_or_not = || rng.gen_range(0.0..1.0) <= bomb_probability;
        let mut local_bomb_count = self.bomb_count;
        while local_bomb_count > 0 {
            for x in 0..self.height {
                for y in 0..self.width {
                    let cell = Cell {x, y};
                    // Ensure no bomb is placed on the clicked cell!
                    if  clicked_cell == cell {
                        continue;
                    }

                    if self.board_state[&cell] == Bomb {
                        continue;
                    }

                    if get_bomb_or_not() {
                        self.board_state.insert(cell, Bomb);
                        local_bomb_count -= 1;
                        self.increment_bomb_count_of_surrounding_cells(cell)?;
                    }
                    if local_bomb_count == 0 {
                        break;
                    }
                }
                if local_bomb_count == 0 {
                    break;
                }
            }
        }
        self.board_initialised = true;
        Ok(())
    }

    fn reveal_safe_patch(&mut self, starting_cell: Cell) -> Result<()> {
        let mut visited_cells = HashSet::new();
        let mut cell_queue = queue![starting_cell];
        while let Ok(cell) = cell_queue.remove() {
            if visited_cells.contains(&cell) {
                continue;
            }
            visited_cells.insert(cell);
            if let Checked(bombs) = self.board_state[&cell] {
                if bombs == Zero {
                    for surrounding_cell in self.get_surrounding_cells(cell, None) {
                        cell_queue.add(surrounding_cell).expect("");
                    }
                }
                self.board_play_state.insert(cell, self.board_state[&cell]);
            }
        }
        Ok(())
    }

    fn get_surrounding_cells(&mut self, cell: Cell, func: Option<fn (engine: &mut Engine,cell: Cell)>) -> Vec<Cell> {
        let (x, y) = cell.into();
        let mut cells = vec![];
        for x_s in (x as i32 - 1)..(x as i32 + 2) {
            if x_s < 0 || x_s >= self.height as i32 {
                continue;
            }
            for y_s in (y as i32 - 1)..(y as i32 + 2) {
                if y_s < 0 || y_s >= self.width as i32 {
                    continue;
                }
                if y_s == y as i32 && x_s == x as i32 {
                    continue;
                }

                let cell = Cell {x: x_s as usize, y: y_s as usize};
                if let Some(f) = func {
                    f(self, cell);
                }

                cells.push(cell);
            }
        }
        return cells;
    }
}

impl CanBeEngine for Engine {
    fn get_size(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    fn get_board_state(&self) -> (GameState, HashMap<Cell,CellState>) {
        (self.game_state, self.board_play_state.clone())
    }

    fn play_move(&mut self, move_type: MoveType, cell: Cell) -> Result<()> {
        if cell.x > self.height || cell.y > self.width {
            Err(ErrorKind::new(
                io::ErrorKind::Other,
                "Move location is out of range",
            ))?
        }

        if self.bomb_count >= self.total_cells {
            Err(ErrorKind::new(io::ErrorKind::Other, "Too many bombs! You can have a maximum of 1 bomb less than total cells"))?;
        }

        if !self.board_initialised {
            self.initialise_board(cell)?;
        }

        match move_type {
            MoveType::Dig => match self.board_play_state[&cell] {
                Unchecked => {
                    self.board_play_state.insert(cell,self.board_state[&cell]);
                    match self.board_play_state[&cell] {
                        Bomb => self.game_state = Complete(Lose),
                        Checked(bombs) => {
                            if bombs == Zero {
                                self.reveal_safe_patch(cell).expect("");
                            }
                            self.checked_cells += 1;
                            if self.checked_cells + self.flagged_cells == self.total_cells {
                                self.game_state = Complete(Win);
                            } else {
                                self.game_state = Playing;
                            }
                        }
                        _ => {}
                    }
                }
                Flagged => {
                    self.board_play_state.insert(cell, Unchecked);
                    self.flagged_cells -= 1;
                }
                _ => {}
            },
            MoveType::Flag => match self.board_play_state[&cell] {
                Unchecked => {
                    self.board_play_state.insert(cell,Flagged);
                    self.flagged_cells += 1;
                }
                Flagged => {
                    self.board_play_state.insert(cell,Unchecked);
                    self.flagged_cells -= 1;
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(Engine::new(self.width, self.height, self.bomb_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
