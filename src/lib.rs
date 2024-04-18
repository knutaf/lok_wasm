mod utils;
mod grid;

use crate::grid::Grid;

#[wasm_bindgen]
pub struct Board {
    grid: Grid<u8>,
}

#[wasm_bindgen]
impl Board {
    pub fn new() -> Board {
        Board {
            grid: Grid::new(6, 4, &('!' as u8)),
        }
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

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
