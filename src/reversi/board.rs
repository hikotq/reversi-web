use std::fmt;
use std::ops::{Add, Deref};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn equal(color1: &Color, color2: &Color) -> bool {
        !(color1.is_black() ^ color2.is_black())
    }

    pub fn is_black(&self) -> bool {
        use self::Color;
        match *self {
            Color::Black => true,
            _ => false,
        }
    }

    pub fn is_white(&self) -> bool {
        !self.is_black()
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: &str = &s.to_uppercase();
        match s {
            "BLACK" => Ok(Color::Black),
            "WHITE" => Ok(Color::White),
            _ => Err("Invalid Color".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum Cell {
    Empty,
    Available,
    Piece(self::Color),
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        use self::Cell::Empty;
        match *self {
            Empty => true,
            _ => false,
        }
    }

    pub fn is_available(&self) -> bool {
        use self::Cell::Available;
        match *self {
            Available => true,
            _ => false,
        }
    }

    pub fn is_piece(&self) -> bool {
        use self::Cell::Piece;
        match *self {
            Piece(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Cell::*;
        let s = match self {
            Piece(color) => {
                if color.is_black() {
                    "black"
                } else {
                    "white"
                }
            }
            Available => "available",
            Empty => "empty",
        };
        write!(f, "{}", s.to_string())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Pos<T> {
    pub x: T,
    pub y: T,
}

impl Add for Pos<i32> {
    type Output = Option<Pos<usize>>;

    fn add(self, other: Pos<i32>) -> Self::Output {
        let x = self.x + other.x;
        let y = self.y + other.y;
        if x < 0 || x > 8 || y < 0 || y > 8 {
            return None;
        }
        let p = Pos {
            x: x as usize,
            y: y as usize,
        };
        Some(p)
    }
}

impl Add<Pos<i32>> for Pos<usize> {
    type Output = Option<Pos<usize>>;

    fn add(self, other: Pos<i32>) -> Self::Output {
        let x = self.x as i32 + other.x;
        let y = self.y as i32 + other.y;
        if x < 0 || x > 7 || y < 0 || y > 7 {
            return None;
        }
        let p = Pos {
            x: x as usize,
            y: y as usize,
        };
        Some(p)
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Move {
    pub x: usize,
    pub y: usize,
    pub color: Color,
}

impl Move {
    fn new(x: i32, y: i32, color: Color) -> Result<Self, String> {
        if x < 0 || x > 8 || y < 0 || y > 8 {
            return Err("Invalid Pos".to_string());
        }
        Ok(Move {
            x: x as usize,
            y: y as usize,
            color,
        })
    }
}

#[derive(Copy, Clone)]
pub struct Board([Cell; 64]);

impl Deref for Board {
    type Target = [Cell; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0[..].fmt(formatter)
    }
}

impl Board {
    pub fn new() -> Self {
        Board([self::Cell::Empty; 64])
    }

    pub fn all_pos() -> Vec<Pos<usize>> {
        let mut v = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                v.push(Pos { x, y });
            }
        }
        v
    }

    pub fn get_cell(&self, p: Pos<usize>) -> Cell {
        self.0[p.y * 8 + p.x]
    }

    pub fn set_cell(&mut self, p: Pos<usize>, cell: Cell) {
        self.0[p.y * 8 + p.x] = cell;
    }

    pub fn count_piece(&self) -> (usize, usize, usize, usize) {
        let mut black = 0;
        let mut white = 0;
        let mut available = 0;
        let mut empty = 0;
        for p in self.0.iter() {
            match p {
                Cell::Piece(color) => {
                    if color.is_black() {
                        black += 1;
                    } else {
                        white += 1;
                    }
                }
                Cell::Available => {
                    available += 1;
                }
                Cell::Empty => {
                    empty += 1;
                }
            }
        }
        (black, white, available, empty)
    }

    pub fn has_available_cell(&self) -> bool {
        self.0.contains(&Cell::Available)
    }

    pub fn show(&self) {
        for y in 0..8 {
            for x in 0..8 {
                let s = match self.get_cell(Pos { x, y }) {
                    Cell::Piece(Color::Black) => "○",
                    Cell::Piece(Color::White) => "●",
                    Cell::Empty => "□",
                    Cell::Available => "×",
                };
                print!("{}", s);
            }
            println!("");
        }
    }
}
