use crossterm::style::Color;
use crossterm::Result;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use crossterm::event::Event::Mouse;
use log::info;
use uuid::Uuid;
use crate::screen::{ClickAction, Point};
use crate::screen::ClickAction::Close;

#[derive(Debug, Copy, Clone)]
pub struct UpdateElement {
    pub point: Point,
    pub value: char,
    pub fg: Option<Color>,
}

#[derive(Debug, Copy, Clone)]
pub enum MouseAction {
    DownMiddle(Point),
    DownLeft(Point),
    DownRight(Point),
    UpMiddle(Point),
    UpLeft(Point),
    UpRight(Point),
    Drag(Point, Point)
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

impl MouseAction {
    pub fn to_point(&self) -> Point {
        match *self {
            MouseAction::DownMiddle(p) => p,
            MouseAction::DownLeft(p) => p,
            MouseAction::DownRight(p) => p,
            MouseAction::UpMiddle(p) => p,
            MouseAction::UpLeft(p) => p,
            MouseAction::UpRight(p) => p,
            MouseAction::Drag(from, _) => from
        }
    }
}

impl Display for MouseAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseAction::DownMiddle(point) => write!(f, "DownMiddle with {:?}", point)?,
            MouseAction::DownLeft(point) => write!(f, "DownLeft with {:?}", point)?,
            MouseAction::DownRight(point) => write!(f, "DownRight with {:?}", point)?,
            MouseAction::UpMiddle(point) => write!(f, "UpMiddle with {:?}", point)?,
            MouseAction::UpLeft(point) => write!(f, "UpLeft with {:?}", point)?,
            MouseAction::UpRight(point) => write!(f, "UpRight with {:?}", point)?,
            MouseAction::Drag(from, to) => write!(f, "Drag from {:?} to: {:?}", from, to)?,
        }
        Ok(())
    }
}

pub trait Component {
    fn get_id(&self) -> Uuid;
    fn get_size(&self) -> (i32, i32);
    fn get_updates(&mut self) -> Result<Vec<UpdateElement>>;
    fn handle_click(&mut self, click: MouseAction) -> Result<ClickAction>;
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
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    border_style: BorderStyle,
    border_title: Box<str>,
    component: Box<dyn Component>,
    pub refresh: bool,
    can_move: bool,
    close_point: Option<Point>,
}

impl Window {
    pub fn new(
        x: i32,
        y: i32,
        z: i32,
        component: Box<dyn Component>,
        border_style: BorderStyle,
        border_title: Box<str>,
        can_move: bool,
        can_close: bool
    ) -> Self {
        let id = component.get_id();
        let (mut width,mut height) = component.get_size();
        if border_style != BorderStyle::None {
            width += 1;
            height += 1;
        }

        let close_point = match can_close {
            true => Some((width - 2, 0).into()),
            false => None,
        };

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
            can_move,
            refresh: true,
            close_point
        };
    }

    fn draw_border(&self) -> Result<Vec<UpdateElement>> {
        let border_elements = BorderElements::new(self.border_style);
        let mut updates = vec![];
        let mut title = self.border_title.clone();
        let mut title_len = title.chars().count() as i32;
        if title_len >= self.width {
            title = Box::from(&title[..(self.width - 2) as usize]);
            title_len = self.width - 1;
        }
        let top_left = (0, 0);
        let top_right = (self.width, 0);
        let bottom_left = (0, self.height);
        let bottom_right = (self.width, self.height);
        if top_left.1 >= 0 {
            if top_left.0 >= 0 {
                // draw top_left corner.
                updates.push(UpdateElement {point: top_left.into(), value: border_elements.top_left, fg: None});
            }
            // draw top_right corner.
            updates.push(UpdateElement {point: top_right.into(), value: border_elements.top_right, fg: None});

            let mut top_line_right_offset = 0;
            if let Some(close_pos) = self.close_point {
                top_line_right_offset = 2;
                // draw Close button.
                updates.push(UpdateElement {point: close_pos, value: 'Ⓧ', fg: None});
                updates.push(UpdateElement {point: close_pos + (1,0).into(), value: ' ', fg: None});
            }

            let mut top_line_offset = 1;
            if title_len > 0 {
                top_line_offset = title_len + 3;
                // draw pre-title char
                updates.push(UpdateElement {point: Point {x: top_left.0 + 1, y: top_left.1 }, value: border_elements.label_frame_left, fg: None});
                // draw title
                for x in top_left.0 + 2..top_left.0 + 2 + title_len {
                    updates.push(UpdateElement {point: Point {x, y: top_left.1 }, value: title.chars().nth(x as usize - 2).unwrap(), fg: None});
                }
                // draw post-title char
                updates.push(UpdateElement {point: Point {x: top_left.0 + 2 + title_len, y: top_left.1 }, value: border_elements.label_frame_right, fg: None});
            }
            // draw from top_left to top_right.
            for x in top_left.0 + top_line_offset..top_right.0 - top_line_right_offset {
                updates.push(UpdateElement {point: Point {x, y: top_left.1}, value: border_elements.horizontal, fg: None});
            }
        }
        if top_left.0 >= 0 {
            // draw bottom_left corner.
            updates.push(UpdateElement {point: bottom_left.into(), value: border_elements.bottom_left, fg: None});
            // draw from top_left to bottom_left.
            for y in (top_left.1 + 1)..bottom_left.1 {
                updates.push(UpdateElement {point: Point {x: top_left.0, y}, value: border_elements.vertical, fg: None});
            }
        }
        // draw bottom_right corner.
        updates.push(UpdateElement {point: bottom_right.into(), value: border_elements.bottom_right, fg: None});
        // draw from bottom_left to bottom_right
        for x in (bottom_left.0 + 1)..bottom_right.0 {
            updates.push(UpdateElement {point: Point {x, y: bottom_left.1}, value: border_elements.horizontal, fg: None});
        }
        // draw from top_right to bottom_right
        for y in (top_right.1 + 1)..bottom_right.1 {
            updates.push(UpdateElement {point: Point {x: top_right.0, y}, value: border_elements.vertical, fg: None});
        }
        Ok(updates)
    }
}

