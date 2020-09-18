use ropey::Rope;
use std::cmp;
use termion::event::Key;

pub struct Prompt<T> {
    pub history: Vec<(String, T)>,
    pub current: Rope,
    pub cursor: usize,
    history_cursor: Option<usize>,
}

fn safe_addition(current: usize, to_add: i64, max: usize) -> usize {
    if to_add < 0 {
        current.checked_sub(to_add.abs() as usize).unwrap_or(0)
    } else {
        cmp::min(max, current.checked_add(to_add as usize).unwrap_or(max))
    }
}

impl<T> Prompt<T> {
    pub fn new() -> Prompt<T> {
        Prompt {
            history: Vec::new(),
            current: Rope::new(),
            cursor: 0,
            history_cursor: None,
        }
    }

    fn maybe_clone_from_history(&mut self) {
        if let Some(history_cursor) = self.history_cursor {
            if history_cursor > 0 {
                if let Some((string, _)) = self.history.get(history_cursor - 1) {
                    self.current = Rope::from(string.as_str());
                    self.history_cursor = None;
                    self.cursor = self.current.len_chars();
                }
            }
        }
    }

    fn move_cursor(&mut self, by_how_much: i64) {
        self.maybe_clone_from_history();
        self.cursor = safe_addition(self.cursor, by_how_much, self.current.len_chars());
    }

    fn move_history_cursor(&mut self, by_how_much: i64) {
        let len = self.current.len_chars();
        if len > 0 {
            self.history_cursor = None;
            return;
        }

        let history_cursor = safe_addition(
            self.history_cursor.unwrap_or(0),
            by_how_much,
            self.history.len(),
        );

        if history_cursor == 0 {
            self.history_cursor = None;
        } else {
            self.history_cursor = Some(history_cursor);
        }
        println!("history {:?}", self.history_cursor);
    }

    fn remove(&mut self, selection: i64) {
        self.maybe_clone_from_history();
        let (start, end) = if selection < 0 {
            (
                self.cursor
                    .checked_sub(selection.abs() as usize)
                    .unwrap_or(0),
                self.cursor,
            )
        } else {
            (
                self.cursor,
                cmp::min(self.cursor + (selection as usize), self.current.len_chars()),
            )
        };
        self.current.remove(start..end);
        if selection < 0 {
            self.cursor = self
                .cursor
                .checked_sub(selection.abs() as usize)
                .unwrap_or(0);
        } else {
            self.cursor = cmp::min(self.cursor, self.current.len_chars());
        }
    }

    fn insert(&mut self, ch: char) {
        self.maybe_clone_from_history();
        self.current.insert_char(self.cursor, ch);
        self.cursor += 1;
    }

    fn execute<F>(&mut self, process: F)
    where
        F: FnOnce(&String) -> T,
    {
        let mut current = Rope::new();
        std::mem::swap(&mut current, &mut self.current);
        let current = String::from(current);
        self.cursor = 0;
        let result = process(&current);
        self.history.push((current, result));
    }

    pub fn key<F>(&mut self, key: Key, process: F)
    where
        F: FnOnce(&String) -> T,
    {
        match key {
            Key::Up => self.move_history_cursor(1),
            Key::Down => self.move_history_cursor(-1),
            Key::Left => self.move_cursor(-1),
            Key::Right => self.move_cursor(1),
            Key::Backspace => self.remove(-1),
            Key::Delete => self.remove(1),
            Key::Char('\n') => self.execute(process),
            Key::Char(ch) => self.insert(ch),
            _ => {}
        }
    }
}
