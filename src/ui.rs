use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{app::App, game::Game};
use crate::{game::GameState, grid::Grid};

/// Grid dimension bounds.
const MIN_GRID_WIDTH: usize = 20;
const MIN_GRID_HEIGHT: usize = 15;
const MAX_GRID_WIDTH: usize = 200;
const MAX_GRID_HEIGHT: usize = 100;

/// Width of each cell in terminal characters.
/// Uses 2 characters per cell for better visual proportions.
const CELL_WIDTH: usize = 2;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(0),    // Grid
                Constraint::Length(3), // Status
            ]);
        let chunks = layout.split(area);

        GridDisplay::new(&self.game.grid).render(chunks[0], buf);

        StatusBar::new(&self.game).render(chunks[1], buf);
    }
}

/// Widget for rendering the game grid.
struct GridDisplay<'a> {
    grid: &'a Grid,
}

impl<'a> GridDisplay<'a> {
    fn new(grid: &'a Grid) -> Self {
        Self { grid }
    }
}

impl<'a> Widget for GridDisplay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Game of Life");
        let inner = block.inner(area);
        block.render(area, buf);

        // Calculate display capacity
        let max_cols = (inner.width as usize) * CELL_WIDTH;
        let max_rows = inner.height as usize;

        // Display warning if grid exceeds capacity of display area
        if self.grid.width > max_cols || self.grid.height > max_rows {
            let warning = format!(
                "Grid {}×{} exceeds display capacity {}×{}",
                self.grid.width, self.grid.height, max_cols, max_rows
            );

            Paragraph::new(warning)
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .render(inner, buf);
            return;
        }

        // Render the grid using the pre-formatted string representation
        Paragraph::new(self.grid.to_string())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .render(inner, buf);
    }
}

/// Widget for the status bar
struct StatusBar<'a> {
    game: &'a Game,
}

impl<'a> StatusBar<'a> {
    fn new(game: &'a Game) -> Self {
        Self { game }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (state_text, state_color) = match self.game.state {
            GameState::Paused => ("PAUSED", Color::Red),
            GameState::Running => ("RUNNING", Color::Green),
        };

        let status_parts = [
            state_text.to_string(),
            format!("gen: {}", self.game.generation),
            format!("pop: {}", self.game.grid.population),
            format!("{}×{}", self.game.grid.width, self.game.grid.height),
            format!("{}ms", self.game.tick_interval.as_millis()),
        ];

        let status_text = status_parts.join(" │ ");
        let help_text = " -- <space>: pause │ <r>: random │ <↑/↓>: speed │ <q>: quit";

        let content = Line::from(vec![status_text.into(), help_text.into()]);

        Paragraph::new(content)
            .style(Style::default().fg(state_color))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}

/// Calculates appropriate grid dimensions based on terminal size.
/// Accounts for borders, margins, and the status bar.
/// Clamps the result within bounds to ensure usablity.
pub fn calculate_grid_size(terminal_size: (u16, u16)) -> (usize, usize) {
    let (term_width, term_height) = terminal_size;

    // 2 border + 2 margin
    let available_width = term_width.saturating_sub(4) as usize / CELL_WIDTH;
    // 2 border + 2 margin + 3 status box
    let available_height = term_height.saturating_sub(7) as usize;

    let grid_width = available_width.clamp(MIN_GRID_WIDTH, MAX_GRID_WIDTH);
    let grid_height = available_height.clamp(MIN_GRID_HEIGHT, MAX_GRID_HEIGHT);

    (grid_width, grid_height)
}
