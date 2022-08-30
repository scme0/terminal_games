use crate::button::ButtonType;
use crate::game::ProgramState::{InGame, PlayAgain, Quit, StartScreen};
use crate::screen::window::{ComponentType, ComponentWrapper};
use crate::{ButtonComponent, ClickType, Component, GameComponent, Window};
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::{execute, terminal, ErrorKind, Result};
use log::{info, warn};
use minesweeper_engine::Engine;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::stdout;
use std::ptr::null;
use crate::screen::draw;

#[derive(Debug, Copy, Clone)]
pub enum GameType {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Copy, Clone)]
enum GameState {
    Fail,
    Success,
}

#[derive(Debug, Copy, Clone)]
enum ProgramState {
    StartScreen,
    InGame(GameType),
    PlayAgain(GameType, GameState),
    Quit,
}

struct State {
    program_state: ProgramState,
    game_screen: GameComponent,
    go_button: ButtonComponent,
}

impl State {
    fn update_game_engine(&mut self, click_type: ClickType) -> Result<()> {
        self.game_screen.click(click_type);
        Ok(())
    }
    fn handle_button_click(
        &mut self,
        button_type: ButtonType,
        click_type: ClickType,
    ) -> Result<()> {
        if let ClickType::Left(_,_) = click_type {
            let program_state = match button_type {
                ButtonType::Easy => InGame(GameType::Easy),
                ButtonType::Medium => InGame(GameType::Medium),
                ButtonType::Hard => InGame(GameType::Hard),
                ButtonType::Retry => InGame(GameType::Easy),
                ButtonType::Home => StartScreen,
                ButtonType::Quit => Quit,
            };
            if let InGame(game_type) = program_state {
                self.game_screen.start(game_type);
            }
            self.program_state = program_state;
        }
        Ok(())
    }

    fn get_window_updates(&self) -> Vec<Window> {
        let mut windows = vec![];
        match self.program_state {
            StartScreen => {
                let (width, height) = self.go_button.size();
                windows.push(Window::new(
                    10,
                    5,
                    1,
                    width,
                    height,
                    self.go_button.update(),
                    ComponentType::Button(ButtonType::Easy),
                    true,
                    Box::from(""),
                ))
            }
            InGame(_) => {
                let (height, width) = self.game_screen.size();
                windows.push(Window::new(
                    10,
                    5,
                    1,
                    width,
                    height,
                    self.game_screen.update(),
                    ComponentType::GameScreen,
                    true,
                    Box::from(""),
                ));
                //windows.push(Window::new(10, 5, 1, width, height, self.go_button.update(), ComponentType::Button(ButtonType::Easy),  true, Box::from("")))
            }
            PlayAgain(_, _) => {}
            Quit => {}
        }
        windows
    }

    fn update_state(&mut self, component: ComponentType, click: ClickType) -> Result<()> {
        // info!("update_state: comp: {}, click {}", component, click);
        match self.program_state {
            StartScreen => match component {
                ComponentType::Button(b) => self.handle_button_click(b, click),
                _ => Ok(()),
            },
            InGame(_) => match component {
                ComponentType::GameScreen => self.update_game_engine(click),
                _ => Ok(()),
            },
            PlayAgain(_, _) => Ok(()),
            Quit => quit_game(),
        }?;
        Ok(())
    }

    fn handle_mouse_click(
        &mut self,
        event: MouseEvent,
    ) -> Result<Option<(ComponentType, ClickType)>> {
        if let MouseEventKind::Down(button) = event.kind {
            let y = event.column as usize;
            let x = event.row as usize;
            let mut selected_component: Option<(ComponentType, usize, usize)> = None;
            for window in self.get_window_updates().iter_mut() {
                info!(
                    "window: x: {}, y: {}, width: {}, height: {}",
                    window.x, window.y, window.width, window.height
                );
                info!("x {}, y {}", x, y);
                if y >= window.y
                    && y < (window.y + window.width)
                    && x >= window.x
                    && x < (window.x + window.height)
                {
                    selected_component = Some((window.component_type, window.x, window.y));
                    break;
                }
            }

            if let Some((componentType, window_x, window_y)) = selected_component {
                let result = match button {
                    MouseButton::Left => {
                        let relative_x = x - window_x;
                        let relative_y = y - window_y;
                        // info!("Left click: x: {}, y: {}", relative_x, relative_y);
                        Some((componentType, ClickType::Left(relative_x, relative_y)))
                    }
                    MouseButton::Middle => {
                        let relative_x = x - window_x;
                        let relative_y = y - window_y;
                        // info!("Middle click: x: {}, y: {}", relative_x, relative_y);
                        Some((componentType, ClickType::Middle(relative_x, relative_y)))
                    }
                    _ => None,
                };
                // match result {
                //     None => warn!("None"),
                //     Some((component, click)) => info!("comp: {}, click: {}", component, click),
                // }
                return Ok(result);
            }
        }
        Ok(None)
    }
}

pub fn start() -> Result<()> {
    let mut state: State = State {
        program_state: StartScreen,
        game_screen: GameComponent::new(),
        go_button: ButtonComponent::new(Box::from("Go"), 5, 1, ButtonType::Easy),
    };
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    execute!(stdout, EnableMouseCapture);

    let result = game_loop(&mut state);

    execute!(stdout, DisableMouseCapture);
    terminal::disable_raw_mode()?;

    return result;
}

fn game_loop(state: &mut State) -> Result<()> {
    // Start game loop.
    loop {
        draw(state.get_window_updates())?;
        match read()? {
            Event::Mouse(event) => {
                match state.handle_mouse_click(event)? {
                    Some((component, clickType)) => {
                        // info!("mouse click handled: co:{}, cl:{}", component, clickType);
                        state.update_state(component, clickType)?;
                    }
                    _ => {
                        //warn!("Got here :(");
                    }
                }
            }
            Event::Key(key) => match key.code {
                KeyCode::Char(char) => {
                    if char == 'q' {
                        quit_game()?;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn quit_game() -> Result<()> {
    Err(ErrorKind::new(io::ErrorKind::Interrupted, "Program exit"))
}
