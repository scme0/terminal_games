#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cell {
    pub x: i32,
    pub y: i32
}

impl From<Cell> for (i32, i32) {
    fn from(c: Cell) -> (i32, i32) {
        let Cell {x, y} = c;
        return (x, y);
    }
}
