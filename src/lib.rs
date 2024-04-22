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

const KNOWN_KEYWORDS: [&'static str; 3] = ["LOK", "TLAK", "TA"];

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq)]
struct BoardCell {
    letter: Option<char>,
    is_blackened: bool,
}

type BoardGrid = Grid<BoardCell>;

#[wasm_bindgen]
impl BoardCell {
    pub fn is_interactive(&self) -> bool {
        self.letter.is_some()
    }

    pub fn is_blackened(&self) -> bool {
        self.is_blackened
    }

    pub fn get_display(&self) -> char {
        if let Some(ch) = self.letter {
            ch
        } else {
            ' '
        }
    }
}

impl BoardCell {
    fn gap() -> BoardCell {
        BoardCell {
            letter: None,
            is_blackened: false,
        }
    }

    fn raw(letter: char) -> BoardCell {
        assert!(letter.is_ascii());

        BoardCell {
            letter: match letter {
                '_' => None,
                _ => Some(letter.to_ascii_uppercase()),
            },
            is_blackened: false,
        }
    }

    fn blank() -> BoardCell {
        BoardCell::raw(' ')
    }

    fn is_done(&self) -> bool {
        self.letter.is_none() || self.is_blackened()
    }

    fn is_traversible(&self) -> bool {
        self.is_done()
    }

    fn get_letter(&self) -> Option<char> {
        match self.letter {
            None => None,
            Some(' ') => None,
            Some(ch) => Some(ch),
        }
    }

    fn get_raw(&self) -> char {
        self.letter.unwrap()
    }

    fn blacken(&mut self) {
        self.is_blackened = true;
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
    ExecutingTA(Option<char>),
}

impl BoardState {
    fn idle() -> BoardState {
        BoardState::GatheringKeyword(String::new(), vec![])
    }
}

struct BoardStep {
    mv: Move,
    grid: BoardGrid,
}

#[wasm_bindgen]
pub struct Board {
    grid: BoardGrid,
    moves: Vec<BoardStep>,
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
        self.get_latest()[&RC(row, col)].clone()
    }

    pub fn blacken(&mut self, row: usize, col: usize) -> bool {
        // TODO Probably this should be properly error handled.
        assert!(row < self.grid.height());
        assert!(col < self.grid.width());

        let target_rc = RC(row, col);
        let latest_grid = self.get_latest();
        if latest_grid[&target_rc].is_blackened() {
            log!("{:?} is already blackened", target_rc);
            return false;
        }

        let mut new_grid = latest_grid.clone();
        new_grid[&target_rc].blacken();

        self.moves.push(BoardStep {
            mv: Move::Blacken(target_rc.clone()),
            grid: new_grid,
        });

        true
    }

    pub fn undo(&mut self) {
        let _ = self.moves.pop();
    }

    pub fn commit_and_check_solution(&self) -> Option<usize> {
        let mut simgrid = self.grid.clone();
        let mut state = BoardState::idle();
        for (mv_num, BoardStep { mv: mv, grid: _ }) in self.moves.iter().enumerate() {
            log!("{:2}: state {:?}, move {:?}", mv_num, state, mv);

            state = match mv {
                Move::Blacken(target_rc) => {
                    let target = simgrid[target_rc].clone();

                    if target.is_blackened() {
                        log!("{:?} already blackened", target_rc);
                        return Some(mv_num);
                    }

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
                                        simgrid[rc].blacken();
                                    }

                                    match *known_keyword {
                                        "LOK" => BoardState::ExecutingLOK,
                                        "TLAK" => BoardState::ExecutingTLAK(vec![]),
                                        "TA" => BoardState::ExecutingTA(None),
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
                            simgrid[target_rc].blacken();
                            BoardState::idle()
                        }
                        BoardState::ExecutingTLAK(exec_rcs) => {
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

                            simgrid[target_rc].blacken();

                            if exec_rcs.len() == 1 {
                                BoardState::idle()
                            } else {
                                let mut next_exec_rcs = exec_rcs.clone();
                                next_exec_rcs.push(target_rc.clone());
                                BoardState::ExecutingTLAK(next_exec_rcs)
                            }
                        }
                        BoardState::ExecutingTA(chosen_letter_opt) => {
                            if let Some(letter) = target.get_letter() {
                                if let Some(chosen_letter) = chosen_letter_opt {
                                    if letter != chosen_letter {
                                        log!(
                                            "Letter {} does not match TA chosen letter {}",
                                            letter,
                                            chosen_letter
                                        );
                                        return Some(mv_num);
                                    }
                                } else {
                                    log!("TA choosing letter {}", letter);
                                }

                                simgrid[target_rc].blacken();

                                // If there are any more of this chosen letter on the board, then the state is still
                                // waiting for those to be blackened out. Otherwise, the TA is done.
                                let mut has_completed_all_letters = true;
                                for (rc, cell) in simgrid.enumerate_row_col() {
                                    if !cell.is_blackened() && cell.get_raw() == letter {
                                        log!("{:?} is still {}", rc, letter);
                                        has_completed_all_letters = false;
                                        break;
                                    }
                                }

                                if has_completed_all_letters {
                                    BoardState::idle()
                                } else {
                                    BoardState::ExecutingTA(Some(letter))
                                }
                            } else {
                                log!("Not a letter: {}", target.get_raw());
                                return Some(mv_num);
                            }
                        }
                    }
                }
            };
        }

        // Must be back in the idle state before considering the board to be done.
        if let BoardState::GatheringKeyword(keyword, _) = state {
            if !keyword.is_empty() {
                log!("Partial keyword {} found. Not done.", keyword);
                return Some(self.moves.len());
            }

            for (rc, cell) in simgrid.enumerate_row_col() {
                if !cell.is_done() {
                    log!("{:?} not done", rc);
                    return Some(self.moves.len());
                }
            }
        } else {
            log!("State {:?} is not idle", state);
            return Some(self.moves.len());
        }

        None
    }
}

impl Board {
    fn get_latest(&self) -> &BoardGrid {
        if let Some(step) = self.moves.last() {
            &step.grid
        } else {
            &self.grid
        }
    }

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
    fn partial_keyword() {
        let mut board = Board::new("L").unwrap();
        board.blacken(0, 0);
        assert_eq!(board.commit_and_check_solution(), Some(1));
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
    fn lok_unsolvable_cant_execute() {
        let mut board = Board::new("LOK").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        assert_eq!(board.commit_and_check_solution(), Some(3));
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
    fn tlak_cant_execute1() {
        let mut board = Board::new("TLAK").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(4));
    }

    #[test]
    fn tlak_cant_execute2() {
        let mut board = Board::new("TLAK ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        assert_eq!(board.commit_and_check_solution(), Some(5));
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

    #[test]
    fn ta_correct() {
        let mut board = Board::new("TA\nQQ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(1, 0);
        board.blacken(1, 1);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn ta_multiple_letters() {
        let mut board = Board::new("TA\nQZ").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(1, 0);
        board.blacken(1, 1);
        assert_eq!(board.commit_and_check_solution(), Some(3));
    }

    #[test]
    fn ta_unsolvable_no_exec() {
        let mut board = Board::new("TA__").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }
}
