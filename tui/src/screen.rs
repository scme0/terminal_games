pub mod window;

use crate::{Click, Component};
use crossterm::{cursor, queue, style::{self, Color, StyledContent, Stylize}, ErrorKind, Result, terminal};
use log::info;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{stdout, Write};
use uuid::Uuid;
use window::Window;

#[derive(Debug, Copy, Clone)]
pub enum ClickAction {
    None,
    Easy,
    Medium,
    Hard,
    Quit,
    Home,
    Retry,
    Close(Uuid)
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
            if x >= window.x && x < window.x + window.height &&
                y >= window.y && y < window.y + window.width {
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
        queue!(stdout, terminal::Clear(terminal::ClearType::Purge))?;
        queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;
        queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorUp))?;
        queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        for window in self.windows.iter() {
            let some_buffer = self.buffer.get(&window.id);
            let buffer = match some_buffer {
                Some(b) => b,
                None => Err(ErrorKind::new(io::ErrorKind::Other, "Should always be Some here!"))?
            };
            for (point, value) in buffer.iter() {
                if !point_map.contains(&point) {
                    point_map.insert(point);
                    queue!(stdout, cursor::MoveTo(point.y as u16, point.x as u16), style::Print(value))?;
                }
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
        for window in self.windows.iter() {
            let some_buffer = self.buffer.get_mut(&window.id);
            let buffer = match some_buffer {
                Some(b) => b,
                None => Err(ErrorKind::new(io::ErrorKind::Other, "Should always be Some here!"))?
            };

            for update_element in window.get_updates().iter(){
                if update_element.y > window.width || update_element.x > window.height {
                    continue;
                }
                let value = update_element
                    .value
                    .to_string()
                    .with(update_element.fg)
                    .on(Color::Rgb { r: 0, g: 0, b: 0 });
                let absolute_x = window.x + update_element.x;
                let absolute_y = window.y + update_element.y;

                buffer.insert((absolute_x, absolute_y).into(), value.clone());
                let key = Point {x: absolute_x, y: absolute_y};
                if !point_map.contains(&key) {
                    point_map.insert(key);
                    // info!("Placing value: {}, {}, {}", updateElement.value, absolute_x, absolute_y);
                    queue!(
                        stdout,
                        cursor::MoveTo(absolute_y as u16, absolute_x as u16),
                        style::Print(value))?;
                }
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
        Ok(())
    }

    pub fn remove(&mut self, window_id: Uuid) {
        info!("remove window: {}", window_id);
        let some_idx = self.windows.iter().enumerate().find(|w| w.1.id == window_id);
        if let Some((idx, _)) = some_idx {
            let _window = self.windows.remove(idx);
            //TODO:draw over removed window area starting from lowest to highest z
        }
    }
}
