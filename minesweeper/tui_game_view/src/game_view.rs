use std::collections::{HashMap, HashSet};
use crossterm::{Result, style::Color};
use log::info;
use minesweeper_engine::engine::Engine;
use uuid::Uuid;
use minesweeper_engine::zero_to_eight::ZeroToEight::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};
use minesweeper_engine::cell_state::CellState::{Bomb, Checked, Flagged, Unchecked};
use std::env::current_exe;
use std::fs::{File, write};
use serde::{Deserialize, Serialize};
use minesweeper_engine::game_state::GameState::{Complete, Initialised, Playing};
use std::path::Path;
use minesweeper_engine::can_be_engine::CanBeEngine;
use minesweeper_engine::cell::Cell;
use minesweeper_engine::cell_state::CellState;
use minesweeper_engine::complete_state::CompleteState;
use minesweeper_engine::game_state::GameState;
use minesweeper_engine::game_stats::GameStats;
use minesweeper_engine::move_type::MoveType;
use minesweeper_engine::zero_to_eight::ZeroToEight;
use tui::screen::{ClickAction, Dimension, GameType, Point};
use tui::screen::ClickAction::Refresh;
use tui::screen::window::{Component, MouseAction, UpdateElement};

const VISUAL_TEST: bool = false;

pub struct GameView {
    id: Uuid,
    engine: Box<dyn CanBeEngine>,
    engine_size: Dimension,
    game_type: GameType,
    top_score_data: TopScore,
    retry_button_location: Vec<Point>,
    chill_factor: ZeroToEight
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TopScore {
    scores: HashMap<GameType,u64>
}

fn convert_to_wide_char(c: char) -> char {
    char::from_u32(c as u32 + 0xFEE0).unwrap()
}

impl GameView {
    pub fn new(game_type: GameType) -> GameView {
        let engine: Box<dyn CanBeEngine> = match VISUAL_TEST {
            true => Box::from(TestEngine::new()),
            false => Box::from(match game_type {
                GameType::Easy => Engine::new(11, 8, 12),
                GameType::Medium => Engine::new(19, 14, 45),
                GameType::Hard => Engine::new(25, 20, 100),
            })
        };
        let engine_size: Dimension =  engine.get_size().into();
        GameView { id: Uuid::new_v4(), engine, engine_size, game_type, top_score_data: TopScore{scores:HashMap::new()}, retry_button_location: vec![], chill_factor: Zero }
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
            Initialised => '🫥',
            Playing => GameView::get_emoji_from_chill_factor(self.chill_factor),
            Complete(result) =>
                match result {
                    CompleteState::Win => '🥳',
                    CompleteState::Lose => '😵'
                }
        };
        let emoji_point = ((size.width/2)-1,0).into();
        GameView::push_stat_char(prior_updates, &mut stat_line_points, emoji_point, emoji);

        // draw flag count
        let flag_point = (((emoji_point.x / 2) - 1 - 4),emoji_point.y).into();
        let mut flags = game_stats.flags_remaining;
        if flags > 999 {
            flags = 999;
        }
        let flag_string = format!("{:03}", flags);
        GameView::push_stat_char(prior_updates, &mut stat_line_points, flag_point, '🚩');
        for (i,char) in flag_string.chars().enumerate() {
            GameView::push_stat_char(prior_updates, &mut stat_line_points, flag_point + (((i+1) as i32)*2, 0).into(), convert_to_wide_char(char));
        }

        // draw clock
        let clock_point = ((((emoji_point.x / 2) - 1) + emoji_point.x),emoji_point.y).into();
        let mut seconds = game_stats.game_run_time;
        if seconds > 999 {
            seconds = 999;
        }
        let seconds_string = format!("{:03}", seconds);
        GameView::push_stat_char(prior_updates, &mut stat_line_points, clock_point, '🕑');

        for (i,char) in seconds_string.chars().enumerate() {
            GameView::push_stat_char(prior_updates, &mut stat_line_points, clock_point + (((i+1) as i32)*2, 0).into(), convert_to_wide_char(char));
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
            Some(result) => {
                if result.to_owned() != u64::MAX{
                    result.to_owned()
                } else{
                    0
                }
            },
        };
        if score > 999 {
            score = 999;
        }
        let score_string = format!("{:03}", score);
        for (i,char) in score_string.chars().enumerate() {
            updates.push(UpdateElement{ point: trophy_point + (((i+1) as i32)*2,0).into(), value:  convert_to_wide_char(char), fg: None});
        }

        let retry_point: Point = ((((halfway_point.x / 2) - 1) + halfway_point.x),halfway_point.y).into();
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
        path.push("minesweeper_top_score.yaml");
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
                current_score
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

    fn get_emoji_from_chill_factor(chill_factor: ZeroToEight) -> char{
        match chill_factor {
            Zero => '😊',
            One => '🙂',
            Two => '😐',
            Three => '😕',
            Four => '😟',
            Five => '😩',
            Six => '😱',
            Seven => '🤯',
            Eight => '🙃',
        }
    }

    fn do_action_on_point_on_engine<V,T: Default>(&mut self, point: Point, variable: V, callback: fn (variable: V, engine: &mut Box<dyn CanBeEngine>, p: Point) -> Result<T>) -> Result<T> {
        let(mut x, mut y) = point.into();
        if x % 2 == 1 {
            x -= 1;
        }
        x /= 2;
        y -= 2;

        if x >= 0 && x < self.engine_size.width && y >= 0 && y < self.engine_size.height {
            return callback(variable, &mut self.engine, (x,y).into());
        }
        Ok(T::default())
    }
}

impl Component for GameView {
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
        let (move_type, point) = match click {
            MouseAction::Middle(p) => (Some(MoveType::Flag), p),
            MouseAction::Right(p) => (Some(MoveType::Flag), p),
            MouseAction::Left(p) => {
                if self.retry_button_location.contains(&click.to_point()) {
                    self.reset();
                    click_actions.push(Refresh);
                    (None, (0,0).into())
                } else {
                    (Some(MoveType::Dig), p)
                }
            },
            MouseAction::Move(p) => {
                self.chill_factor = self.do_action_on_point_on_engine(p, (), |_, e, p| {
                    e.get_chill_factor(Cell{x: p.x, y: p.y})
                })?;
                return Ok(vec![]);
            }
            MouseAction::Double(p) => { (Some(MoveType::DigAround), p)},
            _ => (None, (0,0).into())
        };
        if let Some(mov) = move_type {
            let move_result = self.do_action_on_point_on_engine(point, mov, |m,e, p| {
                e.play_move(m, Cell { x: p.x, y: p.y })
            })?;
            if let Complete(result) = move_result {
                let score = if result == CompleteState::Win {
                    self.engine.get_board_state().0.game_run_time
                } else {
                    u64::MAX
                };
                self.load_best_score(score);
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

    fn get_chill_factor(&mut self, _: Cell) -> Result<ZeroToEight> {
        Ok(Eight)
    }
}
