#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Dimension {
    pub width: i32,
    pub height: i32
}

impl From<Dimension> for (i32, i32) {
    fn from(c: Dimension) -> (i32, i32) {
        let Dimension {width, height} = c;
        return (width, height);
    }
}

impl From<(i32, i32)> for Dimension {
    fn from(p: (i32, i32)) -> Self {
        Dimension {width: p.0, height: p.1}
    }
}