use std::io;
use std::sync::mpsc;
use std::sync::mpsc::RecvError;
use std::thread;
use termion::event;
use termion::input::TermRead;
use termion::terminal_size;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemEvent {
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    Key(event::Key),
    Mouse(event::MouseEvent),
    System(SystemEvent),
    Unsupported(Vec<u8>),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct Events {
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    input_handle: thread::JoinHandle<()>,
    resize_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.events() {
                    if let Ok(evt) = evt {
                        let mut should_return = false;
                        let evt = match evt {
                            event::Event::Key(event::Key::Ctrl('c')) => {
                                Event::System(SystemEvent::Exit)
                            }
                            event::Event::Key(key) => Event::Key(key),
                            event::Event::Mouse(mouse) => Event::Mouse(mouse),
                            event::Event::Unsupported(data) => Event::Unsupported(data),
                        };
                        if evt == Event::System(SystemEvent::Exit) {
                            should_return = true;
                        }
                        if let Err(err) = tx.send(evt) {
                            eprintln!("{}", err);
                            return;
                        }
                        if should_return {
                            return;
                        }
                    }
                }
            })
        };
        let resize_handle = {
            let tx = tx.clone();
            let (mut cols, mut rows) = terminal_size().unwrap();
            thread::spawn(move || loop {
                let (new_cols, new_rows) = terminal_size().unwrap();
                if (cols, rows) != (new_cols, new_rows) {
                    cols = new_cols;
                    rows = new_rows;
                    if tx.send(Event::Resize(cols, rows)).is_err() {
                        break;
                    }
                }
                thread::sleep(std::time::Duration::from_millis(16));
            })
        };
        Events {
            tx,
            rx,
            input_handle,
            resize_handle,
        }
    }

    pub fn next(&self) -> Result<Event, RecvError> {
        self.rx.recv()
    }
}
