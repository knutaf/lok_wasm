use std::ops::{Deref, DerefMut, Index, IndexMut};

/// A row/column pair for indexing into the grid.
/// Distinct from an x/y pair.
#[derive(PartialEq, Clone, Debug)]
pub struct RC(pub usize, pub usize);

/// An x/y pair for indexing into the grid.
/// Distinct from a row/column pair.
#[derive(PartialEq, Clone, Debug)]
pub struct XY(pub usize, pub usize);

/// A simple grid of user-defined objects.
///
/// It dereferences to a slice of [`CellType`], so you can directly manipulate
/// it via regular (mutable) slice methods. In addition, you can index
/// into it by `(row, column)` pairs.
#[derive(Clone)]
pub struct Grid<CellType>
where
    CellType: Clone,
{
    width: usize,
    height: usize,
    cells: Vec<CellType>,
}

impl<CellType> Grid<CellType>
where
    CellType: Clone,
{
    /// The width of the grid in cells.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The height of the grid in cells.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Converts an index into the cells vector into an XY coordinate.
    pub fn index_to_xy(&self, index: usize) -> XY {
        XY(index % self.width(), index / self.width())
    }

    /// Create a blank grid with the given dimensions.
    pub fn new(width: usize, height: usize, template: &CellType) -> Grid<CellType> {
        Grid {
            width,
            height,
            cells: vec![template.clone(); (width * height) as usize],
        }
    }

    pub fn cells(&self) -> &Vec<CellType> {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut Vec<CellType> {
        &mut self.cells
    }

    pub fn enumerate_row_col(&self) -> GridRowColumnEnumerator<CellType> {
        GridRowColumnEnumerator::new(&self)
    }
}

impl<CellType> Index<&RC> for Grid<CellType>
where
    CellType: Clone,
{
    type Output = CellType;
    fn index(&self, RC(row, col): &RC) -> &Self::Output {
        &self.cells[(row * self.width + col) as usize]
    }
}

impl<CellType> IndexMut<&RC> for Grid<CellType>
where
    CellType: Clone,
{
    fn index_mut(&mut self, RC(row, col): &RC) -> &mut Self::Output {
        &mut self.cells[(row * self.width + col) as usize]
    }
}

impl<CellType> Index<&XY> for Grid<CellType>
where
    CellType: Clone,
{
    type Output = CellType;
    fn index(&self, XY(x, y): &XY) -> &Self::Output {
        &self.cells[(*y * self.width + *x) as usize]
    }
}

impl<CellType> IndexMut<&XY> for Grid<CellType>
where
    CellType: Clone,
{
    fn index_mut(&mut self, XY(x, y): &XY) -> &mut Self::Output {
        &mut self.cells[(*y * self.width + *x) as usize]
    }
}

impl<CellType> Deref for Grid<CellType>
where
    CellType: Clone,
{
    type Target = [CellType];
    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

impl<CellType> DerefMut for Grid<CellType>
where
    CellType: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cells
    }
}

/// An enumerator that iterates low to high row number and low to high column number. Basically reading order.
pub struct GridRowColumnEnumerator<'g, CellType>
where
    CellType: Clone,
{
    grid: &'g Grid<CellType>,
    row: usize,
    col: usize,
}

impl<'g, CellType> GridRowColumnEnumerator<'g, CellType>
where
    CellType: Clone,
{
    fn new(grid: &'g Grid<CellType>) -> Self {
        Self {
            grid,
            row: 0,
            col: 0,
        }
    }
}

impl<'g, CellType> Iterator for GridRowColumnEnumerator<'g, CellType>
where
    CellType: Clone,
{
    type Item = (RC, &'g CellType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.grid.height() {
            return None;
        }

        if self.col >= self.grid.width() {
            self.col = 0;
            self.row += 1;
            return self.next();
        }

        let ret = Some((RC(self.row, self.col), &self.grid[&RC(self.row, self.col)]));
        self.col += 1;

        ret
    }
}
