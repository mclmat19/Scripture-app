use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod bible;
mod ui;

use app::{App, Mode, Pane};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Menu => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('f') => {
                        app.search_origin = Mode::Menu;
                        app.mode = Mode::Search;
                        app.search_query.clear();
                        app.search_results.clear();
                        app.search_navigating = false;
                    }
                    KeyCode::Char('c') => {
                        app.mode = Mode::Normal;
                        app.active_pane = Pane::Reader;
                    }
                    KeyCode::Char('r') => {
                        app.search_origin = Mode::Menu;
                        app.mode = Mode::Search;
                        app.search_query.clear();
                        app.search_results.clear();
                        app.search_navigating = false;
                    }
                    _ => {}
                },
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => app.mode = Mode::Menu,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.active_pane == Pane::Tree { app.tree_down(); }
                        else { app.reader_down(); }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if app.active_pane == Pane::Tree { app.tree_up(); }
                        else { app.reader_up(); }
                    }

                    KeyCode::Enter => app.tree_enter(),
                    KeyCode::Char('n') => app.next_chapter(),
                    KeyCode::Char('p') => app.prev_chapter(),
                    KeyCode::Char('/') => {
                        app.search_origin = Mode::Normal;
                        app.mode = Mode::Search;
                        app.search_query.clear();
                        app.search_results.clear();
                        app.search_navigating = false;
                    }

                    KeyCode::Char(':') => {
                        app.mode = Mode::Command;
                        app.command_input.clear();
                    }

                    KeyCode::Tab => {
                        app.active_pane = if app.active_pane == Pane::Tree {
                            Pane::Reader
                        } else {
                            Pane::Tree
                        };
                    }
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Esc => {
                        app.mode = app.search_origin.clone();
                        app.search_query.clear();
                        app.search_navigating = false;
                    }

                    KeyCode::Tab => {
                        app.search_navigating = !app.search_navigating;
                    }

                    KeyCode::Enter => {
                        app.jump_to_search_result();
                        app.search_navigating = false;
                    }
                    KeyCode::Char('j') | KeyCode::Down if app.search_navigating => {
                        if app.search_cursor +1 < app.search_results.len() {
                            app.search_cursor += 1;
                        }
                    }

                    KeyCode::Char('k') | KeyCode::Up if app.search_navigating => {
                        if app.search_cursor > 0 {
                            app.search_cursor -= 1;
                        }
                    }

                    KeyCode::Char(c) if !app.search_navigating => {
                        app.search_query.push(c);
                        app.search_execute();
                    }
                    KeyCode::Backspace => { app.search_query.pop();
                        app.search_execute();
                    }
                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.command_input.clear();
                    }
                    KeyCode::Enter => {
                        app.execute_command();
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Char(c) => app.command_input.push(c),
                    KeyCode::Backspace => {app.command_input.pop(); }
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
