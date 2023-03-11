use evdev::Key;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;

lazy_static! {
    static ref KEYS_TO_STRING: HashMap<Key, char> = HashMap::from(
        [
            (Key::KEY_1, '1'),
            (Key::KEY_2, '2'),
            (Key::KEY_3, '3'),
            (Key::KEY_4, '4'),
            (Key::KEY_5, '5'),
            (Key::KEY_6, '6'),
            (Key::KEY_7, '7'),
            (Key::KEY_8, '8'),
            (Key::KEY_9, '9'),
            (Key::KEY_0, '0'),
            (Key::KEY_MINUS, '-'),
            (Key::KEY_EQUAL, '='),
            (Key::KEY_TAB, '\t'),
            (Key::KEY_Q, 'q'),
            (Key::KEY_W, 'w'),
            (Key::KEY_E, 'e'),
            (Key::KEY_R, 'r'),
            (Key::KEY_T, 't'),
            (Key::KEY_Y, 'y'),
            (Key::KEY_U, 'u'),
            (Key::KEY_I, 'i'),
            (Key::KEY_O, 'o'),
            (Key::KEY_P, 'p'),
            (Key::KEY_LEFTBRACE, '['),
            (Key::KEY_RIGHTBRACE, ']'),
            (Key::KEY_A, 'a'),
            (Key::KEY_S, 's'),
            (Key::KEY_D, 'd'),
            (Key::KEY_F, 'f'),
            (Key::KEY_G, 'g'),
            (Key::KEY_H, 'h'),
            (Key::KEY_J, 'j'),
            (Key::KEY_K, 'k'),
            (Key::KEY_L, 'l'),
            (Key::KEY_SEMICOLON, ';'),
            (Key::KEY_APOSTROPHE, '\''),
            (Key::KEY_GRAVE, '`'),
            (Key::KEY_BACKSLASH, '\\'),
            (Key::KEY_Z, 'z'),
            (Key::KEY_X, 'x'),
            (Key::KEY_C, 'c'),
            (Key::KEY_V, 'v'),
            (Key::KEY_B, 'b'),
            (Key::KEY_N, 'n'),
            (Key::KEY_M, 'm'),
            (Key::KEY_COMMA, ','),
            (Key::KEY_DOT, '.'),
            (Key::KEY_SLASH, '/'),
            (Key::KEY_SPACE, ' '),
            // (Key::KEY_KPASTERISK, '*'),
            // (Key::KEY_SPACE, ' '),
            // (Key::KEY_KP7, '7'),
            // (Key::KEY_KP8, '8'),
            // (Key::KEY_KP9, '9'),
            // (Key::KEY_KPMINUS, '-'),
            // (Key::KEY_KP4, '4'),
            // (Key::KEY_KP5, '5'),
            // (Key::KEY_KP6, '6'),
            // (Key::KEY_KPPLUS, '+'),
            // (Key::KEY_KP1, '1'),
            // (Key::KEY_KP2, '2'),
            // (Key::KEY_KP3, '3'),
            // (Key::KEY_KP0, '0'),
            // (Key::KEY_KPDOT, '.'),
        ]
    );
    static ref SHIFT_KEYS_TO_STRING: HashMap<Key, char> = HashMap::from(
        [
            (Key::KEY_1, '!'),
            (Key::KEY_2, '@'),
            (Key::KEY_3, '#'),
            (Key::KEY_4, '$'),
            (Key::KEY_5, '%'),
            (Key::KEY_6, '^'),
            (Key::KEY_7, '&'),
            (Key::KEY_8, '*'),
            (Key::KEY_9, '('),
            (Key::KEY_0, ')'),
            (Key::KEY_MINUS, '_'),
            (Key::KEY_EQUAL, '+'),
            (Key::KEY_TAB, '\t'),
            (Key::KEY_Q, 'Q'),
            (Key::KEY_W, 'W'),
            (Key::KEY_E, 'E'),
            (Key::KEY_R, 'R'),
            (Key::KEY_T, 'T'),
            (Key::KEY_Y, 'Y'),
            (Key::KEY_U, 'U'),
            (Key::KEY_I, 'I'),
            (Key::KEY_O, 'O'),
            (Key::KEY_P, 'P'),
            (Key::KEY_LEFTBRACE, '{'),
            (Key::KEY_RIGHTBRACE, '}'),
            (Key::KEY_A, 'A'),
            (Key::KEY_S, 'S'),
            (Key::KEY_D, 'D'),
            (Key::KEY_F, 'F'),
            (Key::KEY_G, 'G'),
            (Key::KEY_H, 'H'),
            (Key::KEY_J, 'J'),
            (Key::KEY_K, 'K'),
            (Key::KEY_L, 'L'),
            (Key::KEY_SEMICOLON, ':'),
            (Key::KEY_APOSTROPHE, '"'),
            (Key::KEY_GRAVE, '~'),
            (Key::KEY_BACKSLASH, '|'),
            (Key::KEY_Z, 'Z'),
            (Key::KEY_X, 'X'),
            (Key::KEY_C, 'C'),
            (Key::KEY_V, 'V'),
            (Key::KEY_B, 'B'),
            (Key::KEY_N, 'N'),
            (Key::KEY_M, 'M'),
            (Key::KEY_COMMA, '<'),
            (Key::KEY_DOT, '>'),
            (Key::KEY_SLASH, '?'),
            (Key::KEY_SPACE, ' '),
            // (Key::KEY_KPASTERISK, '*'),
            // (Key::KEY_SPACE, ' '),
            // (Key::KEY_KP7, '7'),
            // (Key::KEY_KP8, '8'),
            // (Key::KEY_KP9, '9'),
            // (Key::KEY_KPMINUS, '-'),
            // (Key::KEY_KP4, '4'),
            // (Key::KEY_KP5, '5'),
            // (Key::KEY_KP6, '6'),
            // (Key::KEY_KPPLUS, '+'),
            // (Key::KEY_KP1, '1'),
            // (Key::KEY_KP2, '2'),
            // (Key::KEY_KP3, '3'),
            // (Key::KEY_KP0, '0'),
            // (Key::KEY_KPDOT, '.'),
        ]
    );

    pub static ref HANDLED_KEYS: HashSet<Key> = HashSet::from_iter(
        KEYS_TO_STRING.iter().map(|(k, _)| k.clone()).chain(
            [
                Key::KEY_BACKSPACE,
                Key::KEY_LEFT,
                Key::KEY_RIGHT,
                Key::KEY_DELETE
            ]
        )
    );
}

