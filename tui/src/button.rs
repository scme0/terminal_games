use crate::{MouseAction, Component, UpdateElement};
use crossterm::style::Color;
use crossterm::Result;
use uuid::Uuid;
use crate::screen::{ClickAction, Dimension};

#[derive(Debug, Clone)]
pub struct ButtonComponent {
    id: Uuid,
    label: Box<str>,
    size: Dimension,
    changed: bool,
    pub click_action: ClickAction,
}

impl ButtonComponent {
    pub fn new(label: Box<str>, size: Dimension, click_action: ClickAction) -> Self {
        return ButtonComponent {
            id: Uuid::new_v4(),
            label,
            size,
            changed: true,
            click_action,
        };
    }
    pub fn update_click_action(&mut self, click_action: ClickAction) {
        self.click_action = click_action;
    }
}

impl Component for ButtonComponent {
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_size(&self) -> Dimension {
        return self.size;
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
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

    fn handle_click(&mut self, click: MouseAction) -> Result<Vec<ClickAction>> {
        Ok(match click {
            MouseAction::DownLeft(_) => vec![self.click_action.clone()],
            _ => vec![]
        })
    }
}
