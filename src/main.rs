use evdev::{Device, Key};
use tokio_stream::{StreamExt, StreamMap};
use env_logger::Env;

mod keyboard;
mod terminal;
mod uinput;
mod permissions;

/// Determine if a device is a keyboard.
///
/// # Arguments
///
/// * `device` - The device to check.
fn check_device_is_keyboard(device: &Device) -> bool {
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
fn release_keyboards() {
    get_keyboards().into_iter().for_each(|mut device| {
        let _ = device.ungrab();
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::init();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let uid = permissions::get_caller_uid()?;
    log::info!("Caller UID: {}", uid);

    // setup uinput virtual device
    let uinput_device = uinput::create_uinput_device()?;

    // fetch keyboards
    let keyboard_devices: Vec<Device> = evdev::enumerate()
        .map(|(_, device)| device)
        .filter(check_device_is_keyboard)
        .collect();


    log::info!("Found {} keyboards", keyboard_devices.len());
    for device in keyboard_devices.iter() {
        log::debug!("Device: {:?}", device.name());
    }

    let mut stream_map = StreamMap::new();
    // Grab the keyboards and feed their streams into `stream_map`.
    for (i, mut device) in keyboard_devices.into_iter().enumerate() {
        let _ = device.grab();
        stream_map.insert(i, device.into_event_stream()?);
    }
    // Setup keyboard
    let mut keyboard = keyboard::Keyboard::new(uinput_device);
    
    log::info!("Listening for events...");

    // Event loop
    while let Some((_, Ok(event))) = stream_map.next().await {
        // Event is passed to the keyboard class
        // It then passes it to the terminal class
        // The terminal class keeps track of the inputs and decides wether
        // to pass it to the virtual device or not
        keyboard.handle_event(&event);
        log::trace!("Keyboard state: {:?}", keyboard);

        if keyboard.is_ctrl_c() || keyboard.is_escape() {
            keyboard.terminal.clear();
            log::info!("Ctrl-C/ESC detected, exiting...");
            release_keyboards();
            break;
        } else if keyboard.is_enter() {
            log::info!("Enter detected, Running command and typing output...");
            permissions::drop_privileges(uid)?;
            log::info!("Dropped privileges");
            let out = keyboard.terminal.run(uid.into());
            match out {
                Ok(out) => {
                    log::info!("Command ran successfully");
                    keyboard.terminal.write(out);
                }
                Err(e) => {
                    log::error!("Command failed: {}", e);
                    keyboard.terminal.clear();
                    keyboard.terminal.write(format!("Command failed: {}", e));
                }
            }
            release_keyboards();
            break;
        }
    }

    Ok(())
}
