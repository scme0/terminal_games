use crate::{ClickType, Component, UpdateElement};
use crossterm::style::Color;
use std::fmt::{Display, Formatter};
use uuid::Uuid;
use crate::screen::window::ComponentType;

#[derive(Debug, Copy, Clone)]
pub enum ButtonType {
    Easy,
    Medium,
    Hard,
    Retry,
    Home,
    Quit,
}

impl Display for ButtonType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct ButtonComponent {
    id: Uuid,
    label: Box<str>,
    width: usize,
    height: usize,
    changed: bool,
    pub button_type: ButtonType,
}

impl ButtonComponent {
    pub fn new(label: Box<str>, width: usize, height: usize, button_type: ButtonType) -> Self {
        return ButtonComponent {
            id: Uuid::new_v4(),
            label,
            width,
            height,
            changed: true,
            button_type,
        };
    }
}

impl Component for ButtonComponent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn size(&self) -> (usize, usize) {
        return (self.height, self.width);
    }

    fn update(&self) -> Vec<UpdateElement> {
        let mut updates = vec![];
        if self.changed {
            let mut x = 0;
            if x > 1 {
                x = x / 2;
            }
            let mut y = 0;
            let label_len = self.label.len();
            if self.width > label_len {
                y = self.width / 2 - label_len / 2;
            }
            let i = 0;
            for c in self.label.chars() {
                y += 1;
                if y >= self.width {
                    break;
                }
                updates.push(UpdateElement {
                    x,
                    y: y + i,
                    value: c,
                    fg: Color::White,
                })
            }
        }
        return updates;
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::Button(self.button_type)
    }
}
