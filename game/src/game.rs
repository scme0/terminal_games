use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEvent, MouseEventKind, poll, read};
use crossterm::{execute, Result, terminal};
use std::io::stdout;
use std::time::{Duration, Instant};
use log::info;
use minesweeper_tui_game_view::game_view::GameView;
use tui::screen::{ClickAction, GameType, Point, Screen};
use tui::screen::border_style::BorderStyle;
use tui::screen::window::Window;
use tui::screen::mouse_action::MouseAction::{Double, Drag, Left, Middle, Move, Right};
use tui::screen::window::button::ButtonComponent;

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
    fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;
        let mut state = State {
            screen: Screen::new(width as i32, height as i32),
            last_left_click: (0,0).into(),
            last_left_click_time: Instant::now()
        };
        state.screen.add(Window::new(
            (0,0).into(),
            99,
            Box::from(ButtonComponent::new(Box::from("Quit"), (6,1).into(), ClickAction::Quit)),
            BorderStyle::Dotted,
            Box::default(),
            false,
            false
        ))?;
        state.screen.add(Window::new(
            (10, 5).into(),
            99,
            Box::from(ButtonComponent::new(Box::from("Easy"), (6, 1).into(), ClickAction::Minesweeper(GameType::Easy))),
            BorderStyle::Single,
            Box::default(),
            false,
            false
        ))?;
        state.screen.add(Window::new(
            (20, 5).into(),
            98,
            Box::from(ButtonComponent::new(Box::from("Medium"), (6, 1).into(), ClickAction::Minesweeper(GameType::Medium))),
            BorderStyle::Single,
            Box::default(),
            false,
            false
        ))?;
        state.screen.add(Window::new(
            (30, 5).into(),
            97,
            Box::from(ButtonComponent::new(Box::from("Hard"), (6, 1).into(), ClickAction::Minesweeper(GameType::Hard))),
            BorderStyle::Single,
            Box::default(),
            false,
            false
        ))?;

        return Ok(state);
    }

    fn handle_click_actions(&mut self, click_actions: Vec<ClickAction>) -> Result<GameRunState>{
        let mut windows_to_remove = vec![];
        for action in click_actions {
            match action {
                ClickAction::Minesweeper(game_type) => {
                    info!("Starting new {:?} game of minesweeper", game_type);
                    self.screen.add(Window::new(
                        (5, 10).into(),
                        0,
                        Box::from(GameView::new(game_type)),
                        BorderStyle::Double,
                        Box::from(format!("{:?}", game_type)),
                        true,
                        true
                    ))?;
                }
                ClickAction::Quit => {
                    info!("Quitting Application");
                    return Ok(GameRunState::Close);
                }
                ClickAction::Close(window_id) => {
                    windows_to_remove.push(window_id);
                }
                ClickAction::Refresh => {
                    info!("Screen refresh requested");
                    self.screen.refresh()?;
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
                match button {
                    MouseButton::Left => {
                        let point = (x,y).into();
                        if point == self.last_left_click &&
                            self.last_left_click_time.elapsed().as_millis() <= 500 {
                            Some(Double(point))
                        }
                        else{
                            self.last_left_click = point;
                            self.last_left_click_time = Instant::now();
                            Some(Left(point))
                        }
                    },
                    MouseButton::Right => Some(Right((x, y).into())),
                    MouseButton::Middle => Some(Middle((x, y).into()))
                }
            },
            MouseEventKind::Up(button) => {
                // Some terminals show "Up" instead of Move.
                match button {
                    MouseButton::Left => Some(Move((x, y).into())),
                    _ => None
                }
            },
            MouseEventKind::Moved => {
                Some(Move((x, y).into()))
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
                        Some(Drag(from, to))
                    },
                    _ => None
                }
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

    let mut state: State = State::new()?;
    let result = state.game_loop();

    execute!(stdout, DisableMouseCapture)?;
    terminal::disable_raw_mode()?;

    return result;
}
