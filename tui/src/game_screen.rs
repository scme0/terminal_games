use std::collections::{HashMap, HashSet};
use crate::{MouseAction, Component, UpdateElement};
use crossterm::{style::Color, Result};
use log::info;
use minesweeper_engine::{CanBeEngine, Cell, CellState, CompleteState, Engine, GameState, GameStats, MoveType};
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
    engine: Box<dyn CanBeEngine>,
    width: i32,
    height: i32
}

impl GameComponent {
    pub fn new(game_type: GameType) -> GameComponent {
        let mut engine: Box<dyn CanBeEngine> = match VISUAL_TEST {
            true => Box::from(TestEngine::new()),
            false => Box::from(match game_type {
                GameType::Easy => Engine::new(11, 8, 12),
                GameType::Medium => Engine::new(19, 14, 45),
                GameType::Hard => Engine::new(25, 20, 100),
            })
        };

        let (e_width, e_height) =  engine.get_size();
        GameComponent { id: Uuid::new_v4(), engine, width: e_width*2, height: e_height + 2 }
    }

    fn push_stat_char(updates: &mut Vec<UpdateElement>, stat_line_points: &mut HashSet<Point>, point: Point, value: char) {
        updates.push(UpdateElement {point, value, fg: None});
        stat_line_points.remove(&point);
    }

    pub fn get_stats_board_updates(&mut self, game_stats: GameStats, prior_updates: &mut Vec<UpdateElement>) -> Result<()> {
        let mut stat_line_points = HashSet::new();
        for x in 0..self.width {
            // draw separator
            prior_updates.push(UpdateElement {point: (x,1).into(), value: '━', fg: None});
            stat_line_points.insert((x,0).into());
        }

        let emoji =         match game_stats.game_state {
            GameState::Initialised => '😶',
            GameState::Playing => '😊',
            GameState::Complete(result) =>
                match result {
                    CompleteState::Win => '🥳',
                    CompleteState::Lose => '😵'
                }
        };
        let emoji_point = ((self.width/2)-1,0).into();
        GameComponent::push_stat_char(prior_updates, &mut stat_line_points, emoji_point, emoji);

        // draw flag count
        let flag_point = (((emoji_point.x / 2) - 1 - 4),emoji_point.y).into();
        // info!("flag_point: {:?}", flag_point);
        let mut flags = game_stats.flags_remaining;
        if flags > 999 {
            flags = 999;
        }
        let flag_string = format!("{:03}", flags);
        GameComponent::push_stat_char(prior_updates, &mut stat_line_points, flag_point,  '🚩');
        // push_stat_char(prior_updates, &mut stat_line_points, flag_point + (1,0).into(),  char::default());
        fn convert_to_wide_char(c: char) -> char {
            char::from_u32(c as u32 + 0xFEE0).unwrap()
        }
        for (i,char) in flag_string.chars().enumerate() {
            GameComponent::push_stat_char(prior_updates, &mut stat_line_points, flag_point + (((i+1) as i32)*2,0).into(), convert_to_wide_char(char));
            // push_stat_char(prior_updates, &mut stat_line_points, flag_point + (((i+1) as i32)*2+1,0).into(), char::default());
        }

        // draw clock
        let clock_point = ((((emoji_point.x / 2) - 1) + emoji_point.x),emoji_point.y).into();
        // info!("clock_point: {:?}", clock_point);
        let mut seconds = game_stats.game_run_time;
        if seconds > 999 {
            seconds = 999;
        }
        let seconds_string = format!("{:03}", seconds);
        GameComponent::push_stat_char(prior_updates, &mut stat_line_points, clock_point,  '⏱');
        GameComponent::push_stat_char(prior_updates, &mut stat_line_points, clock_point + (1,0).into(),  ' ');

        for (i,char) in seconds_string.chars().enumerate() {
            GameComponent::push_stat_char(prior_updates, &mut stat_line_points, clock_point + (((i+1) as i32)*2,0).into(),  convert_to_wide_char(char));
            // push_stat_char(prior_updates, &mut stat_line_points, clock_point + (((i+1) as i32)*2+1,0).into(),  char::default());
        }

        for left_over_point in stat_line_points.iter(){
            prior_updates.push(UpdateElement {point: *left_over_point, value: char::default(), fg: None});
        }

        Ok(())
    }

    pub fn append_updates_from_engine(&mut self, game_updates: &HashMap<Cell, CellState>, prior_updates: &mut Vec<UpdateElement>) -> Result<()>{
        for (cell, cell_state) in game_updates.iter() {
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
                CellState::Cross => ('❌', Color::White),
                CellState::Exploded => ('💥', Color::White)
            };

            prior_updates.push(UpdateElement {
                point: (cell.x * 2, cell.y + 2).into(),
                value,
                fg: Some(fg),
            });

            prior_updates.push(UpdateElement {
                point: (cell.x * 2 + 1, cell.y + 2).into(),
                value: char::default(),
                fg: None
            });
        }
        Ok(())
    }
}

impl Component for GameComponent {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
        let mut updates = vec![];
        let (game_stats, game_updates) = self.engine.get_board_state();
        self.get_stats_board_updates(game_stats, &mut updates)?;
        self.append_updates_from_engine(&game_updates, &mut updates)?;
        Ok(updates)
    }

    fn handle_click(&mut self, click: MouseAction) -> Result<Vec<ClickAction>> {
        let (move_type, (mut x, mut y)) = match click {
            MouseAction::DownMiddle(p) => (Some(MoveType::Flag), p.into()),
            MouseAction::DownRight(p) => (Some(MoveType::Flag), p.into()),
            MouseAction::DownLeft(p) => (Some(MoveType::Dig), p.into()),
            _ => (None, (0,0))
        };
        if let Some(mov) = move_type {
            if x % 2 == 1 {
                x -= 1;
            }
            x /= 2;
            y -= 2;

            if x >= 0 && y >= 0 {
                self.engine.play_move(mov, Cell {x, y})?;
            }
        }
        Ok(vec![])
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
    fn get_size(&self) -> (i32, i32) {
        return (4,3);
    }

    fn get_board_state(&self) -> (GameStats, HashMap<Cell, CellState>) {
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
        return (GameStats{game_state: GameState::Playing, flags_remaining:33, game_run_time: 999}, map);
    }

    fn play_move(&mut self, _: MoveType, _: Cell) -> Result<()> {
        self.updated = true;
        Ok(())
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(TestEngine::new())
    }
}
