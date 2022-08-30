use crate::CellState::{Bomb, Checked, Flagged, Unchecked};
use crate::CompleteState::{Lose, Win};
use crate::GameState::{Complete, Playing};
use crossterm::{execute, terminal, ErrorKind, Result};
use rand::Rng;
use std::io;
use log::info;

#[derive(Debug, Copy, Clone)]
pub enum AdjacentBombs {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

#[derive(Debug, Copy, Clone)]
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
    board_bombs: Vec<Vec<bool>>,
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
        let mut local_bomb_count = bomb_count;
        let mut rng = rand::thread_rng();
        let bomb_probability = bomb_count as f64 / total_cells as f64;

        let mut get_bomb_or_not = || rng.gen_range(0.0..1.0) <= bomb_probability;

        let mut board_play_state = vec![vec![Unchecked; width]; height];
        let mut board_bombs = vec![vec![false; width]; height];
        while local_bomb_count > 0 {
            for x in 0..height {
                for y in 0..width {
                    if board_bombs[x][y] {
                        continue;
                    }
                    board_bombs[x][y] = get_bomb_or_not();
                    if board_bombs[x][y] {
                        local_bomb_count -= 1;
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
        info!("Bombs: {:?}", board_bombs);
        info!("Engine: w: {}, h: {}, b: {}, t: {}, lb: {}", width, height, bomb_count, total_cells, local_bomb_count);
        Engine {
            board_bombs,
            board_play_state,
            game_state: GameState::Initialised,
            width,
            height,
            bomb_count,
            checked_cells: 0,
            flagged_cells: 0,
            total_cells,
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

        match move_type {
            MoveType::Dig => match self.board_play_state[x][y] {
                Unchecked => {
                    if self.board_bombs[x][y] {
                        self.board_play_state[x][y] = Bomb;
                        self.game_state = Complete(Lose);
                    } else {
                        info!("clicked on: x: {}, y: {}", x ,y);
                        let clear_cells = self.get_coords_of_surrounding_clear_cells(x,y);
                        let bomb_count = 8 - clear_cells.len();
                        //let mut bomb_counts = vec![(bomb_count, x, y)];
                        // if bomb_count == 0 {
                        //     for (x_s, y_s) in clear_cells {
                        //         let clear_cells_for_surr = self.get_coords_of_surrounding_clear_cells(x_s,y_s);
                        //         if clear_cells_for_surr.len() == 8
                        //     }
                        // }
                        let adjacent_bombs = match bomb_count {
                            0 => AdjacentBombs::Zero,
                            1 => AdjacentBombs::One,
                            2 => AdjacentBombs::Two,
                            3 => AdjacentBombs::Three,
                            4 => AdjacentBombs::Four,
                            5 => AdjacentBombs::Five,
                            6 => AdjacentBombs::Six,
                            7 => AdjacentBombs::Seven,
                            8 => AdjacentBombs::Eight,
                            _ => Err(ErrorKind::new(
                                io::ErrorKind::Other,
                                "Invalid number of adjacent bombs!",
                            ))?,
                        };
                        self.board_play_state[x][y] = Checked(adjacent_bombs);
                        self.checked_cells += 1;
                        if self.checked_cells + self.flagged_cells == self.total_cells {
                            self.game_state = Complete(Win);
                        } else {
                            self.game_state = Playing;
                        }
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

    fn get_coords_of_surrounding_clear_cells(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut clear_cells = vec![];
        // let mut bomb_count = 0;
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

                if self.board_bombs[x_s as usize][y_s as usize] {
                    info!("bomb found at: x: {}, y: {}", x_s, y_s);
                    clear_cells.push((x_s as usize, y_s as usize));
                    // bomb_count += 1;
                } else {
                    info!("no bomb found at: x: {}, y: {}", x_s, y_s);
                }
            }
        }
        // if bomb_count > 8 {
        //     bomb_count = 8;
        // }
        return clear_cells;
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
