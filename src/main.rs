mod events;
mod prompt;

use crate::events::{Event, Events, SystemEvent};
use crate::prompt::Prompt;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::Paragraph;
use tui::widgets::{Block, Borders, Clear, Widget};
use tui::Frame;
use tui::Terminal;

use termion::event::Key;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let input_events = Events::new();
    let prompt = Rc::new(RefCell::new(Prompt::new()));

    terminal.clear()?;

    loop {
        {
            let prompt = prompt.clone();
            terminal.draw(move |f| {
                // draw_on_clear(f, f.size());
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    // .margin(1)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());
                let a = String::from(&prompt.borrow().current);
                let block =
                    Paragraph::new(a.as_str()).block(Block::default().title("Block").borders(Borders::ALL));
                f.render_widget(block, chunks[0]);
                f.set_cursor(chunks[0].x, chunks[0].y);
            })?;
        }
        {
            let prompt = prompt.clone();
            match input_events.next()? {
                Event::System(SystemEvent::Exit) => {
                    return Ok(());
                }
                Event::Key(key) => prompt.borrow_mut().key(key, |_| ()),
                Event::Unsupported(u) => println!("{:?}", u),
                _ => {}
            }
        }
    }
}