impl Component for Window {
    fn get_id(&self) -> Uuid {
        self.component.get_id()
    }

    fn get_size(&self) -> (i32, i32) {
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

    fn handle_click(&mut self, mouse_action: MouseAction) -> Result<ClickAction> {
        info!("A mouse action! {:?}", mouse_action);
        let action_point = mouse_action.to_point();
        if self.border_style != BorderStyle::None &&
            (action_point.x == 0 || action_point.x == self.width
                || action_point.y == 0 || action_point.y == self.height){
            match mouse_action {
                MouseAction::DownLeft(_) => {
                    if let Some(close_point) = self.close_point {
                        if action_point == close_point || action_point == close_point + (1,0).into() {
                            return Ok(Close(vec![self.get_id()]))
                        }
                    }
                }
                MouseAction::Drag(starting_point, drag_point) => {
                    if self.can_move {
                        let movement_vector = starting_point - drag_point;
                        info!("Dragging: {:?}, {:?}, {:?}", starting_point, drag_point, movement_vector);
                        let mut new_x = self.x - movement_vector.x;
                        let mut new_y = self.y - movement_vector.y;
                        if new_x < 0 {
                            new_x = 0;
                        }

                        if new_y < 0 {
                            new_y = 0;
                        }

                        if self.x != new_x || self.y != new_y {
                            self.x = new_x;
                            self.y = new_y;
                            self.refresh = true;
                            info!("moved window: {}, {}", self.x, self.y);
                        }
                    }
                    return Ok(ClickAction::None);
                }
                _ => {}
            }
        } else {
            let rel_point = calculate_relative_x_y(self, action_point);
            info!("Going to send this click to the component at point: {:?}, orig: {:?}, win.x {:?}, win.y {:?}", rel_point, action_point, self.x, self.y);
            return match mouse_action {
                MouseAction::DownMiddle(_) => self.component.handle_click(MouseAction::DownMiddle(rel_point)),
                MouseAction::DownLeft(_) => self.component.handle_click(MouseAction::DownLeft(rel_point)),
                MouseAction::DownRight(_) => self.component.handle_click(MouseAction::DownRight(rel_point)),
                _ => Ok(ClickAction::None)
            };
        }
        Ok(ClickAction::None)
    }
}

fn calculate_relative_x_y(window: &Window, point: Point) -> Point{
    match window.border_style != BorderStyle::None {
        true => {
            (point.x - 1, point.y - 1).into()
        },
        false => point
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
