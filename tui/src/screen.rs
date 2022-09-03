pub mod window;

use crate::{Click, Component};
use crossterm::{cursor, queue, style::{self, Color, StyledContent, Stylize}, ErrorKind, Result, terminal};
use log::info;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{stdout, Stdout, Write};
use uuid::Uuid;
use minesweeper_engine::CompleteState::Win;
use window::Window;

#[derive(Debug, Clone)]
pub enum ClickAction {
    None,
    Easy,
    Medium,
    Hard,
    Quit,
    Home,
    Retry,
    Close(Vec<Uuid>)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    x: usize,
    y: usize
}

impl From<Point> for (usize, usize) {
    fn from(c: Point) -> (usize, usize) {
        let Point {x, y} = c;
        return (x, y);
    }
}

impl From<(usize, usize)> for Point {
    fn from(p: (usize, usize)) -> Self {
        Point {x: p.0, y: p.1}
    }
}

impl From<(i32, i32)> for Point {
    fn from(p: (i32, i32)) -> Self {
        Point {x: p.0 as usize, y: p.1 as usize}
    }
}

pub struct Screen {
    windows: Vec<Window>,
    buffer: HashMap<Uuid, HashMap<Point, StyledContent<String>>>
}

impl Screen {
    pub fn new() -> Self{
        Screen {windows: vec![], buffer: HashMap::new()}
    }

    // Gets the top-most window for a specific point.
    pub fn handle_click(&mut self, click: Click) -> Result<ClickAction> {
        let (x,y) = click.to_point().into();
        for window in self.windows.iter_mut() {
            info!("looking for window click hit: {}", window.id);
            if x >= window.x && x < window.x + window.width &&
                y >= window.y && y < window.y + window.height {
                info!("got hit: {}", window.id);
                return window.handle_click(click);
            }
        }
        return Ok(ClickAction::None);
    }

    // When no updates have happened but a window has been removed or the terminal has been resized.
    pub fn refresh(&mut self) -> Result<()> {
        let mut point_map = HashSet::new();
        let mut stdout = stdout();
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        for window in self.windows.iter() {
            let some_buffer = self.buffer.get(&window.id);
            let buffer = match some_buffer {
                Some(b) => b,
                None => Err(ErrorKind::new(io::ErrorKind::Other, "Should always be Some here!"))?
            };
            for (point, value) in buffer.iter() {
                Screen::draw_value(&mut stdout, &mut point_map, *point, value.clone())?;
            }
        }
        queue!(stdout,cursor::Hide)?;
        stdout.flush()?;
        Ok(())
    }

    // Draw specific updates for a window. If the update is behind another window, it will only be buffered.
    pub fn draw(&mut self) -> Result<()> {
        let mut point_map = HashSet::new();
        // ensure that windows below other windows do not draw over the top.
        // also draw border and title if set.
        let mut stdout = stdout();
        for window in self.windows.iter_mut() {
            let some_buffer = self.buffer.get_mut(&window.id);
            let buffer = match some_buffer {
                Some(b) => b,
                None => Err(ErrorKind::new(io::ErrorKind::Other, "Should always be Some here!"))?
            };

            for update_element in window.get_updates()?.iter(){
                if update_element.point.y > window.height || update_element.point.x > window.width {
                    continue;
                }
                let mut value = update_element
                    .value
                    .to_string()
                    .on(Color::Rgb { r: 0, g: 0, b: 0 });
                if let Some(fg) = update_element.fg {
                    value = value.with(fg);
                }
                let absolute_x = window.x + update_element.point.x;
                let absolute_y = window.y + update_element.point.y;

                buffer.insert((absolute_x, absolute_y).into(), value.clone());

                Screen::draw_value(&mut stdout,&mut point_map, Point {x: absolute_x, y: absolute_y}, value)?;
            }

            for (key, _) in buffer {
                point_map.insert(*key);
            }
        }
        queue!(stdout,cursor::Hide)?;
        stdout.flush()?;
        Ok(())
    }

    pub fn add(&mut self, window:Window) -> Result<()> {
        let window_id = window.id;
        info!("add window: {}", window_id);
        let some_idx = self.windows.binary_search_by_key(&window.z, |w| w.z);
        match some_idx {
            Ok(_) => Err(ErrorKind::new(io::ErrorKind::Other, "Window with this z value already exists"))?,
            Err(i) => self.windows.insert(i, window)
        }
        self.buffer.insert(window_id, HashMap::new());
        self.refresh()?;
        Ok(())
    }

    pub fn remove_all(&mut self, window_ids: Vec<Uuid>) -> Result<()> {
        let mut windows_removed = false;
        for window_id in window_ids.iter() {
            info!("remove window: {}", window_id);
            let some_idx = self.windows.iter().enumerate().find(|w| w.1.id == *window_id);
            if let Some((idx, _)) = some_idx {
                let window = self.windows.remove(idx);
                self.buffer.remove(&window.id);
                windows_removed = true;
            }
        }
        if windows_removed {
            self.refresh()?;
        }
        Ok(())
    }

    fn draw_value(stdout: &mut Stdout, point_map: &mut HashSet<Point>, point: Point, value: StyledContent<String>) -> Result<()>{
        let value_len = value.content().len();
        if value_len == 0 {
            return Ok(());
        }
        for (i,c) in value.content().chars().enumerate() {
            let current_point = Point {x: point.x + i, y: point.y};
            if !point_map.contains(&current_point) {
                point_map.insert(current_point);
                let styled_char = StyledContent::new(value.style().clone(), c.to_string());
                queue!(stdout, cursor::MoveTo(current_point.x as u16, current_point.y as u16), style::Print(styled_char))?;
            }
        }
        Ok(())
    }
}
