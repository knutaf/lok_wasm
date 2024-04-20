use wasm_bindgen::prelude::*;

extern crate web_sys;

mod grid;
mod utils;

use crate::grid::{Grid, RC};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
struct BoardCell(u8);
type BoardGrid = Grid<BoardCell>;
// TODO: compile time assert that size of board cell is u8

impl BoardCell {
    fn blank() -> BoardCell {
        BoardCell(' ' as u8)
    }

    fn letter(c: char) -> BoardCell {
        assert!(c.is_ascii());
        BoardCell(c as u8)
    }
}

#[wasm_bindgen]
pub struct Board {
    grid: BoardGrid,
    grid_stack: Vec<BoardGrid>,
}

#[wasm_bindgen]
impl Board {
    pub fn new(rows: usize, cols: usize) -> Board {
        let mut board = Board {
            grid: Grid::new(cols, rows, &BoardCell::blank()),
            grid_stack: vec![],
        };

        board.grid[RC(0, 0)] = BoardCell::letter('L');
        board.grid[RC(0, 1)] = BoardCell::letter('O');
        board.grid[RC(0, 2)] = BoardCell::letter('K');

        board
    }

    pub fn width(&self) -> u32 {
        self.grid.width() as u32
    }

    pub fn height(&self) -> u32 {
        self.grid.height() as u32
    }

    pub fn cells(&self) -> *const BoardCell {
        self.grid.cells()
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(result, 4);
    }
}
*/
