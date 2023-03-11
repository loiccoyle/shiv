use evdev::{Device, Key};
use tokio;
use tokio_stream::{StreamExt, StreamMap};

mod uinput;
mod keyboard;
mod terminal;

/// Determine if a device is a keyboard.
fn check_device_is_keyboard(device: &Device) -> bool {
    if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        if device.name() == Some(uinput::UINPUT_DEVICE_NAME) || ! device.name().unwrap().to_lowercase().contains("keyboard") {
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

    // setup uinput virtual device
    let uinput_device = uinput::create_uinput_device()?;

    // fetch keyboards
    let keyboard_devices: Vec<Device> = evdev::enumerate()
        .map(|(_, device)| device)
        .filter(check_device_is_keyboard)
        .collect();

    // Setup keyboard
    let mut keyboard = keyboard::Keyboard::new(uinput_device);

    println!("Found {} keyboard devices", keyboard_devices.len());
    for device in keyboard_devices.iter() {
        println!("Device: {:?}", device.name());
    }
    let mut stream_map = StreamMap::new();

    for (i, mut device) in keyboard_devices.into_iter().enumerate() {
        let _ = device.grab();
        stream_map.insert(i, device.into_event_stream()?);
    }
    println!("Listening for events...");
    println!("{:?} streams", stream_map.len());

    while let Some((_, Ok(event))) = stream_map.next().await {

        keyboard.handle_event(&event);
        // uinput_device.emit(&[event]).unwrap();
        println!("Keyboard state: {:?}", keyboard);

        if keyboard.is_ctrl_c() || keyboard.is_escape() {
            keyboard.terminal.clear();
            println!("Ctrl-C/ESC detected, exiting...");
            release_keyboards();
            break;
        } else if keyboard.is_enter() {
            let out = keyboard.terminal.run();
            keyboard.terminal.write(out);
            println!("Enter detected, Running command and exiting");
            release_keyboards();
            break;
        }
    }

    Ok(())
}
