use evdev::uinput::VirtualDevice;
use evdev::EventType;
use evdev::InputEvent;
use evdev::Key;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Formatter;
use subprocess::Popen;
use subprocess::PopenConfig;
use subprocess::Redirection;

lazy_static! {
    static ref KEY_TO_CHAR: HashMap<Key, char> = HashMap::from(
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
    static ref SHIFT_KEY_TO_CHAR: HashMap<Key, char> = HashMap::from(
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
        KEY_TO_CHAR.iter().map(|(k, _)| *k).chain(
            [
                Key::KEY_BACKSPACE,
                Key::KEY_LEFT,
                Key::KEY_RIGHT,
                Key::KEY_DELETE
            ]
        )
    );

    pub static ref CHAR_TO_KEY: HashMap<char, (Key, bool)> = HashMap::from_iter(
        KEY_TO_CHAR.iter().map(|(k, v)| (*v, (*k, false)))
            .chain(SHIFT_KEY_TO_CHAR.iter().map(|(k, v)| (*v, (*k, true))))
            .chain(
                [
                    ('\n', (Key::KEY_ENTER, false)),
                    ('\t', (Key::KEY_TAB, false))
                ]
            )
    );
}

/// Used to signal if the event should be blocked or passed through to the virtual device.
pub enum EventFlag {
    /// Send the event to the device.
    Emit,
    /// Don't send the event to the device.
    Block,
}

/// Reprents the emulated terminal the user is typing into.
/// It keeps track of their inputs, controls the flow of events to the virtual device, constructs
/// the command string, runs it and types back the output.
pub struct Terminal {
    entry: Vec<char>,
    pos: usize,
    pub device: VirtualDevice,
}

impl Debug for Terminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Terminal {{ entry: {:?}, pos: {} }}",
            self.entry, self.pos
        )
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.clear()
    }
}

impl Terminal {
    /// Creates a new [`Terminal`].
    ///
    /// # Arguments
    ///
    /// * `device` - The [`VirtualDevice`] to use for sending events.
    pub fn new(device: VirtualDevice) -> Terminal {
        let mut term = Terminal {
            entry: Vec::new(),
            pos: 0,
            device,
        };
        // Write the > char
        term.init();
        term
    }

