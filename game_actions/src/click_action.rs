use uuid::Uuid;
use tui::screen::window::has_close_action::HasCloseAndRefreshActions;
use crate::game_type::GameType;

#[derive(Debug, Clone, PartialEq)]
pub enum ClickAction {
    Minesweeper(GameType),
    Quit,
    Close(Uuid),
    Refresh
}

impl HasCloseAndRefreshActions for ClickAction {
    fn get_close_action(id: Uuid) -> Self {
        ClickAction::Close(id)
    }

    fn get_refresh_action() -> Self {
        ClickAction::Refresh
    }
}