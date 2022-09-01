use crate::button::ButtonType;
use crate::game::ProgramState::{InGame, PlayAgain, Quit, StartScreen};
use crate::screen::window::{ComponentType, ComponentWrapper};
use crate::{ButtonComponent, ClickType, Component, GameComponent, UpdateElement, Window};
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::{execute, terminal, ErrorKind, Result};
use log::{info, warn};
use minesweeper_engine::Engine;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::stdout;
use std::ptr::null;
use uuid::Uuid;
use crate::screen::Screen;

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
    screen: Screen,
    program_state: ProgramState,
    game_screen: GameComponent,
    go_button: ButtonComponent,
}

impl State {
    fn new() -> Self {
        let mut state = State {
            program_state: StartScreen,
            game_screen: GameComponent::new(),
            go_button: ButtonComponent::new(Box::from("Go"), 5, 1, ButtonType::Easy),
            screen: Screen::new(),
        };
        state.screen.add(Window::new(
            10,
            5,
            1,
            Box::from(state.go_button.clone()),
            true,
            Box::default(),
        )).expect("");
        return state;
    }

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
                ButtonType::Home => {
                    self.screen.remove(self.game_screen.id());
                    StartScreen
                },
                ButtonType::Quit => Quit,
            };
            if let InGame(game_type) = program_state {
                self.game_screen.start(game_type);
                self.screen.add(Window::new(10, 5, 0, Box::from(self.game_screen.clone()), true, Box::default()))?;
            }
            self.program_state = program_state;
        }
        Ok(())
    }

    fn get_window_updates(&self) -> HashMap<Uuid, Vec<UpdateElement>> {
        let mut window_updates = HashMap::new();
        match self.program_state {
            StartScreen => {
                window_updates.insert(self.go_button.id(), self.go_button.update());
            }
            InGame(_) => {
                window_updates.insert(self.game_screen.id(), self.game_screen.update());
            }
            PlayAgain(_, _) => {}
            Quit => {}
        }
        window_updates
    }

    fn update_state(&mut self, component: ComponentType, click: ClickType) -> Result<()> {
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
            let selected_component = self.screen.get_impact_window_id(x, y);

            if let Some((id, componentType, window_x, window_y)) = selected_component {
                let result = match button {
                    MouseButton::Left => {
                        let relative_x = x - window_x;
                        let relative_y = y - window_y;
                        Some((componentType, ClickType::Left(relative_x, relative_y)))
                    }
                    MouseButton::Middle => {
                        let relative_x = x - window_x;
                        let relative_y = y - window_y;
                        Some((componentType, ClickType::Middle(relative_x, relative_y)))
                    }
                    _ => None,
                };
                return Ok(result);
            }
        }
        Ok(None)
    }

    fn game_loop(&mut self) -> Result<()> {
        // Start game loop.
        loop {
            self.screen.draw(self.get_window_updates())?;
            match read()? {
                Event::Mouse(event) => {
                    match self.handle_mouse_click(event)? {
                        Some((component, clickType)) => {
                            self.update_state(component, clickType)?;
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
}

pub fn start() -> Result<()> {
    let mut state: State = State::new();
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    execute!(stdout, EnableMouseCapture);

    let result = state.game_loop();

    execute!(stdout, DisableMouseCapture);
    terminal::disable_raw_mode()?;

    return result;
}

fn quit_game() -> Result<()> {
    Err(ErrorKind::new(io::ErrorKind::Interrupted, "Program exit"))
}
