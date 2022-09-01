use std::collections::HashMap;
use crate::game::GameType;
use crate::{ClickType, Component, UpdateElement};
use crossterm::{style::Color, Result};
use log::info;
use minesweeper_engine::{AdjacentBombs, CanBeEngine, Cell, CellState, Engine, GameState, MoveType};
use std::f32::consts::E;
use uuid::Uuid;
use minesweeper_engine::AdjacentBombs::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};
use minesweeper_engine::CellState::{Bomb, Checked, Flagged, Unchecked};
use crate::screen::window::ComponentType;

const VISUAL_TEST: bool = false;

struct ClonedEngine {
    size: (usize, usize)
}

impl ClonedEngine {
    fn new(engine: &Box<dyn CanBeEngine>) -> Self {
        ClonedEngine { size: engine.get_size() }
    }
}

impl CanBeEngine for ClonedEngine {
    fn get_size(&self) -> (usize, usize) {
        self.size
    }

    fn get_board_state(&self) -> (GameState, HashMap<Cell, CellState>) {
        (GameState::Playing, HashMap::new())
    }

    fn play_move(&mut self, _: MoveType, _: Cell) -> Result<()> {
        Ok(())
    }
}

pub struct GameComponent {
    id: Uuid,
    engine: Option<Box<dyn CanBeEngine>>,
}

impl Clone for GameComponent {
    fn clone(&self) -> Self {
        let engine : Option<Box<dyn CanBeEngine>> = match &self.engine {
            None => None,
            Some(e) => Some(Box::from(ClonedEngine::new(e)))
        };
        GameComponent {id: self.id, engine }
    }
}

impl GameComponent {
    pub fn new() -> GameComponent {
        GameComponent { engine: None, id: Uuid::new_v4() }
    }

    pub fn start(&mut self, game_type: GameType) {
        if VISUAL_TEST {
            self.engine = Some(Box::from(TestEngine::new()));
        }else {
            let engine = match game_type {
                GameType::Easy => Engine::new(10, 8, 10),
                GameType::Medium => Engine::new(18, 14, 40),
                GameType::Hard => Engine::new(24, 20, 99),
            };
            self.engine = Some(Box::from(engine));
        }
    }

    pub fn click(&mut self, click_type: ClickType) -> Result<()> {
        if let Some(engine) = &mut self.engine {
            let (move_type, x, mut y) = match click_type {
                ClickType::Middle(x, y) => (MoveType::Flag, x, y),
                ClickType::Left(x, y) => (MoveType::Dig, x, y),
            };
            if y % 2 == 1 {
                y -= 1;
            }
            y /= 2;

            engine.play_move(move_type, Cell {x, y})?
        }
        Ok(())
    }
}

impl Component for GameComponent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn size(&self) -> (usize, usize) {
        let size = match &self.engine {
            None => (0, 0),
            Some(engine) => engine.get_size(),
        };
        (size.0, size.1 * 2)
    }

    fn update(&self) -> Vec<UpdateElement> {
        let mut updates = vec![];
        if let Some(engine) = &self.engine {
            for (cell, cellState) in engine.get_board_state().1.iter() {
                let (value, fg) = match cellState {
                    CellState::Unchecked => ('🟩', Color::White),
                    CellState::Checked(adjacent_bombs) => match adjacent_bombs {
                        AdjacentBombs::Zero => ('🟫', Color::White),
                        AdjacentBombs::One => ('１', Color::White),
                        AdjacentBombs::Two => ('２', Color::Cyan),
                        AdjacentBombs::Three => ('３', Color::Green),
                        AdjacentBombs::Four => ('４', Color::Yellow),
                        AdjacentBombs::Five => ('５', Color::DarkYellow),
                        AdjacentBombs::Six => ('６', Color::DarkMagenta),
                        AdjacentBombs::Seven => ('７', Color::Red),
                        AdjacentBombs::Eight => ('８', Color::DarkRed),
                    },
                    CellState::Flagged => ('🚩', Color::White),
                    CellState::Bomb => ('💣', Color::White),
                };

                updates.push(UpdateElement {
                    x: cell.x,
                    y: cell.y * 2,
                    value,
                    fg,
                });
            }
        }
        return updates;
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::GameScreen
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
            map.insert(Cell{x: 0, y: 0}, Checked(Zero));
            map.insert(Cell{x: 0, y: 1}, Checked(One));
            map.insert(Cell{x: 0, y: 2}, Checked(Two));
            map.insert(Cell{x: 1, y: 0}, Checked(Three));
            map.insert(Cell{x: 1, y: 1}, Checked(Four));
            map.insert(Cell{x: 1, y: 2}, Checked(Five));
            map.insert(Cell{x: 2, y: 0}, Checked(Six));
            map.insert(Cell{x: 2, y: 1}, Checked(Seven));
            map.insert(Cell{x: 2, y: 2}, Checked(Eight));
            map.insert(Cell{x: 3, y: 0}, Unchecked);
            map.insert(Cell{x: 3, y: 1}, Flagged);
            map.insert(Cell{x: 3, y: 2}, Bomb);
        }
        return (GameState::Playing, map);
    }

    fn play_move(&mut self, _: MoveType, _: Cell) -> Result<()> {
        self.updated = true;
        Ok(())
    }
}
