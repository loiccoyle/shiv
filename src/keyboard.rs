use evdev::uinput::VirtualDevice;
use evdev::AttributeSet;
use evdev::Key;
use std::collections::HashSet;
use std::fmt::Debug;

use crate::terminal::EventFlag;
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

#[derive(Debug)]
/// Keyboard state.
/// Keeps track of the current state of the keyboard's keys and modifiers.
///
/// It feeds key events to the underlying [`Terminal`] and, if flagged, emits the events to the
/// terminal's virtual device.
pub struct Keyboard {
    pub modifiers: HashSet<Modifier>,
    pub keysyms: AttributeSet<evdev::Key>,
    pub terminal: Terminal,
}

impl Keyboard {
    /// Create a new [`Keyboard`].
    ///
    /// # Arguments
    ///
    /// * `device` - The virtual device to emit events to.
    pub fn new(device: VirtualDevice) -> Keyboard {
        Keyboard {
            modifiers: HashSet::new(),
            keysyms: AttributeSet::new(),
            terminal: Terminal::new(device),
        }
    }

    pub fn handle_event(&mut self, event: &evdev::InputEvent) {
        match event.kind() {
            evdev::InputEventKind::Key(key) => {
                if let Some(modifier) = evdev_modifier_to_enum(key) {
                    self.update_modifiers(event, modifier);
                } else {
                    self.update_keysyms(event, key);
                }
            }
            evdev::InputEventKind::Synchronization(_) => {
                self.terminal.device.emit(&[*event]).unwrap()
            }
            _ => {}
        }
    }

    fn update_keysyms(&mut self, event: &evdev::InputEvent, key: Key) {
        if event.value() == 0 {
            // Key is released
            self.keysyms.remove(key);
            self.terminal.device.emit(&[*event]).unwrap();
        } else {
            match self.terminal.handle_key(key, self.is_shift()) {
                EventFlag::Emit => {
                    log::debug!("Entry change");
                    log::info!("Emitting {:?}", event);
                    // here we emit the event as a single key press regardless of if it was a held down
                    // key or not. This is because we are not handling key repeats. And allows the
                    // grabbed keyboard to decide the rates of the virtual device.
                    self.terminal.send_key(key, self.is_shift())
                }
                EventFlag::Block => {}
            }
            if event.value() == 1 {
                // Key is pressed
                self.keysyms.insert(key);
            } else if event.value() == 2 {
                // Key is repeated
                log::debug!("Key repeat");
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
    // pub fn is_alt(&self) -> bool {
    //     self.modifiers.contains(&Modifier::Alt)
    // }
}
