pub mod window;

use std::cmp::Ordering;
use crate::{MouseAction, Component};
use crossterm::{cursor, queue, style::{self, Color, StyledContent, Stylize}, ErrorKind, Result, terminal};
use log::info;
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{stdout, Stdout, Write};
use std::ops::{Add, Sub};
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
    pub x: i32,
    pub y: i32
}

impl From<Point> for (i32, i32) {
    fn from(c: Point) -> (i32, i32) {
        let Point {x, y} = c;
        return (x, y);
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        (self.x + rhs.x, self.y + rhs.y).into()
    }
}

// impl From<(usize, usize)> for Point {
//     fn from(p: (usize, usize)) -> Self {
//         Point {x: p.0, y: p.1 as}
//     }
// }

impl From<(i32, i32)> for Point {
    fn from(p: (i32, i32)) -> Self {
        Point {x: p.0, y: p.1}
    }
}

pub struct Screen {
    width: i32,
    height: i32,
    windows: Vec<Window>,
    buffer: HashMap<Uuid, HashMap<Point, StyledContent<String>>>
}

impl Screen {
    pub fn new(width: i32, height: i32) -> Self{
        Screen {windows: vec![], buffer: HashMap::new(), width, height}
    }

    // Gets the top-most window for a specific point.
    pub fn handle_click(&mut self, click: MouseAction) -> Result<ClickAction> {
        let (x,y) = click.to_point().into();
        let some_window = self.windows.iter_mut().enumerate().find(|(i,w)| {
            return x >= w.x && x < w.x + w.width + 1 &&
                y >= w.y && y < w.y + w.height + 1;
        });
        if let Some((idx, window)) = some_window {
            // info!("got hit: {}", window.id);
            if !window.can_move || window.z == 0 {
                let mut p = click.to_point();
                p.x -= window.x;
                p.y -= window.y;
                return window.handle_click(match click {
                    MouseAction::DownMiddle(_) => MouseAction::DownMiddle(p),
                    MouseAction::DownLeft(_) => MouseAction::DownLeft(p),
                    MouseAction::DownRight(_) => MouseAction::DownRight(p),
                    MouseAction::UpMiddle(_) => MouseAction::UpMiddle(p),
                    MouseAction::UpLeft(_) => MouseAction::UpLeft(p),
                    MouseAction::UpRight(_) => MouseAction::UpRight(p),
                    MouseAction::Drag(_, mut to) => {
                        to.x -= window.x;
                        to.y -= window.y;
                        MouseAction::Drag(p, to)
                    }
                });
            } else {
                info!("Here!: {:?}", click);
                if let MouseAction::DownLeft(_) = click {
                    info!("will do stuff after here...");
                    //info!("will shuffle: {:?}", self.windows.iter().map(|w| (w.z, w.id)));
                    self.shuffle_windows_back_from_z(0, 0);
                    //info!("shuffled: {:?}", self.windows.iter().map(|w| (w.z, w.id)));
                    self.windows[idx].z = 0;
                    //info!("zeroed this window: {:?}", self.windows.iter().map(|w| (w.z, w.id)));
                    self.windows.sort_by(|w1,w2| {
                        return if w1.z > w2.z {
                            Ordering::Greater
                        } else if w1.z < w2.z {
                            Ordering::Less
                        } else {
                            Ordering::Equal
                        }
                    });
                    //info!("sorted: {:?}", self.windows.iter().map(|w| (w.z, w.id)));
                    self.refresh()?;
                }
                info!("After Here!...");
            }
        }
        return Ok(ClickAction::None);
    }

    pub fn change_size(&mut self, width: i32, height: i32) -> Result<()>{
        self.width = width;
        self.height = height;
        self.refresh()
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
        let mut refresh = false;
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

            if window.refresh {
                buffer.clear();
                refresh = true;
            }

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

        if refresh {
            self.refresh()?;
        }
        Ok(())
    }

    pub fn add(&mut self, window:Window) -> Result<()> {
        let window_id = window.id;
        // info!("add window: {}", window_id);
        let some_idx = self.windows.binary_search_by_key(&window.z, |w| w.z);
        match some_idx {
            Ok(i) => {
                self.shuffle_windows_back_from_z(i, window.z);
                self.windows.insert(i, window);
            },
            Err(i) => self.windows.insert(i, window)
        }
        self.buffer.insert(window_id, HashMap::new());
        self.refresh()?;
        Ok(())
    }

    fn shuffle_windows_back_from_z(&mut self, start_index:usize, z: i32){
        let mut prev_z = z;
        for idx in start_index..self.windows.len() {
            if self.windows[idx].z == prev_z {
                self.windows[idx].z += 1;
                prev_z = self.windows[idx].z;
            }
        }
    }

    pub fn remove_all(&mut self, window_ids: Vec<Uuid>) -> Result<()> {
        let mut windows_removed = false;
        for window_id in window_ids.iter() {
            // info!("remove window: {}", window_id);
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
            let current_point = Point {x: point.x + i as i32, y: point.y};
            if !point_map.contains(&current_point) {
                point_map.insert(current_point);
                let styled_char = StyledContent::new(value.style().clone(), c.to_string());
                queue!(stdout, cursor::MoveTo(current_point.x as u16, current_point.y as u16), style::Print(styled_char))?;
            }
        }
        Ok(())
    }
}
