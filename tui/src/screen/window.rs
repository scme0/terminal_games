use crate::button::ButtonType;
use crate::{ButtonComponent, GameComponent};
use crossterm::style::{Color, StyledContent};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct UpdateElement {
    pub x: usize,
    pub y: usize,
    pub value: char,
    pub bg: Color,
    pub fg: Color,
}

#[derive(Debug, Copy, Clone)]
pub enum ClickType {
    Middle(usize, usize),
    Left(usize, usize),
}

impl Display for ClickType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClickType::Middle(x, y) => write!(f, "Middle with x: {} y: {}", x, y)?,
            ClickType::Left(x, y) => write!(f, "Left with x: {} y: {}", x, y)?,
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ComponentType {
    Button(ButtonType),
    GameScreen,
}

impl Display for ComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub enum ComponentWrapper {
    Button(ButtonComponent),
    GameScreen(GameComponent),
}

impl Component for ComponentWrapper {
    fn size(&self) -> (usize, usize) {
        match self {
            ComponentWrapper::Button(c) => c.size(),
            ComponentWrapper::GameScreen(c) => c.size(),
        }
    }
    fn update(&self) -> Vec<UpdateElement> {
        match self {
            ComponentWrapper::Button(c) => c.update(),
            ComponentWrapper::GameScreen(c) => c.update(),
        }
    }
}

pub trait Component {
    fn size(&self) -> (usize, usize);
    fn update(&self) -> Vec<UpdateElement>;
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: Uuid,
    pub z: i32,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub updates: Vec<UpdateElement>,
    pub show_border: bool,
    pub border_title: Box<str>,
    pub component_type: ComponentType,
}

impl Window {
    pub fn new(
        x: usize,
        y: usize,
        z: i32,
        width: usize,
        height: usize,
        updates: Vec<UpdateElement>,
        component_type: ComponentType,
        show_border: bool,
        border_title: Box<str>,
    ) -> Self {
        let id = Uuid::new_v4();
        return Window {
            id,
            x,
            y,
            z,
            width,
            height,
            updates,
            show_border,
            border_title,
            component_type,
        };
    }
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        return other.id == self.id;
    }
}

impl PartialOrd for Window {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.z < other.z {
            return Some(Ordering::Less);
        } else if self.z > other.z {
            return Some(Ordering::Greater);
        } else if self.z == other.z {
            return Some(Ordering::Equal);
        }
        return None;
    }
}
