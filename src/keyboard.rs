use evdev::uinput::VirtualDevice;
use evdev::AttributeSet;
use evdev::Key;
use std::collections::HashSet;
use std::fmt::Debug;

use crate::terminal::EntryStatus;
use crate::terminal::Terminal;

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

pub struct Keyboard {
    pub modifiers: HashSet<Modifier>,
    pub keysyms: AttributeSet<evdev::Key>,
    pub terminal: Terminal,
}

impl Debug for Keyboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keyboard")
            .field("modifiers", &self.modifiers)
            .field("keysyms", &self.keysyms)
            .field("terminal", &self.terminal)
            .finish()
    }
}

impl Keyboard {
    pub fn new(device: VirtualDevice) -> Keyboard {
        Keyboard {
            modifiers: HashSet::new(),
            keysyms: AttributeSet::new(),
            terminal: Terminal::new(device),
        }
    }

    pub fn handle_event(&mut self, event: &evdev::InputEvent) {
        if let evdev::InputEventKind::Key(key) = event.kind() {
            if let Some(modifier) = evdev_modifier_to_enum(key) {
                self.update_modifiers(event, modifier);
            } else {
                self.update_keysyms(event, key);
            }
        }
    }

    fn update_keysyms(&mut self, event: &evdev::InputEvent, key: Key) {
        if event.value() == 0 {
            self.keysyms.remove(key);
            self.terminal.device.emit(&[*event]).unwrap();
        } else {
            match self.terminal.handle_key(key, self.is_shift()) {
                EntryStatus::Change => {
                    println!("Entry change");
                    self.terminal.device.emit(&[*event]).unwrap();
                }
                EntryStatus::NoChange => {}
            }
            if event.value() == 1 {
                self.keysyms.insert(key);
            } else if event.value() == 2 {
                println!("Key repeat");
            }
        }
    }

    fn update_modifiers(&mut self, event: &evdev::InputEvent, modifier: Modifier) {
        if event.value() == 1 {
            self.modifiers.insert(modifier);
            // Pass through the shift modifier for capitals
            if self.is_shift() {
                self.terminal.device.emit(&[*event]).unwrap();
            }
        } else if event.value() == 0 {
            self.modifiers.remove(&modifier);
            self.terminal.device.emit(&[*event]).unwrap();
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
    pub fn is_alt(&self) -> bool {
        self.modifiers.contains(&Modifier::Alt)
    }
}
