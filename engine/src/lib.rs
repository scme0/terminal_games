use std::convert::Infallible;
use crate::CellState::{Bomb, Checked, Flagged, Unchecked};
use crate::CompleteState::{Lose, Win};
use crate::GameState::{Complete, Playing};
use crossterm::{execute, terminal, ErrorKind, Result};
use rand::Rng;
use std::io;
use log::info;
use crate::AdjacentBombs::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};

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
    board_play_state: Vec<Vec<CellState>>,
    board_state: Vec<Vec<CellState>>,
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
        let mut board_play_state = vec![vec![Unchecked; width]; height];
        let mut board_state = vec![vec![Checked(Zero); width]; height];
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

    pub fn get_size(&self) -> (usize, usize) {
        (self.height, self.width)
    }

    pub fn get_board_state(&self) -> (GameState, Vec<Vec<CellState>>) {
        // info!("boardState: {:?}", self.board_play_state.clone());
        (self.game_state, self.board_play_state.clone())
    }

    pub fn play_move(&mut self, move_type: MoveType, x: usize, y: usize) -> Result<()> {
        if x > self.height || y > self.width {
            Err(ErrorKind::new(
                io::ErrorKind::Other,
                "Move location is out of range",
            ))?
        }

        if self.bomb_count >= self.total_cells {
            Err(ErrorKind::new(io::ErrorKind::Other, "Too many bombs! You can have a maximum of 1 bomb less than total cells"))?;
        }

        if !self.board_initialised {
            self.initialise_board((x,y))?;
        }

        match move_type {
            MoveType::Dig => match self.board_play_state[x][y] {
                Unchecked => {
                    self.board_play_state[x][y] = self.board_state[x][y];
                    match self.board_play_state[x][y] {
                        Bomb => self.game_state = Complete(Lose),
                        Checked(bombs) => {
                            // if bombs == Zero {
                            //     TODO: Reveal all Zeros and adjacent non-zeros on board
                            // }
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
                    self.board_play_state[x][y] = Unchecked;
                    self.flagged_cells -= 1;
                }
                _ => {}
            },
            MoveType::Flag => match self.board_play_state[x][y] {
                Unchecked => {
                    self.board_play_state[x][y] = Flagged;
                    self.flagged_cells += 1;
                }
                Flagged => {
                    self.board_play_state[x][y] = Unchecked;
                    self.flagged_cells -= 1;
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn increment_bomb_count_of_surrounding_cells(&mut self, x: usize, y: usize) -> Result<()> {
        for x_s in (x as i32 - 1)..(x as i32 + 2) {
            if x_s < 0 || x_s >= self.height as i32 {
                info!("out of range x, skipping: {}",x_s);
                continue;
            }
            for y_s in (y as i32 - 1)..(y as i32 + 2) {
                if y_s < 0 || y_s >= self.width as i32 {
                    info!("out of range y, skipping: {}",y_s);
                    continue;
                }

                if y_s == y as i32 && x_s == x as i32 {
                    info!("don't check clicked spot :)");
                    continue;
                }

                if let Checked(bombs) = self.board_state[x_s as usize][y_s as usize] {
                    let mut bombs_as_byte = bombs as u8;
                    bombs_as_byte += 1;
                    let mut new_bombs = AdjacentBombs::from_u8(bombs_as_byte)?;
                    self.board_state[x_s as usize][y_s as usize] = Checked(new_bombs);
                }
            }
        }
        Ok(())
    }

    fn initialise_board(&mut self, clicked_cell: (usize, usize)) -> Result<()> {
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
                    // Ensure no bomb is placed on the clicked cell!
                    if  clicked_cell == (x, y) {
                        continue;
                    }

                    if self.board_state[x][y] == Bomb {
                        continue;
                    }
                    if get_bomb_or_not() {
                        self.board_state[x][y] = Bomb;
                        local_bomb_count -= 1;
                        self.increment_bomb_count_of_surrounding_cells(x, y)?;
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
        info!("Bombs: {:?}", self.board_state);
        info!("Engine: w: {}, h: {}, b: {}, t: {}, lb: {}", self.width, self.height, self.bomb_count, self.total_cells, local_bomb_count);
        Ok(())
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
