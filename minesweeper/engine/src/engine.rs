use std::collections::{HashMap, HashSet};
use std::time::Instant;
use crossterm::{ErrorKind};
use std::io;
use rand::Rng;
use queues::{IsQueue, queue, Queue};
use crate::{Bomb, CanBeEngine, Cell, CellState, Checked, Complete, Cross, Dig, DigAround, Exploded, Flag, Flagged, GameState, GameStats, Lose, MoveType, Playing, Unchecked, Win, Zero, ZeroToEight};

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

    pub fn is_game_won(&self) -> bool {
        return self.checked_cells + self.flagged_cells == self.total_cells &&
            self.flagged_cells == self.bomb_count;
    }

    pub fn win_game(&mut self) {
        if let Some(start_instant) = self.start_instant{
            self.game_complete_time = start_instant.elapsed().as_secs();
        }
        self.game_state = Complete(Win);
    }

    pub fn lose_game(&mut self) {
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

    fn increment_bomb_count_of_surrounding_cells(&mut self, cell: Cell) -> crossterm::Result<()> {
        self.get_surrounding_cells(cell, Some(|s, c| {
            if let Checked(bombs) = s.board_state[&c] {
                let mut bombs_as_byte = bombs as u8;
                bombs_as_byte += 1;
                let new_bombs = ZeroToEight::from_u8(bombs_as_byte).expect("");
                s.board_state.insert(c, Checked(new_bombs));
            }
        }));
        Ok(())
    }

    fn initialise_board(&mut self, clicked_cell: Cell) -> crossterm::Result<()> {
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

    fn reveal_safe_patch(&mut self, starting_cell: Cell) -> crossterm::Result<()> {
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

    fn dig_around_cell(&mut self, cell: Cell) -> GameState {
        if let Checked(adjacent_bombs) = self.board_play_state[&cell] {
            let surrounding_cells = self.get_surrounding_cells(cell, None);
            let num_flagged_surrounding_cells = surrounding_cells.iter().filter(|c|self.board_play_state[c] == Flagged).count();
            if adjacent_bombs.to_usize() == num_flagged_surrounding_cells {
                for c in surrounding_cells {
                    if let Complete(state) = self.dig_cell(c, false) {
                        return Complete(state);
                    }
                }
            }
        }
        return Playing;
    }

    fn flag_cell(&mut self, cell: Cell) -> GameState {
        match self.board_play_state[&cell] {
            Unchecked => {
                self.board_play_state.insert(cell,Flagged);
                self.flagged_cells += 1;
                if self.is_game_won() {
                    self.win_game();
                    return Complete(Win);
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
        return Playing;
    }

    fn dig_cell(&mut self, cell: Cell, also_unflag: bool) -> GameState {
        match self.board_play_state[&cell] {
            Unchecked => {
                self.board_play_state.insert(cell, self.board_state[&cell]);
                match self.board_play_state[&cell] {
                    Bomb => {
                        self.board_play_state.insert(cell, Exploded);
                        self.lose_game();
                        return Complete(Lose);
                    },
                    Checked(bombs) => {
                        if bombs == Zero {
                            self.reveal_safe_patch(cell).expect("");
                        }
                        self.checked_cells += 1;
                        if self.is_game_won() {
                            self.win_game();
                            return Complete(Win);
                        } else {
                            self.game_state = Playing;
                        }
                    }
                    _ => {}
                }
            }
            Flagged => {
                if also_unflag {
                    self.board_play_state.insert(cell, Unchecked);
                    self.flagged_cells -= 1;
                }
            }
            _ => {}
        }
        return Playing;
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

    fn play_move(&mut self, move_type: MoveType, cell: Cell) -> crossterm::Result<GameState> {
        if let Complete(state) = self.game_state {
            return Ok(Complete(state));
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

        let game_state = match move_type {
            Dig => {
                self.dig_cell(cell, true)
            },
            Flag => {
                self.flag_cell(cell)
            },
            DigAround => {
                self.dig_around_cell(cell)
            }
        };

        Ok(game_state)
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(Engine::new(self.width, self.height, self.bomb_count))
    }

    fn get_chill_factor(&mut self, cell: Cell) -> crossterm::Result<ZeroToEight> {
        let mut cell_states: Vec<CellState> = vec![self.board_play_state[&cell]].clone();
        let other_cells = self.get_surrounding_cells(cell, None);
        for cell in other_cells.iter() {
            cell_states.push(self.board_play_state[cell]);
        }
        let mut least_chill_value:u8 = 0;
        for state in cell_states {
            let value =
                match state {
                    Unchecked => 0,
                    Checked(count) => {
                        count.to_usize() as u8
                    }
                    Flagged => 0,
                    Bomb => 8,
                    Cross => 0,
                    Exploded => 8
                };
            if value > least_chill_value {
                least_chill_value = value;
            }
        }
        ZeroToEight::from_u8(least_chill_value)
    }
}
