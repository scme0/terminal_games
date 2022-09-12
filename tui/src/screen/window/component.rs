use std::fmt::{Debug, Formatter};
use uuid::Uuid;
use crate::screen::{Dimension};
use crate::screen::window::has_close_action::HasCloseAndRefreshActions;
use crate::screen::window::mouse_action::MouseAction;
use crate::screen::window::update_element::UpdateElement;

pub trait Component<T: HasCloseAndRefreshActions + PartialEq + Clone> {
    fn get_id(&self) -> Uuid;
    fn get_size(&self) -> Dimension;
    fn get_updates(&mut self) -> crossterm::Result<Vec<UpdateElement>>;
    fn handle_click(&mut self, click: MouseAction) -> crossterm::Result<Vec<T>>;
}

impl<T> Debug for dyn Component<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
