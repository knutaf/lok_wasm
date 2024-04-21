use wasm_bindgen::prelude::*;

extern crate web_sys;

mod grid;
mod utils;

use crate::grid::{Grid, RC};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        if cfg!(target_family = "wasm") {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        } else {
            println!( $( $t )* );
        }
    }
}

const KNOWN_KEYWORDS: [&'static str; 2] = ["LOK", "TLAK"];

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq)]
struct BoardCell(u8);
type BoardGrid = Grid<BoardCell>;
// TODO: compile time assert that size of board cell is u8

#[wasm_bindgen]
impl BoardCell {
    pub fn is_interactive(&self) -> bool {
        *self != BoardCell::gap()
    }

    pub fn get_display(&self) -> char {
        match self.get_raw() {
            '_' | ' ' | '*' => ' ',
            _ => self.get_letter().unwrap(),
        }
    }
}

impl BoardCell {
    fn gap() -> BoardCell {
        BoardCell('_' as u8)
    }

    fn blank() -> BoardCell {
        BoardCell(' ' as u8)
    }

    fn raw(c: char) -> BoardCell {
        assert!(c.is_ascii());
        BoardCell(c.to_ascii_uppercase() as u8)
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

#[derive(Clone, Debug)]
enum BoardState {
    GatheringKeyword(String, Vec<RC>),
    ExecutingLOK,
    ExecutingTLAK(Vec<RC>),
}

impl BoardState {
    fn idle() -> BoardState {
        BoardState::GatheringKeyword(String::new(), vec![])
    }
}

#[wasm_bindgen]
pub struct Board {
    grid: BoardGrid,
    moves: Vec<Move>,
}

#[wasm_bindgen]
impl Board {
    pub fn new(contents: &str) -> Option<Board> {
        log!("puzzle: {}", contents);

        let mut rows = 0;
        let mut cols = 0;
        for line in contents.lines() {
            log!("row {}: {}", rows, line);
            if cols == 0 {
                cols = line.len();
            }

            if line.len() != cols {
                return None;
            }

            rows += 1;
        }

        let mut board = Board {
            grid: Grid::new(cols, rows, &BoardCell::blank()),
            moves: vec![],
        };

        let mut row = 0;
        for line in contents.lines() {
            let mut col = 0;
            for ch in line.chars() {
                board.grid[&RC(row, col)] = BoardCell::raw(ch);
                col += 1;
            }

            row += 1;
        }

        Some(board)
    }

    pub fn width(&self) -> u32 {
        self.grid.width() as u32
    }

    pub fn height(&self) -> u32 {
        self.grid.height() as u32
    }

    pub fn get(&self, row: usize, col: usize) -> BoardCell {
        self.grid[&RC(row, col)].clone()
    }

    pub fn blacken(&mut self, row: usize, col: usize) {
        // TODO Probably this should be properly error handled.
        assert!(row < self.grid.height());
        assert!(col < self.grid.width());

        self.moves.push(Move::Blacken(RC(row, col)));
    }

    pub fn commit_and_check_solution(&self) -> Option<usize> {
        let mut simgrid = self.grid.clone();
        let mut state = BoardState::idle();
        for (mv_num, mv) in self.moves.iter().enumerate() {
            log!("{:2}: state {:?}, move {:?}", mv_num, state, mv);

            state = match mv {
                Move::Blacken(target_rc) => {
                    let target = simgrid[target_rc].clone();

                    match state {
                        BoardState::GatheringKeyword(keyword, keyword_rcs) => {
                            // If this is not the first letter in this keyword, check to make sure the new one is
                            // connected to the most recent letter that was accepted.
                            if let Some(last_rc) = keyword_rcs.last() {
                                if !Board::is_connected_for_keyword(
                                    &simgrid,
                                    keyword_rcs.last().unwrap(),
                                    target_rc,
                                ) {
                                    log!(
                                        "{:?} not connected to {:?} for keyword",
                                        last_rc,
                                        target_rc
                                    );
                                    return Some(mv_num);
                                }
                            }

                            // Keywords consist of only letters.
                            if let Some(letter) = target.get_letter() {
                                let mut new_keyword = keyword.clone();
                                new_keyword.push(letter);

                                // Check to see if the keyword gathered so far could possibly be one of the known
                                // keywords. If not, the solution fails here.
                                if !KNOWN_KEYWORDS
                                    .iter()
                                    .any(|known_keyword| known_keyword.starts_with(&new_keyword))
                                {
                                    log!("{} cannot be any known keyword", new_keyword);
                                    return Some(mv_num);
                                }

                                // So far this is a possible keyword, so accept the RC of the latest letter.
                                let mut new_keyword_rcs = keyword_rcs.clone();
                                new_keyword_rcs.push(target_rc.clone());

                                // If the keyword so far matches a known keyword, then accept it and transition to the
                                // executing state. Otherwise, continue gathering.
                                if let Some(known_keyword) = KNOWN_KEYWORDS
                                    .iter()
                                    .find(|known_keyword| new_keyword == **known_keyword)
                                {
                                    // Have now accumulated a whole keyword. Black it out.
                                    for rc in new_keyword_rcs.iter() {
                                        simgrid[rc] = BoardCell::blackened();
                                    }

                                    match *known_keyword {
                                        "LOK" => BoardState::ExecutingLOK,
                                        "TLAK" => BoardState::ExecutingTLAK(vec![]),
                                        _ => {
                                            panic!("Impossible unknown keyword {}", *known_keyword)
                                        }
                                    }
                                } else {
                                    BoardState::GatheringKeyword(new_keyword, new_keyword_rcs)
                                }
                            } else {
                                log!("Not a letter: {}", target.get_raw());
                                return Some(mv_num);
                            }
                        }
                        BoardState::ExecutingLOK => {
                            if target.is_blackened() {
                                log!("{:?} already blackened", target_rc);
                                return Some(mv_num);
                            }

                            simgrid[target_rc] = BoardCell::blackened();
                            BoardState::idle()
                        }
                        BoardState::ExecutingTLAK(exec_rcs) => {
                            if target.is_blackened() {
                                log!("{:?} already blackened", target_rc);
                                return Some(mv_num);
                            }

                            if let Some(last_exec_rc) = exec_rcs.last() {
                                if !Board::is_adjacent(&simgrid, last_exec_rc, target_rc) {
                                    log!(
                                        "{:?} not adjacent to {:?} for TLAK blacken",
                                        last_exec_rc,
                                        target_rc
                                    );

                                    return Some(mv_num);
                                }
                            }

                            simgrid[target_rc] = BoardCell::blackened();

                            if exec_rcs.len() == 1 {
                                BoardState::idle()
                            } else {
                                let mut next_exec_rcs = exec_rcs.clone();
                                next_exec_rcs.push(target_rc.clone());
                                BoardState::ExecutingTLAK(next_exec_rcs)
                            }
                        }
                    }
                }
            };
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

impl Board {
    fn is_connected_for_keyword(grid: &BoardGrid, rc1: &RC, rc2: &RC) -> bool {
        // TODO this probably needs to change when I add conductors
        Self::is_adjacent(grid, rc1, rc2)
    }

    fn is_adjacent(grid: &BoardGrid, rc1: &RC, rc2: &RC) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lok1x4_correct() {
        let mut board = Board::new("LOK ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn lok1x4_correct_non_blank() {
        let mut board = Board::new("LOKQ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn lok1x4_jump_gap() {
        let mut board = Board::new("LO_K_ ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 3);
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn lok_correct_jump_blackened() {
        let mut board = Board::new("LO KLOK ").unwrap();
        board.blacken(0, 4);
        board.blacken(0, 5);
        board.blacken(0, 6);
        board.blacken(0, 2);

        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 3);
        board.blacken(0, 7);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn lok1x5_unsolvable_extra_space() {
        let mut board = Board::new("LOK  ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(4));
    }

    #[test]
    fn lok1x5_unsolvable_out_of_order() {
        let mut board = Board::new("LKO ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn lok1x4_out_of_order_middle() {
        let mut board = Board::new("LOK ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn lok1x4_out_of_order_backwards() {
        let mut board = Board::new("LOK ").unwrap();
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 0);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(0));
    }

    #[test]
    fn lok2x4_correct() {
        // TODO find a prettier way to write these boards
        let mut board = Board::new("LOK \nLOK ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(1, 3);
        board.blacken(1, 0);
        board.blacken(1, 1);
        board.blacken(1, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn lok2x4_illegal_diagonal() {
        // TODO find a prettier way to write these boards
        let mut board = Board::new("LOK \nLOK ").unwrap();
        board.blacken(0, 0);
        board.blacken(1, 1);
        board.blacken(1, 2);
        board.blacken(1, 3);
        board.blacken(1, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn tlak_correct() {
        let mut board = Board::new("TLAK  ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn tlak_wrong_k() {
        let mut board = Board::new("TLAZ  ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), Some(3));
    }

    #[test]
    fn tlak_correct_non_blank() {
        let mut board = Board::new("TLAKQQ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), None);
    }
}
