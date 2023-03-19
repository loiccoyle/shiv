use evdev::AttributeSet;
use evdev::InputEvent;
use evdev::Key;
use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum Modifier {
    Super,
    Alt,
    Control,
    Shift,
}

fn evdev_modifier_to_enum(key: Key) -> Option<Modifier> {
    match key {
        Key::KEY_LEFTCTRL | Key::KEY_RIGHTCTRL => Some(Modifier::Control),
        Key::KEY_LEFTSHIFT | Key::KEY_RIGHTSHIFT => Some(Modifier::Shift),
        Key::KEY_LEFTALT | Key::KEY_RIGHTALT => Some(Modifier::Alt),
        Key::KEY_LEFTMETA | Key::KEY_RIGHTMETA => Some(Modifier::Super),
        _ => None,
    }
}

#[derive(Debug)]
/// Keyboard state.
/// Keeps track of the current state of the keyboard's keys and modifiers.
pub struct Keyboard {
    modifiers: HashSet<Modifier>,
    keysyms: AttributeSet<evdev::Key>,
}

impl Keyboard {
    /// Create a new [`Keyboard`].
    ///
    /// # Arguments
    ///
    /// * `device` - The virtual device to emit events to.
    pub fn new() -> Keyboard {
        Keyboard {
            modifiers: HashSet::new(),
            keysyms: AttributeSet::new(),
        }
    }

    pub fn handle_event(&mut self, event: InputEvent, key: Key) {
        if let Some(modifier) = evdev_modifier_to_enum(key) {
            self.update_modifiers(event, modifier);
        } else {
            self.update_keysyms(event, key);
        }
    }

    fn update_keysyms(&mut self, event: InputEvent, key: Key) {
        if event.value() == 0 {
            // Key is released
            self.keysyms.remove(key);
        } else if event.value() == 1 {
            // Key is pressed
            self.keysyms.insert(key);
        }
    }

    fn update_modifiers(&mut self, event: InputEvent, modifier: Modifier) {
        if event.value() == 0 {
            // Key is released
            self.modifiers.remove(&modifier);
        } else if event.value() == 1 {
            // Key is pressed
            self.modifiers.insert(modifier);
        }
    }

    // helper functions
    pub fn is_ctrl_c(&self) -> bool {
        self.is_ctrl() && self.keysyms.contains(Key::KEY_C)
    }
    pub fn is_enter(&self) -> bool {
        self.keysyms.contains(Key::KEY_ENTER)
    }
    pub fn is_escape(&self) -> bool {
        self.keysyms.contains(Key::KEY_ESC)
    }

    // modifier query
    pub fn is_ctrl(&self) -> bool {
        self.modifiers.contains(&Modifier::Control)
    }
    pub fn is_shift(&self) -> bool {
        self.modifiers.contains(&Modifier::Shift)
    }
    // pub fn is_alt(&self) -> bool {
    //     self.modifiers.contains(&Modifier::Alt)
    // }
}

#[cfg(test)]
mod test {
    use super::*;
    use evdev::EventType;
    use evdev::InputEvent;
    use evdev::Key;

    #[test]
    fn test_keyboard_modifiers() {
        let mut keyboard = Keyboard::new();
        let event = InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 1);
        keyboard.handle_event(event, Key::KEY_LEFTCTRL);
        assert!(keyboard.is_ctrl());
        assert!(!keyboard.is_shift());
        assert!(!keyboard.is_ctrl_c());
        assert!(!keyboard.is_enter());
        assert!(!keyboard.is_escape());

        let event = InputEvent::new(EventType::KEY, Key::KEY_LEFTCTRL.code(), 0);
        keyboard.handle_event(event, Key::KEY_LEFTCTRL);
        assert!(!keyboard.is_ctrl());
    }

    #[test]
    fn test_keyboard_keysyms() {
        let mut keyboard = Keyboard::new();
        let event = InputEvent::new(EventType::KEY, Key::KEY_C.code(), 1);
        keyboard.handle_event(event, Key::KEY_C);
        assert!(keyboard.keysyms.contains(Key::KEY_C));
        assert!(!keyboard.keysyms.contains(Key::KEY_ENTER));
        assert!(!keyboard.keysyms.contains(Key::KEY_ESC));
        let event = InputEvent::new(EventType::KEY, Key::KEY_C.code(), 0);
        keyboard.handle_event(event, Key::KEY_C);
        assert!(!keyboard.keysyms.contains(Key::KEY_C));
    }
}
