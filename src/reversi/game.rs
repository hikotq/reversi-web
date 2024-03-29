use reversi::board::{Board, Cell, Color, Move, Pos};

const DIR: [Pos<i32>; 8] = [
    Pos { x: 1, y: 0 },
    Pos { x: 1, y: 1 },
    Pos { x: 0, y: 1 },
    Pos { x: -1, y: 1 },
    Pos { x: -1, y: 0 },
    Pos { x: -1, y: -1 },
    Pos { x: 0, y: -1 },
    Pos { x: 1, y: -1 },
];

pub type Winner = Option<Color>;

#[derive(Clone, Debug)]
pub struct Game {
    pub board: Board,
    pub turn: Color,
    pub is_start: bool,
    pub is_over: bool,
    pub pass: bool,
}

impl Default for Game {
    fn default() -> Self {
        let mut board = Board::new();
        board.set_cell(Pos { x: 3, y: 3 }, Cell::Piece(Color::White));
        board.set_cell(Pos { x: 4, y: 4 }, Cell::Piece(Color::White));
        board.set_cell(Pos { x: 3, y: 4 }, Cell::Piece(Color::Black));
        board.set_cell(Pos { x: 4, y: 3 }, Cell::Piece(Color::Black));

        let mut game = Self {
            board: board,
            turn: Color::Black,
            is_start: false,
            is_over: false,
            pass: false,
        };
        game.update_available_cell();
        game
    }
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn change_turn(&mut self) {
        self.turn = match self.turn {
            Color::Black => Color::White,
            _ => Color::Black,
        };
        self.update_available_cell();
    }

    pub fn winner(&self) -> Winner {
        if !self.is_over {
            return None;
        }
        let (black, white, _, _) = self.board.count_piece();
        if black > white {
            Some(Color::Black)
        } else if white > black {
            Some(Color::White)
        } else {
            None
        }
    }

    pub fn put_piece(&mut self, m: Move) -> Result<(), String> {
        if !Color::equal(&self.turn, &m.color) {
            return Err(format!("It's not {:?} turn", m.color));
        }
        if !self.board.get_cell(Pos { x: m.x, y: m.y }).is_available() {
            return Err("Not Available Cell".to_string());
        }
        let pos = Pos { x: m.x, y: m.y };
        self.board.set_cell(pos, Cell::Piece(m.color));
        self.flip(pos);
        Ok(())
    }

    pub fn update_available_cell(&mut self) {
        for pos in Board::all_pos()
            .into_iter()
            .filter(|&p| !self.board.get_cell(p).is_piece())
            .collect::<Vec<Pos<usize>>>()
        {
            if self.can_put(pos, self.turn) {
                self.board.set_cell(pos, Cell::Available);
            } else if self.board.get_cell(pos).is_available() {
                self.board.set_cell(pos, Cell::Empty);
            }
        }
        let (_, _, available, _) = self.board.count_piece();
        println!("{}", available);
        if available == 0 {
            if self.pass {
                self.is_over = true;
            } else {
                self.pass = true;
                self.change_turn();
            }
        } else {
            self.pass = false;
        }
    }

    pub fn can_put(&self, pos: Pos<usize>, turn: Color) -> bool {
        for d in &DIR {
            let mut dir_p = pos + *d;
            if let Some(p) = dir_p {
                match self.board.get_cell(p) {
                    Cell::Piece(color) => {
                        if Color::equal(&turn, &color) {
                            continue;
                        }
                    }
                    _ => continue,
                }
                dir_p = p + *d;
            } else {
                continue;
            }
            while let Some(p) = dir_p {
                if let Cell::Piece(color) = self.board.get_cell(p) {
                    if Color::equal(&turn, &color) {
                        return true;
                    }
                } else {
                    break;
                }
                dir_p = p + *d;
            }
        }
        false
    }

    pub fn flip(&mut self, pos: Pos<usize>) {
        for d in &DIR {
            let p = pos + *d;
            if p.is_none() {
                continue;
            }
            if let Cell::Piece(color) = self.board.get_cell(p.unwrap()) {
                if Color::equal(&self.turn, &color) {
                    continue;
                }
            }
            self.flip_recursive(p, *d).ok();
        }
    }

    pub fn flip_recursive(&mut self, pos: Option<Pos<usize>>, d: Pos<i32>) -> Result<(), String> {
        if let Some(pos) = pos {
            if let Cell::Piece(color) = self.board.get_cell(pos) {
                if Color::equal(&self.turn, &color) {
                    return Ok(());
                } else {
                    if self.flip_recursive(pos + d, d).is_ok() {
                        self.board.set_cell(pos, Cell::Piece(self.turn));
                        return Ok(());
                    }
                }
            }
        }
        Err("out of board".to_string())
    }
}