/// Used to determine if an event changed the entry.
pub enum EntryStatus {
    Change,
    NoChange,
}

#[derive(Debug)]
pub struct Terminal {
    entry: Vec<char>,
    pos: usize,
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {
            entry: Vec::new(),
            pos: 0,
        }
    }

    fn backspace(&mut self) -> EntryStatus {
        if self.pos > 0 {
            self.pos -= 1;
            self.entry.remove(self.pos);
            return EntryStatus::Change;
        }
        return EntryStatus::NoChange;
    }

    fn delete(&mut self) -> EntryStatus {
        if self.pos < self.entry.len() {
            self.entry.remove(self.pos);
            return EntryStatus::Change;
        }
        return EntryStatus::NoChange;
    }

    fn left(&mut self) -> EntryStatus {
        if self.pos > 0 {
            self.pos -= 1;
            return EntryStatus::Change;
        }
        return EntryStatus::NoChange;
    }

    fn right(&mut self) -> EntryStatus {
        if self.pos < self.entry.len() {
            self.pos += 1;
            return EntryStatus::Change;
        }
        return EntryStatus::NoChange;
    }

    fn enter(&mut self) {
        self.entry.clear();
        self.pos = 0;
    }

    fn add_char(&mut self, c: char) -> EntryStatus {
        self.entry.insert(self.pos, c);
        self.pos += 1;
        return EntryStatus::Change;
    }

    fn get_entry(&self) -> String {
        self.entry.iter().collect()
    }

    pub fn handle_key(&mut self, key: Key, shift: bool) -> EntryStatus {
        if shift {
            if let Some(c) = SHIFT_KEYS_TO_STRING.get(&key) {
                return self.add_char(*c);
            }
        } else {
            if let Some(c) = KEYS_TO_STRING.get(&key) {
                return self.add_char(*c);
            }
        }
        match key {
            Key::KEY_BACKSPACE => self.backspace(),
            Key::KEY_DELETE => self.delete(),
            Key::KEY_LEFT => self.left(),
            Key::KEY_RIGHT => self.right(),
            _ => EntryStatus::NoChange,
        }
    }
}
