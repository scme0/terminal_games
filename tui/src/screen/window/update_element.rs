use crossterm::style::Color;
use crate::screen::Point;

#[derive(Debug, Copy, Clone)]
pub struct UpdateElement {
    pub point: Point,
    pub value: char,
    pub fg: Option<Color>,
}
