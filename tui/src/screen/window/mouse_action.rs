use std::fmt::{Display, Formatter};
use crate::screen::point::Point;

#[derive(Debug, Copy, Clone)]
pub enum MouseAction {
    Middle(Point),
    Left(Point),
    Right(Point),
    Double(Point),
    Move(Point),
    Drag(Point, Point)
}

impl MouseAction {
    pub fn to_point(&self) -> Point {
        match *self {
            MouseAction::Middle(p) => p,
            MouseAction::Left(p) => p,
            MouseAction::Right(p) => p,
            MouseAction::Double(p) => p,
            MouseAction::Move(p) => p,
            MouseAction::Drag(from, _) => from,
        }
    }
}

impl Display for MouseAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseAction::Middle(point) => write!(f, "Middle with {:?}", point)?,
            MouseAction::Left(point) => write!(f, "Left with {:?}", point)?,
            MouseAction::Right(point) => write!(f, "Right with {:?}", point)?,
            MouseAction::Double(point) => write!(f, "Double with {:?}", point)?,
            MouseAction::Move(point) => write!(f, "Move with {:?}", point)?,
            MouseAction::Drag(from, to) => write!(f, "Drag from {:?} to: {:?}", from, to)?,
        }
        Ok(())
    }
}
