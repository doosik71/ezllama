use std::io;
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

pub fn select_index<F>(item_count: usize, mut draw: F) -> io::Result<Option<usize>>
where
    F: FnMut(&mut io::Stdout, usize, usize) -> io::Result<()>,
{
    if item_count == 0 {
        return Ok(None);
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let result = run_picker(&mut stdout, item_count, &mut draw);

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn run_picker<F>(
    stdout: &mut io::Stdout,
    item_count: usize,
    draw: &mut F,
) -> io::Result<Option<usize>>
where
    F: FnMut(&mut io::Stdout, usize, usize) -> io::Result<()>,
{
    let mut selected = 0usize;
    let mut offset = 0usize;

    loop {
        draw(stdout, selected, offset)?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key_event) = event::read()? else {
            continue;
        };

        match key_event.code {
            KeyCode::Up => {
                selected = selected.saturating_sub(1);
                if selected < offset {
                    offset = selected;
                }
            }
            KeyCode::Down => {
                if selected + 1 < item_count {
                    selected += 1;
                }
                let visible_rows = visible_rows()?;
                if selected >= offset + visible_rows {
                    offset = selected + 1 - visible_rows;
                }
            }
            KeyCode::PageUp => {
                let visible_rows = visible_rows()?;
                selected = selected.saturating_sub(visible_rows);
                offset = offset.saturating_sub(visible_rows);
                if selected < offset {
                    offset = selected;
                }
            }
            KeyCode::PageDown => {
                let visible_rows = visible_rows()?;
                selected = (selected + visible_rows).min(item_count - 1);
                if selected >= offset + visible_rows {
                    offset = selected + 1 - visible_rows;
                }
            }
            KeyCode::Enter => {
                return Ok(Some(selected));
            }
            KeyCode::Esc => {
                return Ok(None);
            }
            _ => {}
        }
    }
}

fn visible_rows() -> io::Result<usize> {
    let (_, rows) = terminal::size()?;
    Ok(rows.saturating_sub(2).max(1) as usize)
}
