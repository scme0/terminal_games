pub mod button;
mod border_elements;
pub mod border_style;
pub mod component;
pub mod has_close_action;
pub mod mouse_action;
pub mod update_element;

use crossterm::Result;
use std::cmp::Ordering;
use std::fmt::Debug;
use uuid::Uuid;
use crate::screen::dimension::Dimension;
use crate::screen::point::Point;
use crate::screen::window::border_elements::BorderElements;
use crate::screen::window::border_style::BorderStyle;
use crate::screen::window::component::Component;
use crate::screen::window::has_close_action::HasCloseAndRefreshActions;
use crate::screen::window::mouse_action::MouseAction;
use crate::screen::window::update_element::UpdateElement;

#[derive(Debug)]
pub struct Window<T: HasCloseAndRefreshActions + PartialEq + Clone> {
    pub id: Uuid,
    pub z: i32,
    pub location: Point,
    border_style: BorderStyle,
    border_title: Box<str>,
    component: Box<dyn Component<T>>,
    pub refresh: bool,
    pub can_move: bool,
    close_point: Option<Point>,
}

impl<T: HasCloseAndRefreshActions + PartialEq + Clone> Window<T> {
    pub fn new(
        location: Point,
        z: i32,
        component: Box<dyn Component<T>>,
        border_style: BorderStyle,
        border_title: Box<str>,
        can_move: bool,
        can_close: bool
    ) -> Self {
        let id = component.get_id();
        let width =
            Window::<T>::get_window_size(component.get_size(), border_style).width;

        let close_point = match can_close {
            true => Some((width - 4, 0).into()),
            false => None,
        };

        return Window {
            id,
            location,
            z,
            border_style,
            border_title,
            component,
            can_move,
            refresh: true,
            close_point
        };
    }

    fn get_window_size(component_size: Dimension, border_style: BorderStyle) -> Dimension{
        let (mut width,mut height) = component_size.into();
        if border_style != BorderStyle::None {
            width += 4;
            height += 1;
        }
        return (width, height).into();
    }

    fn draw_border(&self) -> Result<Vec<UpdateElement>> {
        let border_elements = BorderElements::new(self.border_style);
        let mut updates = vec![];
        let mut title = self.border_title.clone();
        let mut title_len = title.chars().count() as i32;
        let size = self.get_size();
        if title_len >= size.width {
            title = Box::from(&title[..(size.width - 2) as usize]);
            title_len = size.width - 1;
        }
        let top_left:Point = (0, 0).into();
        let b_top_left = top_left + (1,0).into();
        let top_right:Point = (size.width, 0).into();
        let b_top_right = top_right + (-2,0).into();
        let bottom_left:Point = (0, size.height).into();
        let b_bottom_left = bottom_left + (1,0).into();
        let bottom_right:Point = (size.width, size.height).into();
        let b_bottom_right = bottom_right + (-2,0).into();
        for y in top_left.y..bottom_left.y+1 {
            updates.push(UpdateElement {point: (top_left.x, y).into(), value: ' ', fg: None});
        }
        for y in top_right.y..bottom_right.y+1 {
            updates.push(UpdateElement {point: (top_right.x-1, y).into(), value: ' ', fg: None});
        }

        if b_top_left.y >= 0 {
            if b_top_left.x >= 0 {
                // draw top_left corner.
                updates.push(UpdateElement {point: b_top_left, value: border_elements.top_left, fg: None});
            }
            // draw top_right corner.
            updates.push(UpdateElement {point: b_top_right, value: border_elements.top_right, fg: None});

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
                updates.push(UpdateElement {point: (b_top_left.x + 1, b_top_left.y).into(), value: border_elements.label_frame_left, fg: None});
                // draw title
                for x in b_top_left.x + 2..b_top_left.x + 2 + title_len {
                    updates.push(UpdateElement {point: (x, b_top_left.y).into(), value: title.chars().nth(x as usize - 3).unwrap(), fg: None});
                }
                // draw post-title char
                updates.push(UpdateElement {point: (b_top_left.x + 2 + title_len, b_top_left.y).into(), value: border_elements.label_frame_right, fg: None});
            }
            // draw from top_left to top_right.
            for x in b_top_left.x + top_line_offset..b_top_right.x - top_line_right_offset {
                updates.push(UpdateElement {point: (x, b_top_left.y).into(), value: border_elements.horizontal, fg: None});
            }
        }
        if top_left.x >= 0 {
            // draw bottom_left corner.
            updates.push(UpdateElement {point: b_bottom_left.into(), value: border_elements.bottom_left, fg: None});
            // draw from top_left to bottom_left.
            for y in (b_top_left.y + 1)..b_bottom_left.y {
                updates.push(UpdateElement {point: (b_top_left.x, y).into(), value: border_elements.vertical, fg: None});
            }
        }
        // draw bottom_right corner.
        updates.push(UpdateElement {point: b_bottom_right.into(), value: border_elements.bottom_right, fg: None});
        // draw from bottom_left to bottom_right
        for x in (b_bottom_left.x + 1)..b_bottom_right.x {
            updates.push(UpdateElement {point: (x, b_bottom_left.y).into(), value: border_elements.horizontal, fg: None});
        }
        // draw from top_right to bottom_right
        for y in (b_top_right.y + 1)..b_bottom_right.y {
            updates.push(UpdateElement {point: (b_top_right.x, y).into(), value: border_elements.vertical, fg: None});
        }
        Ok(updates)
    }
    fn get_updates_or_state(&mut self, updates_getter: fn (&mut Box<dyn Component<T>>) -> Result<Vec<UpdateElement>>) -> Result<Vec<UpdateElement>> {
        let mut updates = match self.border_style != BorderStyle::None {
            true => self.draw_border()?,
            false => vec![],
        };

        for update in updates_getter(&mut self.component)?.iter() {
            let point = match self.border_style != BorderStyle::None {
                true => Point{x: update.point.x + 2, y: update.point.y + 1 },
                false => update.point,
            };
            updates.push(UpdateElement {point, value: update.value, fg: update.fg });
        }

        self.refresh = false;

        return Ok(updates);
    }
}

