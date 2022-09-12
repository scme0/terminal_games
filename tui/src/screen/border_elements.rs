use crate::screen::border_style::BorderStyle;

pub struct BorderElements {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub label_frame_left: char,
    pub label_frame_right: char
}

impl BorderElements {
    pub fn new(border_style: BorderStyle) -> Self {
        match border_style {
            BorderStyle::Double => BorderElements { top_left: '╔', top_right: '╗', bottom_left: '╚', bottom_right: '╝', horizontal: '═', vertical: '║', label_frame_left: '╡', label_frame_right: '╞'},
            BorderStyle::Single => BorderElements { top_left: '┏', top_right: '┓', bottom_left: '┗', bottom_right: '┛', horizontal: '━', vertical: '┃', label_frame_left: '┫', label_frame_right: '┣'},
            BorderStyle::Dotted => BorderElements { top_left: '┏', top_right: '┓', bottom_left: '┗', bottom_right: '┛', horizontal: '┅', vertical: '┇', label_frame_left: '╏', label_frame_right: '╏'},
            BorderStyle::None => BorderElements { top_left: '\0', top_right: '\0', bottom_left: '\0', bottom_right: '\0', horizontal: '\0', vertical: '\0', label_frame_left: '\0', label_frame_right: '\0'},
        }
    }
}
