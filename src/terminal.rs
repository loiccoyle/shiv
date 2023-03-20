use evdev::uinput::VirtualDevice;
use evdev::EventType;
use evdev::InputEvent;
use evdev::Key;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

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

    // This map could maybe use phf construct it at compile time
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

#[derive(Debug, Clone)]
pub enum OutputMethod {
    Paste,
    Type,
}

#[derive(Debug, Clone)]
/// Control the [`Terminal`]'s behavior.
pub struct TerminalConfig {
    /// Command to which the user input is used as argument.
    pub pre_cmd: Vec<String>,
    pub output_method: OutputMethod,
    pub key_delay: Option<std::time::Duration>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            pre_cmd: vec!["bash".to_string(), "-c".to_string()],
            output_method: OutputMethod::Paste,
            key_delay: None,
        }
    }
}

#[derive(Clone)]
/// Represents the emulated terminal the user is typing into.
/// It keeps track of their inputs, controls the flow of events to the virtual device, constructs
/// the command string, runs it and types back the output.
pub struct Terminal {
    entry: Vec<char>,
    pos: usize,
    pub device: Arc<Mutex<VirtualDevice>>,
    config: TerminalConfig,
}

impl Debug for Terminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Terminal {{ entry: {:?}, pos: {}, config: {:?} }}",
            self.entry, self.pos, self.config
        )
    }
}

