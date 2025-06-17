use std::time::Duration;

use crate::grid::{CellState, Grid};

/// Bounds for tick interval.
const MIN_INTERVAL: Duration = Duration::from_millis(30);
const MAX_INTERVAL: Duration = Duration::from_millis(1000);
/// Step size for speed adjustments.
const INTERVAL_STEP: Duration = Duration::from_millis(10);
/// Default tick interval.
const DEFAULT_INTERVAL: Duration = Duration::from_millis(100);

/// Current state of the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// Simulation is actively running.
    Running,
    /// Simulation is paused.
    Paused,
}

/// Manages core game logic and simulation state for Conway's Game of Life.
#[derive(Debug, Clone)]
pub struct Game {
    /// Current generation's grid state.
    pub grid: Grid,
    /// Next generation's grid (pre-allocated for performance).
    next_grid: Grid,

    /// Simulation state.
    pub state: GameState,
    /// Time between simulation steps.
    pub tick_interval: Duration,
    /// Number of generations that have elapsed.
    pub generation: u64,
}

impl Game {
    /// Creates a new game with specified grid size
    pub fn new(grid_size: (usize, usize)) -> Self {
        let grid = Grid::new(grid_size.0, grid_size.1);
        let next_grid = Grid::new(grid_size.0, grid_size.1);

        Self {
            grid,
            next_grid,

            state: GameState::Running,
            tick_interval: DEFAULT_INTERVAL,
            generation: 0,
        }
    }

    /// Advances the simulation by one generation.
    ///
    /// Applies Game of Life rules:
    /// - Live cells with 2-3 neighbors -> alive
    /// - Dead cells with 3 neighbors -> alive
    /// - All other cells -> dead
    pub fn step(&mut self) {
        for row in 0..self.grid.height {
            for col in 0..self.grid.width {
                let current_state = self.grid.get(row, col).unwrap_or(CellState::Dead);
                let neighbors = self.grid.count_neighbors(row, col);

                let new_state = match (current_state, neighbors) {
                    (CellState::Alive, 2 | 3) => CellState::Alive, // survival
                    (CellState::Dead, 3) => CellState::Alive,      // birth
                    _ => CellState::Dead,                          // death
                };

                self.next_grid.set(row, col, new_state);
            }
        }

        // Swap grids
        std::mem::swap(&mut self.grid, &mut self.next_grid);

        // Update stats
        self.generation += 1;
    }

    /// Resizes the grid while preserving existing cells where possible.
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if self.grid.width == new_width && self.grid.height == new_height {
            return;
        }
        self.grid.resize(new_width, new_height);
        self.next_grid = Grid::new(new_width, new_height);
    }

    /// Clears the grid and resets stats.
    pub fn clear(&mut self) {
        self.grid.clear();
        self.generation = 0;
    }

    /// Randomizes the grid with the specified density of alive cells (0.0 to 1.0).
    pub fn randomize(&mut self, density: f32) {
        assert!(
            (0.0..=1.0).contains(&density),
            "Density must be within 0.0 to 1.0"
        );

        use rand::Rng;

        let mut rng = rand::rng();

        for row in 0..self.grid.height {
            for col in 0..self.grid.width {
                let state = if rng.random::<f32>() < density {
                    CellState::Alive
                } else {
                    CellState::Dead
                };
                self.grid.set(row, col, state);
            }
        }
        self.generation = 0;
    }

    /// Increases the tick interval (slows down the simulation).
    pub fn inc_interval(&mut self) -> Duration {
        self.tick_interval = self
            .tick_interval
            .saturating_add(INTERVAL_STEP)
            .min(MAX_INTERVAL);
        self.tick_interval
    }

    /// Decreases the tick intierval (speeds up the simulation).
    pub fn dec_interval(&mut self) -> Duration {
        self.tick_interval = self
            .tick_interval
            .saturating_sub(INTERVAL_STEP)
            .max(MIN_INTERVAL);
        self.tick_interval
    }

    /// Toggles between running and paused states.
    pub fn toggle_pause(&mut self) {
        self.state = if self.state == GameState::Running {
            GameState::Paused
        } else {
            GameState::Running
        };
    }

    /// Returns true if the simulation is currently paused.
    pub fn is_paused(&self) -> bool {
        self.state == GameState::Paused
    }
}
