use crate::{ButtonComponent, Click, GameComponent, Window};
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::{execute, terminal, ErrorKind, Result};
use log::info;
use std::io;
use std::io::stdout;
use crate::game_screen::GameType;
use crate::screen::{ClickAction, Screen};

struct State {
    screen: Screen,
}

impl State {
    fn new() -> Self {
        let mut state = State {
            screen: Screen::new(),
        };
        state.screen.add(Window::new(
            10,
            5,
            1,
            Box::from(ButtonComponent::new(Box::from("Go"), 5, 1, ClickAction::Easy)),
            true,
            Box::default(),
        )).expect("");
        return state;
    }

    fn handle_click_action(&mut self, click_action: ClickAction){
        match click_action {
            ClickAction::Easy => {
                info!("here!");
                self.screen.add(Window::new(
                    10,
                    5,
                    0,
                    Box::from(GameComponent::new(GameType::Easy)),
                    true,
                    Box::default(),
                )).expect("");
            }
            ClickAction::Medium => {}
            ClickAction::Hard => {}
            ClickAction::Quit => {}
            ClickAction::Home => {}
            ClickAction::Retry => {}
            ClickAction::Close(window_id) => {
                self.screen.remove(window_id);
            }
            _ => {}
        }
    }

    fn handle_mouse_click(
        &mut self,
        event: MouseEvent,
    ) -> Result<()> {
        if let MouseEventKind::Down(button) = event.kind {
            let y = event.column as usize;
            let x = event.row as usize;
            let click = match button {
                MouseButton::Left => Click::Left((x,y).into()),
                MouseButton::Right => Click::Right((x,y).into()),
                MouseButton::Middle => Click::Middle((x,y).into())
            };
            let click_action = self.screen.handle_click(click)?;

            self.handle_click_action(click_action);
        }
        Ok(())
    }

    fn game_loop(&mut self) -> Result<()> {
        // Start game loop.
        loop {
            self.screen.draw()?;
            match read()? {
                Event::Mouse(event) => self.handle_mouse_click(event)?,
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
    execute!(stdout, EnableMouseCapture)?;

    let result = state.game_loop();

    execute!(stdout, DisableMouseCapture)?;
    terminal::disable_raw_mode()?;

    return result;
}

fn quit_game() -> Result<()> {
    Err(ErrorKind::new(io::ErrorKind::Interrupted, "Program exit"))
}
