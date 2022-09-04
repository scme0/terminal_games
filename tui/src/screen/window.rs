use crossterm::style::Color;
use crossterm::Result;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use uuid::Uuid;
use crate::screen::{ClickAction, Point};

#[derive(Debug, Copy, Clone)]
pub struct UpdateElement {
    pub point: Point,
    pub value: char,
    pub fg: Option<Color>,
}

#[derive(Debug, Copy, Clone)]
pub enum Click {
    Middle(Point),
    Left(Point),
    Right(Point)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BorderStyle {
    None,
    Double,
    Single,
    Dotted
}

struct BorderElements {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    horizontal: char,
    vertical: char,
    label_frame_left: char,
    label_frame_right: char
}

impl BorderElements {
    fn new(border_style: BorderStyle) -> Self {
        match border_style {
            BorderStyle::Double => BorderElements { top_left: '╔', top_right: '╗', bottom_left: '╚', bottom_right: '╝', horizontal: '═', vertical: '║', label_frame_left: '╡', label_frame_right: '╞'},
            BorderStyle::Single => BorderElements { top_left: '┏', top_right: '┓', bottom_left: '┗', bottom_right: '┛', horizontal: '━', vertical: '┃', label_frame_left: '┫', label_frame_right: '┣'},
            BorderStyle::Dotted => BorderElements { top_left: '┏', top_right: '┓', bottom_left: '┗', bottom_right: '┛', horizontal: '┅', vertical: '┇', label_frame_left: '╏', label_frame_right: '╏'},
            BorderStyle::None => BorderElements { top_left: '\0', top_right: '\0', bottom_left: '\0', bottom_right: '\0', horizontal: '\0', vertical: '\0', label_frame_left: '\0', label_frame_right: '\0'},
        }
    }
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
            Click::Right(point) => write!(f, "Right with x: {} y: {}", point.x, point.y)?,
        }
        Ok(())
    }
}

pub trait Component {
    fn get_id(&self) -> Uuid;
    fn get_size(&self) -> (usize, usize);
    fn get_updates(&mut self) -> Result<Vec<UpdateElement>>;
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
    border_style: BorderStyle,
    border_title: Box<str>,
    component: Box<dyn Component>,
    refresh: bool,
}

impl Window {
    pub fn new(
        x: usize,
        y: usize,
        z: i32,
        component: Box<dyn Component>,
        border_style: BorderStyle,
        border_title: Box<str>,
    ) -> Self {
        let id = component.get_id();
        let (mut width,mut height) = component.get_size();
        if border_style != BorderStyle::None {
            width += 1;
            height += 1;
        }
        return Window {
            id,
            x,
            y,
            z,
            width,
            height,
            border_style,
            border_title,
            component,
            refresh: true
        };
    }

    fn draw_border(&self) -> Result<Vec<UpdateElement>> {
        let border_elements = BorderElements::new(self.border_style);
        let mut updates = vec![];
        let mut title = self.border_title.clone();
        let mut title_len = title.chars().count();
        if title_len >= self.width {
            title = Box::from(&title[..self.width - 1]);
            title_len = self.width - 1;
        }
        let top_left = (0, 0);
        let top_right = (self.width, 0);
        let bottom_left = (0, self.height);
        let bottom_right = (self.width, self.height);
        if top_left.1 >= 0 {
            if top_left.0 >= 0 {
                // draw from top_left corner.
                updates.push(UpdateElement {point: top_left.into(), value: border_elements.top_left, fg: None});
            }
            // draw from top_right corner.
            updates.push(UpdateElement {point: top_right.into(), value: border_elements.top_right, fg: None});
            let mut top_line_offset = 1;
            if title_len > 0 {
                top_line_offset = title_len + 3;
                // draw pre-title char
                updates.push(UpdateElement {point: Point {x: top_left.0 as usize + 1, y: top_left.1 as usize }, value: border_elements.label_frame_left, fg: None});
                // draw title
                for x in top_left.0 + 2..top_left.0 + 2 + title_len {
                    updates.push(UpdateElement {point: Point {x: x as usize, y: top_left.1 as usize }, value: title.chars().nth(x - 2).unwrap(), fg: None});
                }
                // draw post-title char
                updates.push(UpdateElement {point: Point {x: top_left.0 as usize + 2 + title_len, y: top_left.1 as usize }, value: border_elements.label_frame_right, fg: None});
            }
            // draw from top_left to bottom_left.
            for x in top_left.0 + top_line_offset..top_right.0 {
                updates.push(UpdateElement {point: Point {x: x as usize, y: top_left.1 as usize}, value: border_elements.horizontal, fg: None});
            }
        }
        if top_left.0 >= 0 {
            // draw from bottom_left corner.
            updates.push(UpdateElement {point: bottom_left.into(), value: border_elements.bottom_left, fg: None});
            // draw from top_left to bottom_left.
            for y in (top_left.1 + 1)..bottom_left.1 {
                updates.push(UpdateElement {point: Point {x: top_left.0 as usize, y: y as usize}, value: border_elements.vertical, fg: None});
            }
        }
        // draw from bottom_right corner.
        updates.push(UpdateElement {point: bottom_right.into(), value: border_elements.bottom_right, fg: None});
        // draw from bottom_left to bottom_right
        for x in (bottom_left.0 + 1)..bottom_right.0 {
            updates.push(UpdateElement {point: Point {x: x as usize, y: bottom_left.1 as usize}, value: border_elements.horizontal, fg: None});
        }
        // draw from top_right to bottom_right
        for y in (top_right.1 + 1)..bottom_right.1 {
            updates.push(UpdateElement {point: Point {x: top_right.0 as usize, y: y as usize}, value: border_elements.vertical, fg: None});
        }
        Ok(updates)
    }
}

impl Component for Window {
    fn get_id(&self) -> Uuid {
        self.component.get_id()
    }

    fn get_size(&self) -> (usize, usize) {
        self.component.get_size()
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
        let mut updates = match self.refresh && self.border_style != BorderStyle::None {
            true => self.draw_border()?,
            false => vec![],
        };

        for update in self.component.get_updates()?.iter() {
            let point = match self.border_style != BorderStyle::None {
                true => Point{x: update.point.x + 1, y: update.point.y + 1 },
                false => update.point,
            };
            updates.push(UpdateElement {point, value: update.value, fg: update.fg });
        }

        self.refresh = false;

        return Ok(updates);
    }

    fn handle_click(&mut self, click: Click) -> Result<ClickAction> {
        fn calculate_relative_x_y(window: &Window, point: Point) -> Point{
            match window.border_style != BorderStyle::None {
                true => (point.x - window.x - 1, point.y - window.y - 1).into(),
                false => (point.x - window.x, point.y - window.y).into()
            }
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
