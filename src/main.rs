use clap::Parser;
use evdev::Device;
use tokio_stream::{StreamExt, StreamMap};

mod cli;
mod keyboard;
mod permissions;
mod terminal;
mod uinput;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Arguments::parse();
    env_logger::Builder::new().filter_level(args.verbose.log_level_filter()).init();
    log::debug!("args: {:?}", args);

    let pre_cmd = shlex::split(&args.pre_cmd).ok_or("Failed to parse command")?;


    let uid = permissions::get_caller_uid()?;
    log::debug!("Caller UID: {}", uid);

    // setup uinput virtual device
    let uinput_device = match uinput::create_uinput_device() {
        Ok(device) => device,
        Err(err) => {
            log::error!("Failed to create uinput device: {}", err);
            std::process::exit(1);
        }
    };
    // fetch keyboards
    let keyboard_devices: Vec<Device> = evdev::enumerate()
        .map(|(_, device)| device)
        .filter(utils::check_device_is_keyboard)
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
    let terminal_config = terminal::TerminalConfig { pre_cmd };
    let mut keyboard = keyboard::Keyboard::new(uinput_device, terminal_config.into());

    log::info!("Listening for keyboard events...");
    log::info!("Ctrl-C/ESC to exit");
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
            break;
        } else if keyboard.is_enter() {
            log::info!("Enter detected, Running command and typing output...");
            if let Err(e) = permissions::drop_privileges(uid) {
                log::error!("Failed to drop privileges: {}", e);
                keyboard.terminal.clear();
            };
            log::debug!("Dropped privileges");
            let out = keyboard.terminal.run(uid);
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
            break;
        }
    }
    utils::release_keyboards();
    Ok(())
}
