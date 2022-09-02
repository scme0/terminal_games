use crossterm::style::Color;
use crossterm::Result;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use uuid::Uuid;
use crate::screen::{ClickAction, Point};

#[derive(Debug, Copy, Clone)]
pub struct UpdateElement {
    pub x: usize,
    pub y: usize,
    pub value: char,
    pub fg: Color,
}

#[derive(Debug, Copy, Clone)]
pub enum Click {
    Middle(Point),
    Left(Point),
    Right(Point)
}

impl Click {
    pub fn to_point(&self) -> Point {
        match *self {
            Click::Middle(p) => p,
            Click::Left(    p) => p,
            Click::Right(p) => p
        }
    }
}

impl Display for Click {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Click::Middle(point) => write!(f, "Middle with x: {} y: {}", point.x, point.y)?,
            Click::Left(point) => write!(f, "Left with x: {} y: {}", point.x, point.y)?,
            _ => {}
        }
        Ok(())
    }
}

pub trait Component {
    fn get_id(&self) -> Uuid;
    fn get_size(&self) -> (usize, usize);
    fn get_updates(&self) -> Vec<UpdateElement>;
    fn handle_click(&mut self, click: Click) -> Result<ClickAction>;
}

impl Debug for dyn Component {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Window {
    pub id: Uuid,
    pub z: i32,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub show_border: bool,
    pub border_title: Box<str>,
    component: Box<dyn Component>
}

impl Window {
    pub fn new(
        x: usize,
        y: usize,
        z: i32,
        component: Box<dyn Component>,
        show_border: bool,
        border_title: Box<str>,
    ) -> Self {
        let id = component.get_id();
        let (height, width) = component.get_size();
        return Window {
            id,
            x,
            y,
            z,
            width,
            height,
            show_border,
            border_title,
            component
        };
    }
}

impl Component for Window {
    fn get_id(&self) -> Uuid {
        self.component.get_id()
    }

    fn get_size(&self) -> (usize, usize) {
        self.component.get_size()
    }

    fn get_updates(&self) -> Vec<UpdateElement> {
        self.component.get_updates()
    }

    fn handle_click(&mut self, click: Click) -> Result<ClickAction> {
        fn calculate_relative_x_y(window: &Window, point: Point) -> Point{
            (point.x - window.x, point.y - window.y).into()
        }
        self.component.handle_click(match click {
            Click::Middle(p) => Click::Middle(calculate_relative_x_y(self, p)),
            Click::Left(p) => Click::Left(calculate_relative_x_y(self, p)),
            Click::Right(p) => Click::Right(calculate_relative_x_y(self, p))
        })
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
