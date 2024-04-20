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

    fn is_traversible(&self) -> bool {
        self.is_done()
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

    fn is_connected_for_keyword(grid: &BoardGrid, rc1: &RC, rc2: &RC) -> bool {
        assert_ne!(rc1, rc2);

        // Must be either vertically or horizontally aligned
        if rc1.0 != rc2.0 && rc1.1 != rc2.1 {
            return false;
        }

        let row_walk_inc: isize = rc2.0.cmp(&rc1.0) as i8 as isize;
        let col_walk_inc: isize = rc2.1.cmp(&rc1.1) as i8 as isize;
        assert!(row_walk_inc == 0 || col_walk_inc == 0);

        log!(
            "Walk from {:?} to {:?}, using ({}, {})",
            rc1,
            rc2,
            row_walk_inc,
            col_walk_inc
        );

        let mut current_rc = rc1.clone();
        loop {
            assert!(row_walk_inc >= 0 || current_rc.0 > 0);
            assert!(col_walk_inc >= 0 || current_rc.1 > 0);
            current_rc = RC(
                current_rc.0.checked_add_signed(row_walk_inc).unwrap(),
                current_rc.1.checked_add_signed(col_walk_inc).unwrap(),
            );

            assert!(current_rc.0 < grid.height());
            assert!(current_rc.1 < grid.width());

            if current_rc == *rc2 {
                return true;
            }

            let current = grid[&current_rc];
            if !current.is_traversible() {
                log!(
                    "Not connected: {:?} is not available for traversal",
                    current_rc
                );
                return false;
            }
        }

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
                            if !Board::is_connected_for_keyword(&simgrid, &rc_l, target_rc) {
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
                            if !Board::is_connected_for_keyword(&simgrid, &rc_o, target_rc) {
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

        for (rc, cell) in simgrid.enumerate_row_col() {
            if !cell.is_done() {
                log!("{:?} not done", rc);
                return Some(self.moves.len());
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
    fn lok5_unsolvable_extra_space() {
        let mut board = Board::new(1, 5, "LOK  ");
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(4));
    }

    #[test]
    fn lok5_unsolvable_out_of_order() {
        let mut board = Board::new(1, 4, "LKO ");
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn lok4_out_of_order_middle() {
        let mut board = Board::new(1, 4, "LOK ");
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn lok4_out_of_order_backwards() {
        let mut board = Board::new(1, 4, "LOK ");
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 0);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(0));
    }
}
