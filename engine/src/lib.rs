use std::collections::{HashMap, HashSet};
use crate::CellState::{Bomb, Checked, Cross, Exploded, Flagged, Unchecked};
use crate::CompleteState::{Lose, Win};
use crate::GameState::{Complete, Playing};
use crossterm::{ErrorKind, Result};
use rand::Rng;
use std::io;
use std::time::Instant;
use log::info;
use queues::{IsQueue, Queue, queue};
use crate::AdjacentBombs::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};
use crate::MoveType::{Flag, Dig, DigAround};

pub trait CanBeEngine {
    fn get_size(&self) -> (i32, i32);
    fn get_board_state(&self) -> (GameStats, HashMap<Cell,CellState>);
    fn play_move(&mut self, move_type: MoveType, cell: Cell) -> Result<()>;
    fn make_clone(&self) -> Box<dyn CanBeEngine>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cell {
    pub x: i32,
    pub y: i32
}

impl From<Cell> for (i32, i32) {
    fn from(c: Cell) -> (i32, i32) {
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

    pub fn to_usize(&self) -> usize {
        match self {
            Zero => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CellState {
    Unchecked,
    Checked(AdjacentBombs),
    Flagged,
    Bomb,
    Cross,
    Exploded
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

pub struct GameStats {
    pub game_state: GameState,
    pub flags_remaining: i32,
    pub game_run_time: u64,
}

#[derive(Debug, Clone)]
pub struct Engine {
    game_state: GameState,
    board_play_state: HashMap<Cell,CellState>,
    board_state: HashMap<Cell,CellState>,
    board_initialised: bool,
    width: i32,
    height: i32,
    bomb_count: i32,
    checked_cells: i32,
    flagged_cells: i32,
    total_cells: i32,
    start_instant: Option<Instant>,
    game_complete_time: u64
}

#[derive(Debug, Clone)]
pub enum MoveType {
    DigAround,
    Dig,
    Flag,
}

impl Engine {
    pub fn new(width: i32, height: i32, bomb_count: i32) -> Self {
        let total_cells = width * height;
        let mut board_play_state = HashMap::new();
        let mut board_state = HashMap::new();
        for x in 0..width {
            for y in 0..height {
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
            board_initialised: false,
            start_instant: None,
            game_complete_time: 0,
        }
    }

    pub(crate) fn win_game(&mut self) {
        if let Some(start_instant) = self.start_instant{
            self.game_complete_time = start_instant.elapsed().as_secs();
        }
        self.game_state = Complete(Win);
    }

    pub(crate) fn lose_game(&mut self) {
        if let Some(start_instant) = self.start_instant{
            self.game_complete_time = start_instant.elapsed().as_secs();
        }
        self.game_state = Complete(Lose);
        for x in 0..self.width {
            for y in 0..self.height {
                let cell = Cell{x,y};
                if let Some(cell_state) = self.board_state.get(&cell) {
                    if let Some(p_cell_state) = self.board_play_state.get(&cell) {
                        if *cell_state == Bomb {
                            if *p_cell_state != Flagged && *p_cell_state != Exploded {
                                self.board_play_state.insert(cell, Bomb);
                            }
                        } else {
                            if *p_cell_state == Flagged {
                                self.board_play_state.insert(cell, Cross);
                            }
                        }
                    }
                }
            }
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

        let mut safe_cells = self.get_surrounding_cells(clicked_cell, None);
        safe_cells.push(clicked_cell);

        let mut rng = rand::thread_rng();
        let bomb_probability = self.bomb_count as f64 / self.total_cells as f64;
        let mut get_bomb_or_not = || rng.gen_range(0.0..1.0) <= bomb_probability;
        let mut local_bomb_count = self.bomb_count;
        while local_bomb_count > 0 {
            for x in 0..self.width {
                for y in 0..self.height {
                    let cell = Cell {x, y};
                    // Ensure no bomb is placed on or around the clicked cell!
                    if  safe_cells.contains(&cell) {
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
                if self.board_play_state[&cell] == Unchecked {
                    self.board_play_state.insert(cell, self.board_state[&cell]);
                    self.checked_cells += 1;
                }
            }
        }
        Ok(())
    }

    fn get_surrounding_cells(&mut self, cell: Cell, func: Option<fn (engine: &mut Engine,cell: Cell)>) -> Vec<Cell> {
        let (x, y) = cell.into();
        let mut cells = vec![];
        for x_s in (x - 1)..(x + 2) {
            if x_s < 0 || x_s >= self.width {
                continue;
            }
            for y_s in (y - 1)..(y + 2) {
                if y_s < 0 || y_s >= self.height {
                    continue;
                }
                if y_s == y && x_s == x {
                    continue;
                }

                let cell = Cell {x: x_s, y: y_s};
                if let Some(f) = func {
                    f(self, cell);
                }

                cells.push(cell);
            }
        }
        return cells;
    }

    fn dig_around_cell(&mut self, cell: Cell) {
        if let Checked(adjacentBombs) = self.board_play_state[&cell] {
            let surrounding_cells = self.get_surrounding_cells(cell, None);
            let num_flagged_surrounding_cells = surrounding_cells.iter().filter(|c|self.board_play_state[c] == Flagged).count();
            if adjacentBombs.to_usize() == num_flagged_surrounding_cells {
                for c in surrounding_cells {
                    self.dig_cell(c, false);
                }
            }
        }
    }

    fn flag_cell(&mut self, cell: Cell) {
        match self.board_play_state[&cell] {
            Unchecked => {
                self.board_play_state.insert(cell,Flagged);
                self.flagged_cells += 1;
                if self.checked_cells + self.flagged_cells == self.total_cells {
                    self.win_game();
                } else {
                    self.game_state = Playing;
                }
            }
            Flagged => {
                self.board_play_state.insert(cell,Unchecked);
                self.flagged_cells -= 1;
            }
            _ => {}
        }
    }

    fn dig_cell(&mut self, cell: Cell, alsoUnFlag: bool) {
        match self.board_play_state[&cell] {
            Unchecked => {
                self.board_play_state.insert(cell, self.board_state[&cell]);
                match self.board_play_state[&cell] {
                    Bomb => {
                        self.board_play_state.insert(cell, Exploded);
                        self.lose_game();
                    },
                    Checked(bombs) => {
                        if bombs == Zero {
                            self.reveal_safe_patch(cell).expect("");
                        }
                        self.checked_cells += 1;
                        if self.checked_cells + self.flagged_cells == self.total_cells {
                            self.win_game();
                        } else {
                            self.game_state = Playing;
                        }
                    }
                    _ => {}
                }
            }
            Flagged => {
                if alsoUnFlag {
                    self.board_play_state.insert(cell, Unchecked);
                    self.flagged_cells -= 1;
                }
            }
            _ => {}
        }
    }
}

impl CanBeEngine for Engine {
    fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn get_board_state(&self) -> (GameStats, HashMap<Cell,CellState>) {
        let game_time = match self.game_state {
            Complete(_) => self.game_complete_time,
            _ => match self.start_instant{
                None => 0,
                Some(instant) => instant.elapsed().as_secs()
            }
        };
        (GameStats {game_state: self.game_state, flags_remaining: self.bomb_count - self.flagged_cells, game_run_time: game_time}, self.board_play_state.clone())
    }

    fn play_move(&mut self, move_type: MoveType, cell: Cell) -> Result<()> {
        if let Complete(_) = self.game_state {
            return Ok(());
        }

        if cell.x > self.width || cell.y > self.height {
            Err(ErrorKind::new(
                io::ErrorKind::Other,
                "Move location is out of range",
            ))?
        }

        if self.bomb_count >= self.total_cells {
            Err(ErrorKind::new(io::ErrorKind::Other, "Too many bombs! You can have a maximum of 1 bomb less than total cells"))?;
        }

        if !self.board_initialised {
            self.start_instant = Some(Instant::now());
            self.initialise_board(cell)?;
        }

        match move_type {
            Dig => {
                self.dig_cell(cell, true);
            },
            Flag => {
                self.flag_cell(cell);
            },
            DigAround => {
                self.dig_around_cell(cell);
            }

        }
        info!("state: {:?}, flagged: {}, checked: {}", self.game_state, self.flagged_cells, self.checked_cells);
        Ok(())
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(Engine::new(self.width, self.height, self.bomb_count))
    }
}
