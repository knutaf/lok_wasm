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
pub struct Board {
    grid: Grid<u8>,
}

#[wasm_bindgen]
impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            grid: Grid::new(4, 1, &(' ' as u8)),
        };

        board.grid[RC(0, 0)] = 'L' as u8;
        board.grid[RC(0, 1)] = 'O' as u8;
        board.grid[RC(0, 2)] = 'K' as u8;

        board
    }

    pub fn width(&self) -> u32 {
        self.grid.width() as u32
    }

    pub fn height(&self) -> u32 {
        self.grid.height() as u32
    }

    pub fn cells(&self) -> *const u8 {
        self.grid.cells()
    }
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
