mod button;
mod game;
mod game_screen;
mod screen;

use crate::button::ButtonComponent;
use crate::game_screen::GameComponent;
use crate::screen::window::{ClickType, Component, UpdateElement, Window};
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::style::Color;
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal, Result,
};
use flexi_logger::{FileSpec, Logger};
use log::{info, warn};
use std::borrow::BorrowMut;
use std::io::{stdin, stdout, Read, Stdout, Write};
// use crate::game::Game;

fn main() -> Result<()> {
    // TODO: Figure out what "expect" is and what it does.
    Logger::try_with_str("info")
        .expect("stuff")
        .log_to_file(FileSpec::default().suppress_timestamp())
        .start()
        .expect("thing");

    info!("*** Minesweeper 0.0.1 ***");
    game::start()
}