impl<T: HasCloseAndRefreshActions + PartialEq + Clone> Component<T> for Window<T> {
    fn get_id(&self) -> Uuid {
        self.component.get_id()
    }

    fn get_size(&self) -> Dimension {
        Window::<T>::get_window_size(self.component.get_size().clone(), self.border_style)
    }

    fn get_state(&mut self) -> Result<Vec<UpdateElement>> {
        self.get_updates_or_state(|c| c.get_state())
    }

    fn get_updates(&mut self) -> Result<Vec<UpdateElement>> {
        self.get_updates_or_state(|c| c.get_updates())
    }

    fn handle_click(&mut self, mouse_action: MouseAction) -> Result<Vec<T>> {
        let size = self.get_size();
        let action_point = mouse_action.to_point();
        if self.border_style != BorderStyle::None &&
            (action_point.x == 0 || action_point.x == 1 || action_point.x == size.width - 1 || action_point.x == size.width - 2
                || action_point.y == 0 || action_point.y == size.height){
            match mouse_action {
                MouseAction::Left(_) => {
                    if let Some(close_point) = self.close_point {
                        if action_point == close_point || action_point == close_point + (1,0).into() {
                            return Ok(vec![T::get_close_action(self.get_id())])
                        }
                    }
                }
                MouseAction::Drag(starting_point, drag_point) => {
                    if self.can_move {
                        let movement_vector = drag_point - starting_point;
                        let mut new_x = self.location.x + movement_vector.x;
                        let mut new_y = self.location.y + movement_vector.y;
                        if new_x < 0 {
                            new_x = 0;
                        }

                        if new_y < 0 {
                            new_y = 0;
                        }

                        if self.location.x != new_x || self.location.y != new_y {
                            self.location.x = new_x;
                            self.location.y = new_y;
                            self.refresh = true;
                        }
                    }
                    return Ok(vec![]);
                }
                _ => {}
            }
        } else {
            let rel_point = calculate_relative_x_y(self, action_point);
            let click_actions = match mouse_action {
                MouseAction::Middle(_) => self.component.handle_click(MouseAction::Middle(rel_point))?,
                MouseAction::Left(_) => self.component.handle_click(MouseAction::Left(rel_point))?,
                MouseAction::Right(_) => self.component.handle_click(MouseAction::Right(rel_point))?,
                MouseAction::Double(_) => self.component.handle_click(MouseAction::Double(rel_point))?,
                MouseAction::Move(_) => self.component.handle_click(MouseAction::Move(rel_point))?,
                MouseAction::Drag(_, vector) => self.component.handle_click(MouseAction::Drag(rel_point, vector))?,
            };
            if click_actions.contains(&T::get_refresh_action()) {
                self.refresh = true;
            }
            return Ok(click_actions);
        }
        Ok(vec![])
    }
}

fn calculate_relative_x_y<T: HasCloseAndRefreshActions + PartialEq + Clone>(window: &Window<T>, point: Point) -> Point{
    match window.border_style != BorderStyle::None {
        true => {
            (point.x - 2, point.y - 1).into()
        },
        false => point
    }
}

impl<T: HasCloseAndRefreshActions + PartialEq + Clone> PartialEq for Window<T> {
    fn eq(&self, other: &Self) -> bool {
        return other.id == self.id;
    }
}

impl<T: HasCloseAndRefreshActions + PartialEq + Clone> PartialOrd for Window<T> {
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
