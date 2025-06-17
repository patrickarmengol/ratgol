use std::fmt;

/// State of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Dead,
    Alive,
}

impl CellState {
    /// Returns true if the cell is alive
    pub fn is_alive(&self) -> bool {
        matches!(self, CellState::Alive)
    }
}

/// A 2D grid for Conway's Game of Life with toroidal topology
///
/// Uses a flat vector internally for better cache locality
#[derive(Clone, Debug)]
pub struct Grid {
    /// Flat storage of all cells.
    cells: Vec<CellState>,
    /// Width of th egrid in cells.
    pub width: usize,
    /// Height of the grid in cells.
    pub height: usize,
    /// Current number of alive cells.
    pub population: usize,
}

impl Grid {
    /// Creates a new grid of given dimensions with all cells initially dead.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "Grid dimensions must be positive and non-zero"
        );

        Self {
            cells: vec![CellState::Dead; width * height],
            width,
            height,
            population: 0,
        }
    }

    /// Gets the state of a cell at the given coordinates.
    ///
    /// Returns `None` if coordinates are out of bounds.
    pub fn get(&self, row: usize, col: usize) -> Option<CellState> {
        // Ensure coordinate is in bounds
        if row >= self.height || col >= self.width {
            return None;
        }

        Some(self.cells[row * self.width + col])
    }

    /// Gets the state of a cell with toroidal wrapping.
    pub fn get_wrapped(&self, row: isize, col: isize) -> CellState {
        let row = row.rem_euclid(self.height as isize) as usize;
        let col = col.rem_euclid(self.width as isize) as usize;
        self.cells[row * self.width + col]
    }

    /// Sets the state of a cell at the given coordinates.
    ///
    /// Returns `None` if coordinates are out of bounds.
    /// Automatically updates the population count.
    pub fn set(&mut self, row: usize, col: usize, state: CellState) -> Option<CellState> {
        if row >= self.height || col >= self.width {
            return None;
        }

        let idx = row * self.width + col;
        let old = self.cells[idx];

        // only update on state changes
        if old != state {
            // Update population count
            if old.is_alive() && !state.is_alive() {
                self.population = self.population.saturating_sub(1);
            } else if !old.is_alive() && state.is_alive() {
                self.population += 1;
            }
            self.cells[idx] = state;
        }

        Some(old)
    }

    /// Counts the number of alive neighbors in Moore neighborhood of a cell.
    pub fn count_neighbors(&self, row: usize, col: usize) -> u8 {
        const NEIGHBORS: [(isize, isize); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        NEIGHBORS
            .iter()
            .filter(|&&(dr, dc)| {
                self.get_wrapped(row as isize + dr, col as isize + dc)
                    .is_alive()
            })
            .count() as u8
    }

    /// Resizes the grid, preserving existing cells that fit within the new dimensions.
    ///
    /// Cells outside the new dimensions are discarded. New areas are initalized dead.
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let mut new_cells = vec![CellState::Dead; new_width * new_height];

        let mut new_pop = 0;

        // Copy existing cells that fit in the new grid
        let copy_width = self.width.min(new_width);
        let copy_height = self.height.min(new_height);

        for row in 0..copy_height {
            for col in 0..copy_width {
                let old_idx = row * self.width + col;
                let new_idx = row * new_width + col;
                let state = self.cells[old_idx];
                new_cells[new_idx] = state;
                if state.is_alive() {
                    new_pop += 1
                }
            }
        }

        self.cells = new_cells;
        self.width = new_width;
        self.height = new_height;
        self.population = new_pop;
        // Trail length is preserved during resize
    }

    /// Clears all cells, setting them to dead.
    pub fn clear(&mut self) {
        self.cells.fill(CellState::Dead);
        self.population = 0;
    }

    /// Checks if the grid is empty (no alive cells).
    pub fn is_empty(&self) -> bool {
        self.population == 0
    }

    /// Returns an iterator over all cells with their (row, col) coordinates.
    pub fn iter_cells(&self) -> impl Iterator<Item = ((usize, usize), CellState)> + '_ {
        self.cells.iter().enumerate().map(move |(idx, &state)| {
            let row = idx / self.width;
            let col = idx % self.width;
            ((row, col), state)
        })
    }

    /// Returns an iterator over only live cells.
    pub fn iter_alive_cells(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.cells
            .iter()
            .enumerate()
            .filter(|&(_, &state)| state.is_alive())
            .map(move |(idx, _)| {
                let row = idx / self.width;
                let col = idx % self.width;
                (row, col)
            })
    }

    /// Renders the grid to a string using Unicode block characters.
    pub fn render(&self) -> String {
        // Pre-allocates the string with the exact capacity needed.
        // Each cell is 2 chars wide, plus newlines
        let mut result = String::with_capacity(self.height * (self.width * 2 + 1));

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = row * self.width + col;
                let cell_str = match self.cells[idx] {
                    CellState::Alive => "██",
                    CellState::Dead => "  ",
                };
                result.push_str(cell_str);
            }
            if row < self.height - 1 {
                result.push('\n');
            }
        }

        result
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}
