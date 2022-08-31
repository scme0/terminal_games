use crate::game::GameType;
use crate::{ClickType, Component, UpdateElement};
use crossterm::{style::Color, Result};
use log::info;
use minesweeper_engine::{AdjacentBombs, CellState, Engine, MoveType};
use std::f32::consts::E;

#[derive(Debug, Clone)]
pub struct GameComponent {
    engine: Option<Engine>,
}

impl GameComponent {
    pub fn new() -> GameComponent {
        GameComponent { engine: None }
    }

    pub fn start(&mut self, game_type: GameType) {
        let engine = match game_type {
            GameType::Easy => Engine::new(10, 8, 10), //10,8,10
            GameType::Medium => Engine::new(18, 14, 40),
            GameType::Hard => Engine::new(24, 20, 99),
        };
        self.engine = Some(engine);
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

            engine.play_move(move_type, x, y)?
        }
        Ok(())
    }
}

impl Component for GameComponent {
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
            for x in engine.get_board_state().1.iter().enumerate() {
                for y in x.1.iter().enumerate() {
                    let (value, bg) = match y.1 {
                        CellState::Unchecked => ('🟩',Color::Black),
                        CellState::Checked(adjacent_bombs) => match adjacent_bombs {
                            AdjacentBombs::Zero => ('🟫', Color::Black),
                            AdjacentBombs::One => ('１', Color::Grey),
                            AdjacentBombs::Two => ('２', Color::Green),
                            AdjacentBombs::Three => ('３', Color::Yellow),
                            AdjacentBombs::Four => ('４', Color::Cyan),
                            AdjacentBombs::Five => ('５', Color::Blue),
                            AdjacentBombs::Six => ('６', Color::Rgb {r:138,g:43,b:226}),
                            AdjacentBombs::Seven => ('７', Color::Red),
                            AdjacentBombs::Eight => ('８', Color::Magenta),
                        },
                        CellState::Flagged => ('🚩', Color::White),
                        CellState::Bomb => ('💣', Color::Red),
                    };

                    updates.push(UpdateElement {
                        x: x.0,
                        y: y.0*2,
                        value,
                        bg,
                        fg: Color::DarkGrey,
                    });
                }
            }
        }
        return updates;
    }
}
