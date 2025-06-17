use ratgol::app::App;

fn main() -> color_eyre::Result<()> {
    // Initialize error handling
    color_eyre::install().unwrap();

    // Initialize terminal
    let terminal = ratatui::init();

    // Create and run the app
    let app = App::new();
    let result = app.run(terminal);

    // Restore terminal
    ratatui::restore();

    result
}
