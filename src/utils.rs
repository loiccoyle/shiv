use evdev::{Device, Key};

use crate::uinput;
/// Determine if a device is a keyboard.
///
/// # Arguments
///
/// * `device` - The device to check.
pub fn check_device_is_keyboard(device: &Device) -> bool {
    if device
        .supported_keys()
        .map_or(false, |keys| keys.contains(Key::KEY_ENTER))
    {
        if device.name() == Some(uinput::UINPUT_DEVICE_NAME)
            || !device.name().unwrap().to_lowercase().contains("keyboard")
        {
            return false;
        }
        true
    } else {
        false
    }
}

/// Get a list of all keyboards.
fn get_keyboards() -> Vec<Device> {
    evdev::enumerate()
        .map(|(_, device)| device)
        .filter(check_device_is_keyboard)
        .collect()
}

/// Release the keyboards.
pub fn release_keyboards() {
    get_keyboards().into_iter().for_each(|mut device| {
        let _ = device.ungrab();
    });
}
