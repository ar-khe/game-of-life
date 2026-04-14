use std::io::{self, stdout, Stdout};

use crossterm::terminal::LeaveAlternateScreen;
use ratatui::{
    Terminal, backend::CrosstermBackend, crossterm::{
        execute,
        terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode}
    },
};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    set_panic_hook();
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        hook(panic_info);
    }));
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}