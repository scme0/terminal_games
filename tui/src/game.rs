use crate::{ButtonComponent, Click, Component, GameComponent, Window};
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
use crate::screen::window::BorderStyle;

struct State {
    screen: Screen,
}

impl State {
    fn new() -> Self {
        let mut state = State {
            screen: Screen::new(),
        };
        state.screen.add(Window::new(
            5,
            10,
            1,
            Box::from(ButtonComponent::new(Box::from("Go"), 2, 1, ClickAction::Easy)),
            BorderStyle::Single,
            Box::default(),
        )).expect("");
        return state;
    }

    fn handle_click_action(&mut self, click_action: ClickAction) -> Result<()>{
        match click_action {
            ClickAction::Easy => {
                let game = GameComponent::new(GameType::Easy);
                let game_id = game.get_id();
                self.screen.add(Window::new(
                    5,
                    10,
                    0,
                    Box::from(game),
                    BorderStyle::Double,
                    Box::from("Easy peasy"),
                ))?;
                let mut button = ButtonComponent::new(Box::from("Close"), 5, 1, ClickAction::None);
                button.update_click_action(ClickAction::Close(vec!{button.get_id(), game_id}));
                self.screen.add(Window::new(
                    5,
                    5,
                    2,
                    Box::from(button),
                    BorderStyle::Dotted,
                    Box::default(),
                ))?;
            }
            ClickAction::Medium => {}
            ClickAction::Hard => {}
            ClickAction::Quit => {}
            ClickAction::Home => {}
            ClickAction::Retry => {}
            ClickAction::Close(window_ids) => {
                self.screen.remove_all(window_ids)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_click(
        &mut self,
        event: MouseEvent,
    ) -> Result<()> {
        let x = event.column as usize;
        let y = event.row as usize;
        match event.kind {
            MouseEventKind::Down(_) => info!("down! {}, {}", x, y),
            MouseEventKind::Up(_) => info!("up! {}, {}", x, y),
            MouseEventKind::Drag(_) => info!("drag! {}, {}", x, y),
            MouseEventKind::Moved => info!("moved! {}, {}", x, y),
            _ => {}
        }
        if let MouseEventKind::Down(button) = event.kind {
            let click = match button {
                MouseButton::Left => Click::Left((x,y).into()),
                MouseButton::Right => Click::Right((x,y).into()),
                MouseButton::Middle => Click::Middle((x,y).into())
            };
            let click_action = self.screen.handle_click(click)?;

            self.handle_click_action(click_action)?;
        }
        Ok(())
    }

    fn game_loop(&mut self) -> Result<()> {
        // Start game loop.
        loop {
            self.screen.draw()?;
            match read()? {
                Event::Mouse(event) => self.handle_mouse_click(event)?,
                Event::Resize(width, height) => self.handle_resize(width, height)?,
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

    fn handle_resize(&mut self, _: u16, _: u16) -> Result<()> {
        self.screen.refresh()
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

fn quit_game() -> Result<()> {
    Err(ErrorKind::new(io::ErrorKind::Interrupted, "Program exit"))
}
