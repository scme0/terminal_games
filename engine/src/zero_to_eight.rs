use crossterm::ErrorKind;
use std::io;
use crate::{Eight, Five, Four, One, Seven, Six, Three, Two, Zero};

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum ZeroToEight {
    #[default]
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
}

impl ZeroToEight {
    pub fn from_u8(number: u8) -> crossterm::Result<ZeroToEight> {
        let bombs = match number {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            _ => Err(ErrorKind::new(io::ErrorKind::Other, "This number cannot be an adjacent number of bombs!"))?
        };
        Ok(bombs)
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Zero => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8
        }
    }
}
