use crossterm::event::{KeyCode, KeyEvent};
use ratatui::DefaultTerminal;

use crate::{
    event::{AppEvent, Event, EventHandler},
    game::Game,
    ui::calculate_grid_size,
};

/// Application settings for configuring behavior.
#[derive(Debug, Clone)]
pub struct AppSettings {
    /// Density of alive cells when randomizing (0.0 to 1.0)
    pub fill_density: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            fill_density: 0.3, // for randomizer
        }
    }
}

/// Main application state and control logic.
pub struct App {
    /// The game logic and grid state.
    pub game: Game,
    /// Event handler for terminal and application events.
    events: EventHandler,
    /// Flag to signal application shutdown.
    should_quit: bool,
    /// User configurable settings.
    pub settings: AppSettings,
}

impl App {
    /// Creates a new application instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the application's main loop until the user quits.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Processes all pending events and updates application state.
    fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.game.step(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
                crossterm::event::Event::Resize(w, h) => {
                    let (new_grid_width, new_grid_height) = calculate_grid_size((w, h));
                    self.game.resize(new_grid_width, new_grid_height);
                }
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Randomize => self.game.randomize(self.settings.fill_density),
                AppEvent::Clear => self.game.clear(),
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    /// Processes keyboard input.
    ///
    /// # Keybinds
    ///
    /// `Esc` or `q`: Quit the application
    /// `Space`: Toggle pause/resume
    /// `Up`: Increase simulation tick interval
    /// `Down`: Decrease simulation tick interval
    /// `r`: Randomize grid
    /// `c`: Clear grid
    fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),

            KeyCode::Up => {
                self.events.set_tick_interval(self.game.inc_interval());
            }
            KeyCode::Down => {
                self.events.set_tick_interval(self.game.dec_interval());
            }
            KeyCode::Char(' ') => {
                self.game.toggle_pause();
                if self.game.is_paused() {
                    self.events.pause();
                } else {
                    self.events.resume();
                }
            }
            KeyCode::Char('r') => self.events.send(AppEvent::Randomize),
            KeyCode::Char('c') => self.events.send(AppEvent::Clear),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Signals the application to terminate.
    fn quit(&mut self) {
        self.should_quit = true;
    }
}

impl Default for App {
    fn default() -> Self {
        let terminal_size = crossterm::terminal::size().unwrap();
        let grid_size = calculate_grid_size(terminal_size);

        let settings = AppSettings::default();

        let mut game = Game::new(grid_size);
        game.randomize(settings.fill_density);

        let events = EventHandler::new(game.tick_interval, game.is_paused());

        Self {
            game,
            should_quit: false,
            events,
            settings,
        }
    }
}