impl Terminal {
    /// Creates a new [`Terminal`].
    ///
    /// # Arguments
    ///
    /// * `device` - The [`VirtualDevice`] to use for sending events.
    pub fn new(device: VirtualDevice, config: TerminalConfig) -> Result<Terminal, Box<dyn Error>> {
        let term = Terminal {
            entry: Vec::new(),
            pos: 0,
            device: Arc::new(Mutex::new(device)),
            config,
        };
        // Write the >< chars
        term.init()?;
        Ok(term)
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
    pub fn send_key(&self, key: Key, shift: bool) -> Result<(), Box<dyn Error>> {
        let events = self.key_events(key, shift);
        self.send_events(events)
    }

    fn key_events(&self, key: Key, shift: bool) -> Vec<InputEvent> {
        if shift {
            vec![
                InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.code(), 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                InputEvent::new(EventType::KEY, key.code(), 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                InputEvent::new(EventType::KEY, key.code(), 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.code(), 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]
        } else {
            vec![
                InputEvent::new(EventType::KEY, key.code(), 1),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
                InputEvent::new(EventType::KEY, key.code(), 0),
                InputEvent::new(EventType::SYNCHRONIZATION, 0, 0),
            ]
        }
    }

    fn init(&self) -> Result<(), Box<dyn Error>> {
        // Send the >< chars and move the cursor to the middle
        self.send_key(Key::KEY_DOT, true)?;
        self.send_key(Key::KEY_COMMA, true)?;
        self.send_key(Key::KEY_LEFT, false)?;
        Ok(())
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

    fn home(&mut self) -> Result<EventFlag, Box<dyn Error>> {
        if self.pos > 0 {
            let n_lefts = self.pos;
            let left_events = self.key_events(Key::KEY_LEFT, false).repeat(n_lefts);
            log::trace!("home left events: {:?}", left_events);
            self.send_events(left_events)?;
            self.pos = 0;
        }
        // always block the event as we emulate the home by typing a bunch of left arrows
        Ok(EventFlag::Block)
    }

    fn end(&mut self) -> Result<EventFlag, Box<dyn Error>> {
        if self.pos < self.entry.len() {
            let right_events = self.end_events();
            log::trace!("end right events: {:?}", right_events);
            self.send_events(right_events)?;
            self.pos = self.entry.len();
        }
        // always block the event as we emulate the end by typing a bunch of right arrows
        Ok(EventFlag::Block)
    }

    fn end_events(&self) -> Vec<InputEvent> {
        let n_rights = self.entry.len() - self.pos;
        self.key_events(Key::KEY_RIGHT, false).repeat(n_rights)
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
    pub fn handle_key(&mut self, key: Key, shift: bool) -> Result<EventFlag, Box<dyn Error>> {
        if shift {
            if let Some(c) = SHIFT_KEY_TO_CHAR.get(&key) {
                return Ok(self.add_char(*c));
            }
        } else if let Some(c) = KEY_TO_CHAR.get(&key) {
            return Ok(self.add_char(*c));
        }
        match key {
            Key::KEY_BACKSPACE => Ok(self.backspace()),
            Key::KEY_DELETE => Ok(self.delete()),
            Key::KEY_LEFT => Ok(self.left()),
            Key::KEY_RIGHT => Ok(self.right()),
            Key::KEY_END => self.end(),
            Key::KEY_HOME => self.home(),
            _ => Ok(EventFlag::Block),
        }
    }

    /// Run the command and return the stdout and stderr outputs.
    ///
    /// # Arguments
    ///
    /// * `uid` - The user id to run the command as.
    pub async fn run(&self, uid: u32) -> Result<String, Box<dyn Error>> {
        let mut command = Command::new("sudo");
        command
            .args(["-u", &format!("#{}", uid), "-i", "--"])
            .args(&self.config.pre_cmd)
            .arg(self.get_entry());
        log::info!("Running command: {:?}", &command);
        let output = command.output()?;

        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        Ok((out + err).to_string())
    }

    /// Clear the input line. By sending backspace and delete events.
    pub fn clear(&self) -> Result<(), Box<dyn Error>> {
        self.send_events(self.clear_events())
    }

    /// Generate the clear events.
    pub fn clear_events(&self) -> Vec<InputEvent> {
        // The delete events
        // +1 for the < char
        let n_to_right = self.entry.len() - self.pos + 1;
        let mut events = self.key_events(Key::KEY_DELETE, false).repeat(n_to_right);

        // The backspace events
        // + 1 for the > chars
        events.extend_from_slice(
            &self
                .key_events(Key::KEY_BACKSPACE, false)
                .repeat(self.pos + 1),
        );
        events
    }

    /// Write the command output.
    ///
    /// # Arguments
    ///
    /// * `contents`: The contents of the command output.
    pub fn write(&self, contents: String) -> Result<(), Box<dyn Error>> {
        log::info!("Writing contents: {}", contents);
        let clear_event = self.clear_events();
        if !contents.is_empty() {
            match self.config.output_method {
                OutputMethod::Type => self.write_type(contents, Some(clear_event)),
                OutputMethod::Paste => self.write_paste(contents, Some(clear_event)),
            }
        } else {
            self.send_events(clear_event)
        }
    }

    /// Write the command output through the virtual device by sending the right key events.
    ///
    /// # Arguments
    ///
    /// * `contents`: The contents of the command output.
    /// * `prev_events`: Append to these events and send all at once.
    pub fn write_type(
        &self,
        contents: String,
        prev_events: Option<Vec<InputEvent>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut events = prev_events.unwrap_or(Vec::new());

        for c in contents.chars() {
            if let Some((key, shift)) = CHAR_TO_KEY.get(&c) {
                events.extend_from_slice(&self.key_events(*key, *shift));
            } else {
                log::warn!("No key found for char: {}", c);
            }
        }
        log::trace!("Write events: {:?}", events);

        self.send_events(events)
    }

    /// Write the command output through the clipboard.
    ///
    /// # Arguments
    ///
    /// * `contents`: The contents of the command output.
    /// * `prev_events`: Append to these events and send all at once.
    fn write_paste(
        &self,
        contents: String,
        prev_events: Option<Vec<InputEvent>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut events = prev_events.unwrap_or(Vec::new());

        events.extend_from_slice(&self.key_events(Key::KEY_PASTE, false));
        log::trace!("Paste events: {:?}", events);

        let mut clipboard = arboard::Clipboard::new()?;
        let cp_data = clipboard.get_text().ok();
        log::debug!("Clipboard data: {:?}", cp_data);
        clipboard.set_text(contents)?;
        // Paste the contents
        self.send_events(events)
        // We need to wait for the key to register before resetting the clipboard
        // reset the clipboard in a new thread
        // if let Some(data) = cp_data {
        //     log::debug!("Resetting clipboard to: {}", data);
        //     clipboard.set_text(data).unwrap();
        // }
    }

    fn send_events(&self, events: Vec<InputEvent>) -> Result<(), Box<dyn Error>> {
        if let Some(delay) = self.config.key_delay {
            // 2 by 2 to send the SYNCHRONIZATION report along with the key
            for events in events.chunks(2) {
                self.emit(events)?;
                std::thread::sleep(delay);
            }
        } else {
            self.emit(events.as_slice())?;
        }
        Ok(())
    }

    pub fn emit(&self, events: &[InputEvent]) -> Result<(), Box<dyn Error>> {
        self.device.clone().lock().unwrap().emit(events)?;
        Ok(())
    }
}