    /// Sends a key event.
    ///
    /// # Arguments
    ///
    /// * `key` - The [`Key`] to send.
    ///
    /// # Panics
    ///
    /// Panics if the events could not be emitted.
    pub fn send_key(&mut self, key: Key, shift: bool) {
        let events = if shift {
            vec![
                InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.code(), 1),
                InputEvent::new(EventType::KEY, key.code(), 1),
                InputEvent::new(EventType::KEY, key.code(), 0),
                InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.code(), 0),
            ]
        } else {
            vec![
                InputEvent::new(EventType::KEY, key.code(), 1),
                InputEvent::new(EventType::KEY, key.code(), 0),
            ]
        };
        self.device.emit(events.as_slice()).unwrap();
    }

    fn init(&mut self) {
        // Send the >< chars and move the cursor to the middle
        self.send_key(Key::KEY_DOT, true);
        self.send_key(Key::KEY_COMMA, true);
        self.send_key(Key::KEY_LEFT, false);
    }

    fn backspace(&mut self) -> EventFlag {
        if self.pos > 0 {
            self.pos -= 1;
            self.entry.remove(self.pos);
            return EventFlag::Emit;
        }
        EventFlag::Block
    }

    fn delete(&mut self) -> EventFlag {
        if self.pos < self.entry.len() {
            self.entry.remove(self.pos);
            return EventFlag::Emit;
        }
        EventFlag::Block
    }

    fn left(&mut self) -> EventFlag {
        if self.pos > 0 {
            self.pos -= 1;
            return EventFlag::Emit;
        }
        EventFlag::Block
    }

    fn right(&mut self) -> EventFlag {
        if self.pos < self.entry.len() {
            self.pos += 1;
            return EventFlag::Emit;
        }
        EventFlag::Block
    }

    fn home(&mut self) -> EventFlag {
        if self.pos > 0 {
            let n_lefts = self.pos;
            let left_events = [
                InputEvent::new(EventType::KEY, Key::KEY_LEFT.code(), 1),
                InputEvent::new(EventType::KEY, Key::KEY_LEFT.code(), 0),
            ]
            .repeat(n_lefts);
            log::trace!("home left events: {:?}", left_events);
            self.device.emit(left_events.as_slice()).unwrap();
            self.pos = 0;
        }
        // always block the event as we emulate the home by typing a bunch of left arrows
        EventFlag::Block
    }

    fn end(&mut self) -> EventFlag {
        if self.pos < self.entry.len() {
            let n_rights = self.entry.len() - self.pos;
            let right_events = [
                InputEvent::new(EventType::KEY, Key::KEY_RIGHT.code(), 1),
                InputEvent::new(EventType::KEY, Key::KEY_RIGHT.code(), 0),
            ]
            .repeat(n_rights);
            log::trace!("end right events: {:?}", right_events);
            self.device.emit(right_events.as_slice()).unwrap();
            self.pos = self.entry.len();
        }
        // always block the event as we emulate the end by typing a bunch of right arrows
        EventFlag::Block
    }

    fn add_char(&mut self, c: char) -> EventFlag {
        self.entry.insert(self.pos, c);
        self.pos += 1;
        EventFlag::Emit
    }

    fn get_entry(&self) -> String {
        self.entry.iter().collect()
    }

    /// Handle a key event.
    ///
    /// # Arguments
    ///
    /// * `key` - The [`Key`] that was pressed.
    /// * `shift` - Whether the shift key was pressed.
    pub fn handle_key(&mut self, key: Key, shift: bool) -> EventFlag {
        if shift {
            if let Some(c) = SHIFT_KEY_TO_CHAR.get(&key) {
                return self.add_char(*c);
            }
        } else if let Some(c) = KEY_TO_CHAR.get(&key) {
            return self.add_char(*c);
        }
        match key {
            Key::KEY_BACKSPACE => self.backspace(),
            Key::KEY_DELETE => self.delete(),
            Key::KEY_LEFT => self.left(),
            Key::KEY_RIGHT => self.right(),
            Key::KEY_END => self.end(),
            Key::KEY_HOME => self.home(),
            _ => EventFlag::Block,
        }
    }

    /// Run the command and return the stdout and stderr outputs.
    pub fn run(&mut self, uid: Option<u32>) -> Result<String, Box<dyn std::error::Error>> {
        let command = self.get_entry();

        log::info!("Running command: {}", command);
        let mut p = Popen::create(
            &["sh", "-c", &command],
            PopenConfig {
                stdout: Redirection::Pipe,
                stderr: Redirection::Pipe,
                setuid: uid,
                ..Default::default()
            },
        )?;

        p.wait()?;
        let (out, err) = p.communicate(None)?;

        Ok(format!(
            "{}{}",
            out.unwrap_or("".into()),
            err.unwrap_or("".into())
        ))
    }

    /// Clear the text by sending backspaces
    pub fn clear(&mut self) {
        // Move to the end of the entry
        self.end();
        // move one right for the last < char
        self.send_key(Key::KEY_RIGHT, false);

        let mut events = vec![];
        // Send the backspaces
        events.append(
            vec![
                InputEvent::new(EventType::KEY, Key::KEY_BACKSPACE.code(), 1),
                InputEvent::new(EventType::KEY, Key::KEY_BACKSPACE.code(), 0),
            ]
            .as_mut(),
        );
        // +2 for the >< chars
        events = events.repeat(self.entry.len() + 2);
        log::trace!("Clear BS events: {:?}", events);
        self.device.emit(events.as_slice()).unwrap();
    }

    /// Write the command output through the virtual device by sending the right key events.
    ///
    /// # Arguments
    ///
    /// * `contents`: The contents of the command output.
    pub fn write(&mut self, contents: String) {
        self.clear();
        let mut events = Vec::new();
        for c in contents.chars() {
            if let Some((key, shift)) = CHAR_TO_KEY.get(&c) {
                if *shift {
                    events.push(InputEvent::new(
                        EventType::KEY,
                        Key::KEY_LEFTSHIFT.code(),
                        1,
                    ));
                }
                events.push(InputEvent::new(EventType::KEY, key.code(), 1));
                events.push(InputEvent::new(EventType::KEY, key.code(), 0));
                if *shift {
                    events.push(InputEvent::new(
                        EventType::KEY,
                        Key::KEY_LEFTSHIFT.code(),
                        0,
                    ));
                }
            } else {
                log::warn!("No key found for char: {}", c);
            }
        }
        log::trace!("Write events: {:?}", events);
        self.device.emit(events.as_slice()).unwrap();
    }
}
