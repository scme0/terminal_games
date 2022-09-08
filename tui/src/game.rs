use crate::{ButtonComponent, MouseAction, Component, GameComponent, Window};
use crossterm::event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEvent, MouseEventKind, poll};
use crossterm::{execute, terminal, ErrorKind, Result};
use log::info;
use std::io;
use std::io::stdout;
use std::time::{Duration, Instant};
use crate::game_screen::GameType;
use crate::MouseAction::{DoubleLeft, DownLeft, DownMiddle, DownRight, Drag, UpLeft, UpMiddle, UpRight};
use crate::screen::{ClickAction, Point, Screen};
use crate::screen::window::BorderStyle;
use crate::screen::window::BorderStyle::Double;

#[derive(PartialEq)]
enum GameRunState {
    KeepRunning,
    Close
}

struct State {
    screen: Screen,
    last_left_click: Point,
    last_left_click_time: Instant
}

impl State {
    fn new() -> Self {
        let (width, height) = terminal::size().expect("");
        let mut state = State {
            screen: Screen::new(width as i32, height as i32),
            last_left_click: (0,0).into(),
            last_left_click_time: Instant::now()
        };
        state.screen.add(Window::new(
            (10, 5).into(),
            99,
            Box::from(ButtonComponent::new(Box::from("Easy"), (6, 1).into(), ClickAction::Easy)),
            BorderStyle::Single,
            Box::default(),
            false,
            false
        )).expect("");
        state.screen.add(Window::new(
            (20, 5).into(),
            98,
            Box::from(ButtonComponent::new(Box::from("Medium"), (6, 1).into(), ClickAction::Medium)),
            BorderStyle::Single,
            Box::default(),
            false,
            false
        )).expect("");
        state.screen.add(Window::new(
            (30, 5).into(),
            97,
            Box::from(ButtonComponent::new(Box::from("Hard"), (6, 1).into(), ClickAction::Hard)),
            BorderStyle::Single,
            Box::default(),
            false,
            false
        )).expect("");

        return state;
    }

    fn handle_click_actions(&mut self, click_actions: Vec<ClickAction>) -> Result<GameRunState>{
        let mut windows_to_remove = vec![];
        for action in click_actions {
            match action {
                ClickAction::Easy => {
                    self.screen.add(Window::new(
                        (5, 10).into(),
                        0,
                        Box::from(GameComponent::new(GameType::Easy)),
                        BorderStyle::Double,
                        Box::from("Easy peasy"),
                        true,
                        true
                    ))?;
                }
                ClickAction::Medium => {
                    self.screen.add(Window::new(
                        (5, 10).into(),
                        0,
                        Box::from(GameComponent::new(GameType::Medium)),
                        BorderStyle::Double,
                        Box::from("Medium meh"),
                        true,
                        true
                    ))?;
                }
                ClickAction::Hard => {
                    self.screen.add(Window::new(
                        (5, 10).into(),
                        0,
                        Box::from(GameComponent::new(GameType::Hard)),
                        BorderStyle::Double,
                        Box::from("Hard shmard"),
                        true,
                        true
                    ))?;
                }
                ClickAction::Quit => {
                    return Ok(GameRunState::Close);
                }
                ClickAction::Close(window_id) => {
                    windows_to_remove.push(window_id);
                }
            }
        }
        if !windows_to_remove.is_empty() {
            self.screen.remove_all(windows_to_remove)?;
        }
        Ok(GameRunState::KeepRunning)
    }

    fn handle_mouse_click(
        &mut self,
        event: MouseEvent,
    ) -> Result<GameRunState> {
        let x = event.column as i32;
        let mut y = event.row as i32;
        let some_click = match event.kind {
            MouseEventKind::Down(button) => {
                // info!("down! {}, {}, {:?}", x, y, button);
                match button {
                    MouseButton::Left => {
                        let point = (x,y).into();
                        if point == self.last_left_click &&
                            self.last_left_click_time.elapsed().as_millis() <= 500 {
                            Some(DoubleLeft(point))
                        }
                        else{
                            self.last_left_click = point;
                            self.last_left_click_time = Instant::now();
                            Some(DownLeft(point))
                        }
                    },
                    MouseButton::Right => Some(DownRight((x, y).into())),
                    MouseButton::Middle => Some(DownMiddle((x, y).into()))
                }
            },
            MouseEventKind::Up(button) => {
                // info!("up! {}, {}, {:?}", x, y, button);
                match button {
                    MouseButton::Left => Some(UpLeft((x, y).into())),
                    MouseButton::Right => Some(UpRight((x, y).into())),
                    MouseButton::Middle => Some(UpMiddle((x, y).into()))
                }
            },
            MouseEventKind::Drag(button) => {
                match button {
                    MouseButton::Left => {
                        let (_, last_y) = self.last_left_click.into();
                        if y == 0 && last_y != 1 {
                            y = last_y;
                        }
                        let to = (x, y).into();
                        let from = self.last_left_click;
                        self.last_left_click = to;
                        // info!("drag! {:?}, {:?}", to, from);
                        Some(Drag(from, to))

                    },
                    _ => None
                }
            },
            MouseEventKind::Moved => {
                // info!("moved! {}, {}", x, y);
                None
            },
            _ => None
        };
        if let Some(click) = some_click {
            let click_action = self.screen.handle_click(click)?;
            return self.handle_click_actions(click_action);
        }
        Ok(GameRunState::KeepRunning)
    }

    fn game_loop(&mut self) -> Result<()> {
        // Start game loop.
        loop {
            self.screen.draw()?;
            if let Ok(ready) = poll(Duration::from_millis(30)) {
                if ready {
                    match read()? {
                        Event::Mouse(event) => {
                            if self.handle_mouse_click(event)? == GameRunState::Close {
                                return Ok(());
                            }
                        },
                        Event::Resize(width, height) =>
                            self.screen.change_size(width as i32, height as i32)?,
                        Event::Key(key) => match key.code {
                            KeyCode::Char(char) => {
                                if char == 'q' {
                                    return Ok(());
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn start() -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    execute!(stdout, EnableMouseCapture)?;

    let mut state: State = State::new();
    let result = state.game_loop();

    execute!(stdout, DisableMouseCapture)?;
    terminal::disable_raw_mode()?;

    return result;
}
