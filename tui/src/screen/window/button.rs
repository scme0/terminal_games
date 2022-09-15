use crossterm::Result;
use uuid::Uuid;
use crate::screen::dimension::Dimension;
use crate::screen::window::component::Component;
use crate::screen::window::has_close_action::HasCloseAndRefreshActions;
use crate::screen::window::mouse_action::MouseAction;
use crate::screen::window::update_element::UpdateElement;

#[derive(Debug, Clone)]
pub struct ButtonComponent<T: HasCloseAndRefreshActions + PartialEq + Clone> {
    id: Uuid,
    label: Box<str>,
    size: Dimension,
    changed: bool,
    pub click_action: T,
}

impl<T: HasCloseAndRefreshActions + PartialEq + Clone> ButtonComponent<T> {
    pub fn new(label: Box<str>, size: Dimension, click_action: T) -> Self {
        return ButtonComponent {
            id: Uuid::new_v4(),
            label,
            size,
            changed: true,
            click_action,
        };
    }
}

impl<T: HasCloseAndRefreshActions + PartialEq + Clone> Component<T> for ButtonComponent<T> {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_size(&self) -> Dimension {
        return self.size;
    }

    fn get_state(&mut self) -> Result<Vec<UpdateElement>> {
        let mut updates = vec![];
        if self.changed {
            let mut y = 0;
            if y > 1 {
                y = y / 2;
            }
            let mut x = 0;
            let label_len = self.label.len() as i32;
            if self.size.width > label_len {
                x = self.size.width / 2 as i32 - label_len / 2 as i32;
            }
            for c in self.label.chars() {
                updates.push(UpdateElement {
                    point: (x,y).into(),
                    value: c,
                    fg: None,
                });
                x += 1;
                if x >= self.size.width {
                    break;
                }
            }
        }
        return Ok(updates);
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
        self.get_state()
    }

    fn handle_click(&mut self, click: MouseAction) -> Result<Vec<T>> {
        Ok(match click {
            MouseAction::Left(_) => vec![self.click_action.clone()],
            _ => vec![]
        })
    }
}
