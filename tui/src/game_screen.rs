use std::collections::HashMap;
use crate::{Click, Component, UpdateElement};
use crossterm::{style::Color, Result};
use minesweeper_engine::{CanBeEngine, Cell, CellState, Engine, GameState, MoveType};
use uuid::Uuid;
use minesweeper_engine::AdjacentBombs::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};
use minesweeper_engine::CellState::{Bomb, Checked, Flagged, Unchecked};
use crate::screen::{ClickAction, Point};

const VISUAL_TEST: bool = false;

#[derive(Debug, Copy, Clone)]
pub enum GameType {
    Easy,
    Medium,
    Hard,
}

pub struct GameComponent {
    id: Uuid,
    engine: Option<Box<dyn CanBeEngine>>,
}

impl Clone for GameComponent {
    fn clone(&self) -> Self {
        let engine : Option<Box<dyn CanBeEngine>> = match &self.engine {
            None => None,
            Some(e) => {
                Some(e.make_clone())
            }
        };
        GameComponent {id: self.id, engine }
    }
}

impl GameComponent {
    pub fn new(game_type: GameType) -> GameComponent {
        GameComponent { id: Uuid::new_v4(), engine: match VISUAL_TEST {
            true => Some(Box::from(TestEngine::new())),
            false => Some(Box::from(match game_type {
                GameType::Easy => Engine::new(10, 8, 10),
                GameType::Medium => Engine::new(18, 14, 40),
                GameType::Hard => Engine::new(24, 20, 99),
            }))
        }  }
    }
}

impl Component for GameComponent {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_size(&self) -> (usize, usize) {
        let size = match &self.engine {
            None => (0, 0),
            Some(engine) => engine.get_size(),
        };
        (size.0 * 2, size.1)
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
        let mut updates = vec![];
        if let Some(engine) = &self.engine {
            for (cell, cell_state) in engine.get_board_state().1.iter() {
                let (value, fg) = match cell_state {
                    Unchecked => ('🟩', Color::White),
                    Checked(adjacent_bombs) => match adjacent_bombs {
                        Zero => ('🟫', Color::White),
                        One => ('１', Color::White),
                        Two => ('２', Color::Cyan),
                        Three => ('３', Color::Green),
                        Four => ('４', Color::Yellow),
                        Five => ('５', Color::DarkYellow),
                        Six => ('６', Color::DarkMagenta),
                        Seven => ('７', Color::Red),
                        Eight => ('８', Color::DarkRed),
                    },
                    Flagged => ('🚩', Color::White),
                    Bomb => ('💣', Color::White),
                };

                updates.push(UpdateElement {
                    point: (cell.x * 2, cell.y).into(),
                    value,
                    fg: Some(fg),
                });

                updates.push(UpdateElement {
                    point: (cell.x * 2 + 1, cell.y).into(),
                    value: char::default(),
                    fg: None
                });
            }
        }
        return Ok(updates);
    }

    fn handle_click(&mut self, click: Click) -> Result<ClickAction> {
        if let Some(engine) = &mut self.engine {
            let (move_type, (mut x, y)) = match click {
                Click::Middle(p) => (Some(MoveType::Flag), p.into()),
                Click::Right(p) => (Some(MoveType::Flag), p.into()),
                Click::Left(p) => (Some(MoveType::Dig), p.into()),
            };
            if let Some(mov) = move_type {
                if x % 2 == 1 {
                    x -= 1;
                }
                x /= 2;

                engine.play_move(mov, Cell {x, y})?;
            }
        }
        Ok(ClickAction::None)
    }
}

struct TestEngine {
    updated: bool
}

impl TestEngine {
    fn new() -> Self {
        TestEngine { updated: false }
    }
}

impl CanBeEngine for TestEngine {
    fn get_size(&self) -> (usize, usize) {
        return (4,3);
    }

    fn get_board_state(&self) -> (GameState, HashMap<Cell, CellState>) {
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
        return (GameState::Playing, map);
    }

    fn play_move(&mut self, _: MoveType, _: Cell) -> Result<()> {
        self.updated = true;
        Ok(())
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(TestEngine::new())
    }
}
