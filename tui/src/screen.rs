pub mod window;

use crate::screen::window::ComponentWrapper;
use crate::{ClickType, Component};
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
use std::collections::BinaryHeap;
use std::io;
use std::io::{stdout, Stdout, Write};
use uuid::Uuid;
use window::Window;

pub fn draw(windows: Vec<Window>) -> Result<()> {
    // ensure that windows below other windows do not draw over the top.
    // also draw border and title if set.
    let mut stdout = stdout();
    for window in windows.iter() {
        // info!("window: x: {}, y: {}, width: {}, height: {}", window.x, window.y, window.width, window.height);
        for updateElement in window.updates.iter() {
            let value = updateElement
                .value
                .to_string()
                .with(updateElement.fg)
                .on(Color::Rgb {r:0, g:0, b:0});
            if updateElement.y > window.width || updateElement.x > window.height {
                continue;
            }
            let absolute_x = (window.x + updateElement.x) as u16;
            let absolute_y = (window.y + updateElement.y) as u16;

            // info!("printing: value: {}, x: {}, y: {}", value, absolute_x, absolute_y);
            queue!(
                    stdout,
                    cursor::MoveTo(absolute_y, absolute_x),
                    style::Print(value),
                    cursor::Hide
                )?;
        }
    }
    stdout.flush()?;
    Ok(())
}
