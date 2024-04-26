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

const KNOWN_KEYWORDS: [&'static str; 4] = ["LOK", "TLAK", "TA", "BE"];
const WILDCARD_LETTER: char = '?';

#[wasm_bindgen]
#[derive(Copy, Clone, PartialEq)]
pub struct BoardCell {
    letter: Option<char>,
    is_blackened: bool,
    is_marked_for_path: bool,
    was_ever_wildcard: bool,
    mark_count: u32,
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

    pub fn is_marked_for_path(&self) -> bool {
        self.is_marked_for_path
    }

    pub fn get_display(&self) -> char {
        self.get_letter().unwrap_or(' ')
    }

    pub fn get_mark_count(&self) -> u32 {
        self.mark_count
    }
}

impl BoardCell {
    fn raw(letter: char) -> BoardCell {
        assert!(letter.is_ascii());

        BoardCell {
            letter: match letter {
                '-' => None,
                _ => Some(letter.to_ascii_uppercase()),
            },
            was_ever_wildcard: letter == WILDCARD_LETTER,
            is_blackened: false,
            is_marked_for_path: false,
            mark_count: 0,
        }
    }

    fn blank() -> BoardCell {
        BoardCell::raw('_')
    }

    fn is_blank(&self) -> bool {
        match self.letter {
            Some('_') => true,
            _ => false,
        }
    }

    fn is_done(&self) -> bool {
        self.letter.is_none() || self.is_blackened()
    }

    fn is_traversible_for_adjacency(&self) -> bool {
        self.is_done()
    }

    fn is_traversible_for_keyword(&self) -> bool {
        self.is_traversible_for_adjacency() || self.is_conductor()
    }

    fn is_conductor(&self) -> bool {
        !self.is_blackened() && self.get_raw() == 'X'
    }

    fn was_ever_wildcard(&self) -> bool {
        self.was_ever_wildcard
    }

    fn get_letter(&self) -> Option<char> {
        match self.letter {
            None => None,
            Some('_') => None,
            Some(ch) => Some(ch),
        }
    }

    fn get_letter_or_blank(&self) -> Option<char> {
        match self.letter {
            None => None,
            Some(ch) => Some(ch),
        }
    }

    fn get_raw(&self) -> char {
        self.letter.unwrap()
    }

    fn blacken(&mut self) {
        self.is_blackened = true;
        self.mark_count += 1;
    }

    fn mark_path(&mut self) {
        self.is_marked_for_path = true;
        self.mark_count += 1;
    }

    fn change_letter(&mut self, letter: char) -> bool {
        match letter {
            '-' | '_' => false,
            _ => {
                self.letter = Some(letter.to_ascii_uppercase());
                if letter == WILDCARD_LETTER {
                    self.was_ever_wildcard = true;
                }
                true
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Move {
    Blacken(RC),
    MarkPath(RC),
    ChangeLetter(RC, char),
}

impl Move {
    fn get_rc(&self) -> &RC {
        match &self {
            Move::Blacken(rc) => rc,
            Move::MarkPath(rc) => rc,
            Move::ChangeLetter(rc, _) => rc,
        }
    }
}

#[derive(Clone, Debug)]
enum BoardState {
    GatheringKeyword(String, Vec<Move>),
    ExecutingLOK,
    ExecutingTLAK(Vec<RC>),
    ExecutingTA(Option<char>),
    ExecutingBE,
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
        log!("puzzle:\n{}", contents);

        let mut rows = 0;
        let mut cols = 0;
        for line in contents.lines() {
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

    pub fn blacken(&mut self, row: usize, col: usize) {
        assert!(row < self.grid.height());
        assert!(col < self.grid.width());

        let target_rc = RC(row, col);
        let mut new_grid = self.get_latest().clone();
        new_grid[&target_rc].blacken();

        self.moves.push(BoardStep {
            mv: Move::Blacken(target_rc.clone()),
            grid: new_grid,
        });
    }

    pub fn mark_path(&mut self, row: usize, col: usize) {
        assert!(row < self.grid.height());
        assert!(col < self.grid.width());

        let target_rc = RC(row, col);
        let mut new_grid = self.get_latest().clone();
        new_grid[&target_rc].mark_path();

        self.moves.push(BoardStep {
            mv: Move::MarkPath(target_rc.clone()),
            grid: new_grid,
        });
    }

    pub fn change_letter(&mut self, row: usize, col: usize, letter: char) {
        assert!(row < self.grid.height());
        assert!(col < self.grid.width());

        let target_rc = RC(row, col);
        let mut new_grid = self.get_latest().clone();
        if !new_grid[&target_rc].change_letter(letter) {
            return;
        }

        self.moves.push(BoardStep {
            mv: Move::ChangeLetter(target_rc.clone(), letter),
            grid: new_grid,
        });
    }

    pub fn undo(&mut self) {
        let _ = self.moves.pop();
    }

    pub fn commit_and_check_solution(&self) -> Option<usize> {
        let mut simgrid = self.grid.clone();
        let mut state = BoardState::idle();
        for (mv_num, BoardStep { mv, grid: _ }) in self.moves.iter().enumerate() {
            log!("{:2}: state {:?}, move {:?}", mv_num, state, mv);

            let target = simgrid[mv.get_rc()].clone();
            if target.is_blackened() {
                log!("{:?} already blackened", mv.get_rc());
                return Some(mv_num);
            }

            state = match mv {
                Move::Blacken(target_rc) => {
                    match state {
                        BoardState::GatheringKeyword(keyword, keyword_moves) => {
                            if !Board::is_connected_for_keyword(&simgrid, &keyword_moves, target_rc)
                            {
                                log!("{:?} not connected to previous keyword move", target_rc);
                                return Some(mv_num);
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
                                let mut new_keyword_moves = keyword_moves.clone();
                                new_keyword_moves.push(mv.clone());

                                // If the keyword so far matches a known keyword, then accept it and transition to the
                                // executing state. Otherwise, continue gathering.
                                if let Some(known_keyword) = KNOWN_KEYWORDS
                                    .iter()
                                    .find(|known_keyword| new_keyword == **known_keyword)
                                {
                                    // Have now accumulated a whole keyword. Black it out.
                                    for mv in new_keyword_moves.iter() {
                                        if let Move::Blacken(rc) = mv {
                                            simgrid[rc].blacken();
                                        }
                                    }

                                    match *known_keyword {
                                        "LOK" => BoardState::ExecutingLOK,
                                        "TLAK" => BoardState::ExecutingTLAK(vec![]),
                                        "TA" => BoardState::ExecutingTA(None),
                                        "BE" => BoardState::ExecutingBE,
                                        _ => {
                                            panic!("Impossible unknown keyword {}", *known_keyword)
                                        }
                                    }
                                } else {
                                    BoardState::GatheringKeyword(new_keyword, new_keyword_moves)
                                }
                            } else {
                                log!("Not a letter: {}", target.get_raw());
                                return Some(mv_num);
                            }
                        }
                        BoardState::ExecutingLOK => {
                            assert!(!target.is_blackened());
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

                            assert!(!target.is_blackened());
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
                            if let Some(letter) = target.get_letter_or_blank() {
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

                                assert!(!target.is_blackened());
                                simgrid[target_rc].blacken();

                                // If there are any more of this chosen letter on the board, then the state is still
                                // waiting for those to be blackened out. Otherwise, the TA is done.
                                let mut has_completed_all_letters = true;
                                for (rc, cell) in simgrid.enumerate_row_col() {
                                    if cell.is_blackened() {
                                        continue;
                                    }

                                    if let Some(cell_letter) = cell.get_letter_or_blank() {
                                        if cell_letter == letter {
                                            log!("{:?} is still {}", rc, letter);
                                            has_completed_all_letters = false;
                                            break;
                                        }
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
                        BoardState::ExecutingBE => {
                            log!("Cannot blacken while executing BE");
                            return Some(mv_num);
                        }
                    }
                }
                Move::MarkPath(target_rc) => match state {
                    BoardState::GatheringKeyword(keyword, keyword_moves) => {
                        if !Board::is_connected_for_keyword(&simgrid, &keyword_moves, target_rc) {
                            log!("{:?} not connected to previous keyword move", target_rc);
                            return Some(mv_num);
                        }

                        let mut new_keyword_moves = keyword_moves.clone();
                        new_keyword_moves.push(mv.clone());
                        BoardState::GatheringKeyword(keyword.clone(), new_keyword_moves)
                    }
                    BoardState::ExecutingLOK
                    | BoardState::ExecutingTLAK(_)
                    | BoardState::ExecutingTA(_)
                    | BoardState::ExecutingBE => {
                        log!("Cannot mark path while executing a keyword");
                        return Some(mv_num);
                    }
                },
                Move::ChangeLetter(target_rc, letter) => match state {
                    BoardState::GatheringKeyword(_, _)
                    | BoardState::ExecutingLOK
                    | BoardState::ExecutingTLAK(_)
                    | BoardState::ExecutingTA(_) => {
                        if target.was_ever_wildcard() {
                            if !simgrid[target_rc].change_letter(*letter) {
                                log!("Not allowed to change letter to '{}'", letter);
                                return Some(mv_num);
                            }

                            state
                        } else {
                            log!(
                                "Not allowed to change this cell's letter in state {:?}",
                                state
                            );
                            return Some(mv_num);
                        }
                    }
                    BoardState::ExecutingBE => {
                        if !target.is_blank() {
                            log!(
                                "Not allowed to change letter in non-blank cell: {:?}",
                                target.get_letter()
                            );
                            return Some(mv_num);
                        }

                        if !simgrid[target_rc].change_letter(*letter) {
                            log!("Not allowed to change letter to '{}'", letter);
                            return Some(mv_num);
                        }

                        BoardState::idle()
                    }
                },
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

    fn is_adjacent(grid: &BoardGrid, rc1: &RC, rc2: &RC) -> bool {
        if rc1 == rc2 {
            return false;
        }

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
            if !current.is_traversible_for_adjacency() {
                log!(
                    "Not connected: {:?} is not available for adjacency traversal",
                    current_rc
                );
                return false;
            }
        }
    }

    fn is_connected_for_keyword(
        grid: &BoardGrid,
        moves: &Vec<Move>,
        rc2: &RC, // other parts considered will be rc1 (prior move) and rc0 (2 prior moves)
    ) -> bool {
        // If this is the first move, then it is always accepted.
        if moves.len() == 0 {
            return true;
        }

        let rc1 = moves.last().unwrap().get_rc();

        if rc1 == rc2 {
            return false;
        }

        // Must be either vertically or horizontally aligned
        if rc2.0 != rc1.0 && rc2.1 != rc1.1 {
            return false;
        }

        // By default, just walk between the previous step and the current step.
        let mut row_walk_inc = rc2.0.cmp(&rc1.0) as i8 as isize;
        let mut col_walk_inc = rc2.1.cmp(&rc1.1) as i8 as isize;

        // If an earlier RC, rc0, was present, it may need to be factored in to the direction of movement.
        if moves.len() >= 2 {
            let rc0 = moves.get(moves.len() - 2).unwrap().get_rc();
            assert!(rc1.0 == rc0.0 || rc1.1 == rc0.1);

            if grid[rc1].is_conductor() {
                let (backtracking_row_walk_inc, backtracking_col_walk_inc) = (
                    rc0.0.cmp(&rc1.0) as i8 as isize,
                    rc0.1.cmp(&rc1.1) as i8 as isize,
                );

                if backtracking_row_walk_inc == row_walk_inc
                    && backtracking_col_walk_inc == col_walk_inc
                {
                    log!("Cannot backtrack through conductor {:?}", rc1);
                    return false;
                }
            } else {
                // If the previous RC was a regular space and not a conductor, then the direction from rc0 to rc1 must
                // be followed to get to rc2.
                row_walk_inc = rc1.0.cmp(&rc0.0) as i8 as isize;
                col_walk_inc = rc1.1.cmp(&rc0.1) as i8 as isize;
            }
        } else {
            // Cannot have a conductor accepted as the first move.
            assert!(!grid[rc1].is_conductor());
        }

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
            if row_walk_inc < 0 && current_rc.0 == 0 {
                log!(
                    "Traversed out of bounds to negative row from {:?}",
                    current_rc
                );
                return false;
            }

            if col_walk_inc < 0 && current_rc.1 == 0 {
                log!(
                    "Traversed out of bounds to negative col from {:?}",
                    current_rc
                );
                return false;
            }

            current_rc = RC(
                current_rc.0.checked_add_signed(row_walk_inc).unwrap(),
                current_rc.1.checked_add_signed(col_walk_inc).unwrap(),
            );

            if current_rc.0 >= grid.height() {
                log!("Traversed beyond row bounds from {:?}", current_rc);
                return false;
            }

            if current_rc.1 >= grid.width() {
                log!("Traversed beyond col bounds from {:?}", current_rc);
                return false;
            }

            if current_rc == *rc2 {
                return true;
            }

            let current = grid[&current_rc];
            if !current.is_traversible_for_keyword() {
                log!(
                    "Not connected: {:?} is not available for keyword traversal",
                    current_rc
                );
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lok1x4_correct() {
        let mut board = Board::new("LOK_").unwrap();
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
        let mut board = Board::new("LO-K-_").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 3);
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn lok_correct_jump_blackened() {
        let mut board = Board::new("LO_KLOK_").unwrap();
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
        let mut board = Board::new("LOK__").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(4));
    }

    #[test]
    fn lok1x5_unsolvable_out_of_order() {
        let mut board = Board::new("LKO_").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn lok1x4_out_of_order_middle() {
        let mut board = Board::new("LOK_").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(1));
    }

    #[test]
    fn lok1x4_out_of_order_backwards() {
        let mut board = Board::new("LOK_").unwrap();
        board.blacken(0, 2);
        board.blacken(0, 1);
        board.blacken(0, 0);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(0));
    }

    #[test]
    fn lok2x4_correct() {
        let mut board = Board::new(
            "LOK_\n\
             LOK_",
        )
        .unwrap();
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
        let mut board = Board::new(
            "LOK_\n\
             LOK_",
        )
        .unwrap();
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
    fn lok_illegal_turn() {
        let mut board = Board::new(
            "OL\n\
             K_",
        )
        .unwrap();

        board.blacken(0, 1);
        board.blacken(0, 0);
        board.blacken(1, 0);
        board.blacken(1, 1);

        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn lok_cannot_mark_path() {
        let mut board = Board::new("LOK_").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.mark_path(0, 3);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(3));
    }

    #[test]
    fn lok_cannot_change_letter() {
        let mut board = Board::new("LOK_").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.change_letter(0, 3, 'Q');
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), Some(3));
    }

    #[test]
    fn tlak_correct() {
        let mut board = Board::new("TLAK__").unwrap();
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
        let mut board = Board::new("TLAK_").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        assert_eq!(board.commit_and_check_solution(), Some(5));
    }

    #[test]
    fn tlak_wrong_k() {
        let mut board = Board::new("TLAZ__").unwrap();
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
    fn tlak_cannot_mark_path() {
        let mut board = Board::new("TLAK__").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        board.mark_path(0, 5);
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), Some(5));
    }

    #[test]
    fn tlak_cannot_change_leter() {
        let mut board = Board::new("TLAK__").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);
        board.change_letter(0, 5, 'Q');
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), Some(5));
    }

    #[test]
    fn ta_correct() {
        let mut board = Board::new(
            "TA-\n\
             Q-Q",
        )
        .unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(1, 0);
        board.blacken(1, 2);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn ta_multiple_letters() {
        let mut board = Board::new(
            "TA\n\
             QZ",
        )
        .unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(1, 0);
        board.blacken(1, 1);
        assert_eq!(board.commit_and_check_solution(), Some(3));
    }

    #[test]
    fn ta_correct_blanks() {
        let mut board = Board::new("TA__",).unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn ta_unsolvable_no_exec() {
        let mut board = Board::new("TA--").unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn ta_cannot_mark_path() {
        let mut board = Board::new(
            "TA-\n\
             Q-Q",
        )
        .unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(1, 0);
        board.mark_path(1, 0);
        board.blacken(1, 2);
        assert_eq!(board.commit_and_check_solution(), Some(3));
    }

    #[test]
    fn ta_cannot_change_letter() {
        let mut board = Board::new(
            "TA-\n\
             Z-Q",
        )
        .unwrap();
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.change_letter(1, 0, 'Q');
        board.blacken(1, 0);
        board.blacken(1, 2);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn x_correct() {
        let mut board = Board::new(
            "TXLX\n\
             -K--\n\
             -XAX\n\
             ----\n\
             TAX_",
        )
        .unwrap();

        // TLAK
        board.blacken(0, 0);
        board.mark_path(0, 1);
        board.blacken(0, 2);
        board.mark_path(0, 3);
        board.mark_path(2, 3);
        board.blacken(2, 2);
        board.mark_path(2, 1);
        board.blacken(1, 1);

        // Exec TLAK
        board.blacken(4, 2);
        board.blacken(4, 3);

        // TA
        board.blacken(4, 0);
        board.blacken(4, 1);

        // Exec TA
        board.blacken(0, 1);
        board.blacken(0, 3);
        board.blacken(2, 1);
        board.blacken(2, 3);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn x_implicit_move_through() {
        let mut board = Board::new("TXA").unwrap();

        // TA
        board.blacken(0, 0);
        board.blacken(0, 2);

        // Exec TA
        board.blacken(0, 1);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn x_loop() {
        let mut board = Board::new(
            "TXX\n\
             -XX\n\
             -AX",
        )
        .unwrap();

        // T
        board.blacken(0, 0);

        // Loop
        board.mark_path(0, 2);
        board.mark_path(1, 2);
        board.mark_path(1, 1);
        board.mark_path(0, 1);
        board.mark_path(0, 2);
        board.mark_path(1, 2);
        board.mark_path(1, 1);
        board.mark_path(0, 1);
        board.mark_path(0, 2);

        // A
        board.mark_path(2, 2);
        board.blacken(2, 1);

        // Exec TA
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(1, 1);
        board.blacken(1, 2);
        board.blacken(2, 2);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn x_incorrect_reversal() {
        let mut board = Board::new(
            "_-K\n\
             LOX\n\
             --X",
        )
        .unwrap();

        board.blacken(1, 0);
        board.blacken(1, 1);
        board.mark_path(1, 2);
        board.mark_path(2, 2);

        // Reversal not allowed
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 0);

        assert_eq!(board.commit_and_check_solution(), Some(4));
    }

    #[test]
    fn tlak_x_not_adjacent() {
        let mut board = Board::new("TLAK_X_LOK").unwrap();

        // TLAK
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);
        board.blacken(0, 3);

        // Exec TLAK, but these aren't adjacent because conductor
        board.blacken(0, 4);
        board.blacken(0, 6);

        // LOK
        board.blacken(0, 7);
        board.blacken(0, 8);
        board.blacken(0, 9);

        // Exec LOK
        board.blacken(0, 5);

        assert_eq!(board.commit_and_check_solution(), Some(5));
    }

    #[test]
    fn be_correct() {
        let mut board = Board::new("BEA_Z").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE
        board.change_letter(0, 3, 't');

        // TA
        board.blacken(0, 3);
        board.blacken(0, 2);

        // Exec TA
        board.blacken(0, 4);
        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn be_cannot_change_full_cell() {
        let mut board = Board::new("BEZ").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE, but not allowed to change regular cell
        board.change_letter(0, 2, 'Q');
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn be_cannot_change_letter_on_blackened() {
        let mut board = Board::new("BEBE_").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE
        board.change_letter(0, 4, 'Z');

        // BE
        board.blacken(0, 2);
        board.blacken(0, 3);

        // Exec BE, but not allowed to change letter of a blackened cell
        board.change_letter(0, 0, 'Z');
        assert_eq!(board.commit_and_check_solution(), Some(5));
    }

    #[test]
    fn be_cannot_blacken() {
        let mut board = Board::new("BEA_Z").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE, but blacken is not allowed
        board.blacken(0, 3);
        board.change_letter(0, 3, 't');

        // TA
        board.blacken(0, 3);
        board.blacken(0, 2);

        // Exec TA
        board.blacken(0, 4);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn be_cannot_mark_path() {
        let mut board = Board::new("BEA_Z").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE, but blacken is not allowed
        board.mark_path(0, 3);
        board.change_letter(0, 3, 't');

        // TA
        board.blacken(0, 3);
        board.blacken(0, 2);

        // Exec TA
        board.blacken(0, 4);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn be_invalid_underscore() {
        let mut board = Board::new("BELOK_").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE, but dash not allowed
        board.change_letter(0, 5, '_');

        // LOK
        board.blacken(0, 2);
        board.blacken(0, 3);
        board.blacken(0, 4);

        // Exec LOK
        board.blacken(0, 5);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn be_invalid_dash() {
        let mut board = Board::new("BEL_OK_").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE, but dash not allowed
        board.change_letter(0, 3, '-');

        // LOK
        board.blacken(0, 2);
        board.blacken(0, 4);
        board.blacken(0, 5);

        // Exec LOK
        board.blacken(0, 6);
        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn wildcard_correct_multiuse() {
        let mut board = Board::new(
            "?X\n\
                                    XX",
        )
        .unwrap();

        // T
        board.change_letter(0, 0, 'T');
        board.blacken(0, 0);
        board.mark_path(0, 1);
        board.mark_path(1, 1);
        board.mark_path(1, 0);

        // A
        board.change_letter(0, 0, 'A');
        board.blacken(0, 0);

        // Exec TA
        board.blacken(0, 1);
        board.blacken(1, 0);
        board.blacken(1, 1);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn wildcard_change_to_x() {
        let mut board = Board::new(
            "LO?\n\
                                    --K",
        )
        .unwrap();

        // LOK
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.change_letter(0, 2, 'X');
        board.mark_path(0, 2);
        board.blacken(1, 2);

        // Exec LOK
        board.blacken(0, 2);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn wildcard_correct_change_first_then_blacken() {
        let mut board = Board::new("????").unwrap();

        // LOK
        board.change_letter(0, 0, 'L');
        board.change_letter(0, 1, 'O');
        board.change_letter(0, 2, 'K');
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 3);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn wildcard_correct_change_and_blacken_interleaved() {
        let mut board = Board::new("????").unwrap();

        // LOK
        board.change_letter(0, 0, 'L');
        board.blacken(0, 0);
        board.change_letter(0, 1, 'O');
        board.blacken(0, 1);
        board.change_letter(0, 2, 'K');
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 3);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn be_makes_wildcard() {
        let mut board = Board::new("BE_AQ").unwrap();

        // BE
        board.blacken(0, 0);
        board.blacken(0, 1);

        // Exec BE
        board.change_letter(0, 2, '?');

        // TA
        board.change_letter(0, 2, 'T');
        board.blacken(0, 2);
        board.blacken(0, 3);

        // Exec TA
        board.blacken(0, 4);

        assert_eq!(board.commit_and_check_solution(), None);
    }

    #[test]
    fn cannot_change_regular_letter() {
        let mut board = Board::new("LOQ_").unwrap();

        // LOK, but can't randomly change a regular letter
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.change_letter(0, 2, 'K');
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 3);

        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn cannot_change_blank() {
        let mut board = Board::new("LO_K").unwrap();

        // LOK, but can't randomly change a blank
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.change_letter(0, 2, 'K');
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 3);

        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn cannot_change_gap() {
        let mut board = Board::new("LO-K").unwrap();

        // LOK, but can't randomly change a blank
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.change_letter(0, 2, 'K');
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 3);

        assert_eq!(board.commit_and_check_solution(), Some(2));
    }

    #[test]
    fn wildcard_cannot_change_blackened() {
        let mut board = Board::new("?OK_AQ").unwrap();

        // LOK
        board.change_letter(0, 0, 'L');
        board.blacken(0, 0);
        board.blacken(0, 1);
        board.blacken(0, 2);

        // Exec LOK
        board.blacken(0, 3);

        // TA, but you can't change a blackened cell, even if it had a wildcard before
        board.change_letter(0, 0, 'T');
        board.blacken(0, 0);
        board.blacken(0, 4);

        // Exec TA
        board.blacken(0, 5);

        assert_eq!(board.commit_and_check_solution(), Some(5));
    }
}
