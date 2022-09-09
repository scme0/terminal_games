use std::collections::{HashMap, HashSet};
use crate::{MouseAction, Component, UpdateElement};
use crossterm::{style::Color, Result};
use log::info;
use minesweeper_engine::{CanBeEngine, Cell, CellState, CompleteState, Engine, GameState, GameStats, MoveType};
use uuid::Uuid;
use minesweeper_engine::AdjacentBombs::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};
use minesweeper_engine::CellState::{Bomb, Checked, Flagged, Unchecked};
use crate::screen::{ClickAction, Dimension, Point};
use std::env::current_exe;
use std::fs;
use std::fs::{File, write};
use serde::{Deserialize, Serialize};
use minesweeper_engine::GameState::{Complete,Playing,Initialised};
use std::path::Path;
use crate::screen::ClickAction::Refresh;

const VISUAL_TEST: bool = false;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum GameType {
    Easy,
    Medium,
    Hard,
}

pub struct GameComponent {
    id: Uuid,
    engine: Box<dyn CanBeEngine>,
    engine_size: Dimension,
    game_type: GameType,
    top_score_data: TopScore,
    retry_button_location: Vec<Point>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TopScore {
    scores: HashMap<GameType,u64>
}

fn convert_to_wide_char(c: char) -> char {
    char::from_u32(c as u32 + 0xFEE0).unwrap()
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
        let engine_size: Dimension =  engine.get_size().into();
        GameComponent { id: Uuid::new_v4(), engine, engine_size, game_type, top_score_data: TopScore{scores:HashMap::new()}, retry_button_location: vec![] }
    }

    fn reset(&mut self) {
        self.engine = self.engine.make_clone();
    }

    fn convert_engine_size_to_size(&self) -> Dimension {
        let engine_size = self.engine_size.clone();
        let y_offset = if let Complete(_) = self.engine.get_board_state().0.game_state {
            4
        } else {
            2
        };
        (engine_size.width * 2, engine_size.height + y_offset).into()
    }

    fn push_stat_char(updates: &mut Vec<UpdateElement>, stat_line_points: &mut HashSet<Point>, point: Point, value: char) {
        updates.push(UpdateElement {point, value, fg: None});
        stat_line_points.remove(&point);
    }

