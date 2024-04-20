use wasm_bindgen::prelude::*;

extern crate web_sys;

mod grid;
mod utils;

use crate::grid::{Grid, RC};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        if cfg!(wasm32) {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        } else {
            println!( $( $t )* );
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq)]
struct BoardCell(u8);
type BoardGrid = Grid<BoardCell>;
// TODO: compile time assert that size of board cell is u8

impl BoardCell {
    fn gap() -> BoardCell {
        BoardCell('_' as u8)
    }

    fn blank() -> BoardCell {
        BoardCell(' ' as u8)
    }

    fn raw(c: char) -> BoardCell {
        assert!(c.is_ascii());
        BoardCell(c as u8)
    }

    fn blackened() -> BoardCell {
        BoardCell('*' as u8)
    }

    fn is_blackened(&self) -> bool {
        *self == BoardCell::blackened()
    }

    fn is_done(&self) -> bool {
        *self == BoardCell::gap() || self.is_blackened()
    }

    fn get_letter(&self) -> Option<char> {
        match self.get_raw() {
            ' ' | '*' => None,
            _ => Some(self.get_raw()),
        }
    }

    fn get_raw(&self) -> char {
        self.0 as char
    }
}

#[derive(Debug)]
enum Move {
    Blacken(RC),
}

#[derive(Debug)]
enum BoardState {
    Idle,
    L(RC),
    LO(RC, RC),
    LOK(RC, RC, RC),
}

#[wasm_bindgen]
pub struct Board {
    grid: BoardGrid,
    moves: Vec<Move>,
}

#[wasm_bindgen]
impl Board {
    pub fn new(rows: usize, cols: usize, contents: &str) -> Board {
        let mut board = Board {
            grid: Grid::new(cols, rows, &BoardCell::blank()),
            moves: vec![],
        };

        for (i, ch) in contents.chars().enumerate() {
            board.grid.cells_mut()[i] = BoardCell::raw(ch);
        }

        board
    }

    pub fn width(&self) -> u32 {
        self.grid.width() as u32
    }

    pub fn height(&self) -> u32 {
        self.grid.height() as u32
    }

    pub fn cells(&self) -> *const BoardCell {
        self.grid.cells().as_ptr()
    }

    pub fn blacken(&mut self, row: usize, col: usize) {
        // TODO Probably this should be properly error handled.
        assert!(row < self.grid.height());
        assert!(col < self.grid.width());

        self.moves.push(Move::Blacken(RC(row, col)));
    }

    fn is_connected_for_keyword(&self, rc1: &RC, rc2: &RC) -> bool {
        // TODO implement
        true
    }

    pub fn commit_and_check_solution(&self) -> Option<usize> {
        let mut simgrid = self.grid.clone();
        let mut state = BoardState::Idle;
        for (mv_num, mv) in self.moves.iter().enumerate() {
            log!("{:2}: state {:?}, move {:?}", mv_num, state, mv);

            match mv {
                Move::Blacken(target_rc) => {
                    let target = simgrid[target_rc].clone();
                    match state {
                        BoardState::Idle => {
                            if let Some(letter) = target.get_letter() {
                                match letter {
                                    'L' => {
                                        state = BoardState::L(target_rc.clone());
                                    }
                                    _ => {
                                        log!("Letter {} not valid", letter);
                                        return Some(mv_num);
                                    }
                                }
                            } else {
                                log!("Not a letter: {}", target.get_raw());
                                return Some(mv_num);
                            }
                        }
                        BoardState::L(rc_l) => {
                            if !self.is_connected_for_keyword(&rc_l, target_rc) {
                                log!("{:?} not connected to {:?} for keyword", rc_l, target_rc);
                                return Some(mv_num);
                            }

                            if let Some(letter) = target.get_letter() {
                                match letter {
                                    'O' => {
                                        state = BoardState::LO(rc_l.clone(), target_rc.clone());
                                    }
                                    _ => {
                                        log!("Letter {} not valid. Expected O", letter);
                                        return Some(mv_num);
                                    }
                                }
                            } else {
                                log!("Not a letter: {}", target.get_raw());
                                return Some(mv_num);
                            }
                        }
                        BoardState::LO(rc_l, rc_o) => {
                            if !self.is_connected_for_keyword(&rc_o, target_rc) {
                                log!("{:?} not connected to {:?} for keyword", rc_o, target_rc);
                                return Some(mv_num);
                            }

                            if let Some(letter) = target.get_letter() {
                                match letter {
                                    'K' => {
                                        state = BoardState::LOK(
                                            rc_l.clone(),
                                            rc_o.clone(),
                                            target_rc.clone(),
                                        );
                                        simgrid[&rc_l] = BoardCell::blackened();
                                        simgrid[&rc_o] = BoardCell::blackened();
                                        simgrid[target_rc] = BoardCell::blackened();
                                    }
                                    _ => {
                                        log!("Letter {} not valid. Expected K", letter);
                                        return Some(mv_num);
                                    }
                                }
                            } else {
                                log!("Not a letter: {}", target.get_raw());
                                return Some(mv_num);
                            }
                        }
                        BoardState::LOK(rc_l, rc_o, rc_k) => {
                            if target.is_blackened() {
                                log!("{:?} already blackened", target_rc);
                                return Some(mv_num);
                            }

                            simgrid[target_rc] = BoardCell::blackened();
                            state = BoardState::Idle;
                        }
                    }
                }
            }
        }

        for r in 0..simgrid.height() {
            for c in 0..simgrid.width() {
                let rc = RC(r, c);
                if !simgrid[&rc].is_done() {
                    log!("{:?} not done", rc);
                    return Some(self.moves.len());
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lok4_correct() {
        let mut board = Board::new(1, 4, "LOK ");
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert!(board.commit_and_check_solution().is_none());
    }

    #[test]
    fn lok5_unsolvable() {
        let mut board = Board::new(1, 5, "LOK  ");
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert!(board.commit_and_check_solution() == Some(4));
    }

    #[test]
    fn lok4_out_of_order_middle() {
        let mut board = Board::new(1, 4, "LOK ");
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert!(board.commit_and_check_solution() == Some(1));
    }

    #[test]
    fn lok4_out_of_order_backwards() {
        let mut board = Board::new(1, 4, "LOK ");
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 0);
        board.blacken(0, 3);
        assert!(board.commit_and_check_solution() == Some(0));
    }
}
