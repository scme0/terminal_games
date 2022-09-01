pub mod window;

use crate::screen::window::{ComponentType, ComponentWrapper};
use crate::{ClickType, Component, UpdateElement};
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::{
    cursor, execute, queue,
    style::{self, style, Color, StyledContent, Stylize},
    terminal, ErrorKind, Result,
};
use log::info;
use sorted_list::SortedList;
use std::collections::{HashMap};
use std::io;
use std::io::{stdout, Stdout, Write};
use uuid::Uuid;
use minesweeper_engine::CompleteState::Win;
use window::Window;

pub struct Screen {
    windows: Vec<Window>,
    buffer: HashMap<Uuid, HashMap<(usize,usize), StyledContent<String>>>
}

impl Screen {
    pub fn new() -> Self{
        Screen {windows: vec![], buffer: HashMap::new()}
    }

    // When no updates have happened but a window has been removed or the terminal has been resized.
    pub fn refresh(&self) {

    }

    // Draw specific updates for a window. If the update is behind another window, it will only be buffered.
    pub fn draw(&mut self, window_updates: HashMap<Uuid, Vec<UpdateElement>>) -> Result<()> {
        // ensure that windows below other windows do not draw over the top.
        // also draw border and title if set.
        let mut stdout = stdout();
        for window in self.windows.iter() {
            let mut some_buffer = self.buffer.get_mut(&window.id);
            let mut buffer = match some_buffer {
                Some(b) => b,
                None => {
                    self.buffer.insert(window.id, HashMap::new());
                    match self.buffer.get_mut(&window.id) {
                        None => Err(ErrorKind::new(io::ErrorKind::Other, "Should always be Some here!"))?,
                        Some(m) => m
                    }
                }
            };

            match window_updates.get(&window.id) {
                None => {}
                Some(u) => {
                    for updateElement in u.iter() {
                        let value = updateElement
                            .value
                            .to_string()
                            .with(updateElement.fg)
                            .on(Color::Rgb { r: 0, g: 0, b: 0 });
                        if updateElement.y > window.width || updateElement.x > window.height {
                            continue;
                        }
                        let absolute_x = (window.x + updateElement.x) as u16;
                        let absolute_y = (window.y + updateElement.y) as u16;
                        buffer.insert((updateElement.x, updateElement.y), value.clone());
                        // info!("Placing value: {}, {}, {}", updateElement.value, absolute_x, absolute_y);
                        queue!(
                    stdout,
                    cursor::MoveTo(absolute_y, absolute_x),
                    style::Print(value),
                    cursor::Hide
                    )?;
                    }
                }
            };
        }
        stdout.flush()?;
        Ok(())
    }

    pub fn get_impact_window_id(&self, x:usize, y:usize) -> Option<(Uuid, ComponentType, usize, usize)> {
        for window in self.windows.iter() {
            if y >= window.y
                && y < (window.y + window.width)
                && x >= window.x
                && x < (window.x + window.height)
            {
                return Some((window.id, window.component_type, window.x, window.y));
            }
        }
        None
    }

    pub fn add(&mut self, window:Window) -> Result<()> {
        let some_idx = self.windows.binary_search_by_key(&window.z, |w| w.z);
        match some_idx {
            Ok(_) => Err(ErrorKind::new(io::ErrorKind::Other, "Window with this z value already exists"))?,
            Err(i) => self.windows.insert(i, window)
        }
        Ok(())
    }

    pub fn remove(&mut self, window_id: Uuid) {
        let some_idx = self.windows.iter().enumerate().find(|w| w.1.id == window_id);
        if let Some((idx, _)) = some_idx {
            let window = self.windows.remove(idx);
            //TODO:draw over removed window area starting from lowest to highest z
        }
    }
}