    pub fn get_stats_board_updates(&mut self, game_stats: GameStats, prior_updates: &mut Vec<UpdateElement>) -> Result<()> {
        let size = self.get_size();
        let mut stat_line_points = HashSet::new();
        for x in 0..size.width {
            // draw separator
            prior_updates.push(UpdateElement {point: (x,1).into(), value: '━', fg: None});
            stat_line_points.insert((x,0).into());
        }

        let emoji =         match game_stats.game_state {
            Initialised => '😶',
            Playing => '😊',
            Complete(result) =>
                match result {
                    CompleteState::Win => '🥳',
                    CompleteState::Lose => '😵'
                }
        };
        let emoji_point = ((size.width/2)-1,0).into();
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
        GameComponent::push_stat_char(prior_updates, &mut stat_line_points, clock_point,  '🕑');//⏱
        //GameComponent::push_stat_char(prior_updates, &mut stat_line_points, clock_point + (1,0).into(),  ' ');

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

    fn append_complete_menu_updates(&mut self, game_stats: GameStats, updates: &mut Vec<UpdateElement>) -> Result<()> {
        let size = self.get_size();
        for x in 0..size.width {
            // draw separator
            updates.push(UpdateElement {point: (x,size.height - 2).into(), value: '━', fg: None});
        }
        let halfway_point = ((size.width/2)-1, size.height - 1).into();
        updates.push(UpdateElement{point: halfway_point, value: '┃', fg: None});
        let trophy_point = (((halfway_point.x / 2) - 1 - 4),halfway_point.y).into();
        updates.push(UpdateElement{point: trophy_point, value: '🏆', fg: None});

        let mut score = match self.top_score_data.scores.get(&self.game_type) {
            None => {
                if let Complete(res) = game_stats.game_state {
                    if res == CompleteState::Win {
                        game_stats.game_run_time
                    } else {
                        0
                    }
                } else {
                    0
                }
            },
            Some(result) => result.to_owned(),
        };
        if score > 999 {
            score = 999;
        }
        let score_string = format!("{:03}", score);
        for (i,char) in score_string.chars().enumerate() {
            updates.push(UpdateElement{ point: trophy_point + (((i+1) as i32)*2,0).into(), value:  convert_to_wide_char(char), fg: None});
        }

        let retry_point: Point = ((((halfway_point.x / 2) - 1) + halfway_point.x),halfway_point.y).into();
        // info!("retry_point: {:?}", retry_point);
        for (i, char) in "Retry?".chars().enumerate() {
            updates.push(UpdateElement{ point: retry_point + (i as i32,0).into(), value: char, fg: None});
        }
        for i in halfway_point.x-1..size.width {
            self.retry_button_location.push((i,halfway_point.y).into());
        }
        Ok(())
    }

    fn load_best_score(&mut self, current_score: u64) {
        let mut path = current_exe().unwrap();
        path.pop();
        path.push("top_score.yaml");
        if !Path::new(&path.clone()).exists()
        {
            write(&path.clone(), serde_yaml::to_string(&self.top_score_data).expect("")).expect("")
        }
        let file = File::open(path.clone());
        let mut top_score_data: TopScore =  if let Ok(f) = file {
            serde_yaml::from_reader(f).expect("")
        } else {
            self.top_score_data.clone()
        };

        let top_score_for_game_type = match top_score_data.scores.get(&self.game_type) {
            None => {
                if current_score != u64::MAX {
                    current_score
                } else {
                    0
                }
            },
            Some(result) => {
                if current_score < result.to_owned() {
                    current_score
                } else {
                    result.to_owned()
                }
            }
        };
        top_score_data.scores.insert(self.game_type, top_score_for_game_type);
        self.top_score_data = top_score_data;
        write(&path.clone(), serde_yaml::to_string(&self.top_score_data).expect("")).expect("");
    }
}

impl Component for GameComponent {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_size(&self) -> Dimension {
        self.convert_engine_size_to_size()
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
        let mut updates = vec![];
        let (game_stats, game_updates) = self.engine.get_board_state();
        self.get_stats_board_updates(game_stats, &mut updates)?;
        self.append_updates_from_engine(&game_updates, &mut updates)?;
        if let Complete(_) = game_stats.game_state {
            self.append_complete_menu_updates(game_stats, &mut updates)?;
        } else{
            self.retry_button_location.clear();
        }
        Ok(updates)
    }

    fn handle_click(&mut self, click: MouseAction) -> Result<Vec<ClickAction>> {
        let mut click_actions = vec![];
        let (move_type, (mut x, mut y)) = match click {
            MouseAction::DownMiddle(p) => (Some(MoveType::Flag), p.into()),
            MouseAction::DownRight(p) => (Some(MoveType::Flag), p.into()),
            MouseAction::DownLeft(p) => {
                if self.retry_button_location.contains(&click.to_point()) {
                    self.reset();
                    click_actions.push(Refresh);
                    (None, (0,0))
                } else {
                    (Some(MoveType::Dig), p.into())
                }
            },
            MouseAction::DoubleLeft(p) => { (Some(MoveType::DigAround), p.into())},
            _ => (None, (0,0))
        };
        if let Some(mov) = move_type {
            if x % 2 == 1 {
                x -= 1;
            }
            x /= 2;
            y -= 2;

            if x >= 0 && x < self.engine_size.width && y >= 0 && y < self.engine_size.height {
                if let Complete(result) = self.engine.play_move(mov, Cell {x, y})? {
                    let score = if result == CompleteState::Win {
                        self.engine.get_board_state().0.game_run_time
                    } else {
                        u64::MAX
                    };
                    self.load_best_score(score);
                }
            }
        }
        Ok(click_actions)
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
        return (GameStats{game_state: Playing, flags_remaining:33, game_run_time: 999}, map);
    }

    fn play_move(&mut self, _: MoveType, _: Cell) -> Result<GameState> {
        self.updated = true;
        Ok(Playing)
    }

    fn make_clone(&self) -> Box<dyn CanBeEngine> {
        Box::from(TestEngine::new())
    }
}
