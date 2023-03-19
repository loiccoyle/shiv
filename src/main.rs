use clap::Parser;
use evdev::Device;
use tokio::{spawn, task::JoinHandle};
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
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
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

    if keyboard_devices.is_empty() {
        log::error!("No keyboard found");
        std::process::exit(1);
    }

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
    let terminal_config = terminal::TerminalConfig {
        pre_cmd,
        output_method: if args.type_output {
            terminal::OutputMethod::Type
        } else {
            terminal::OutputMethod::Paste
        },
        key_delay: args.key_delay,
    };
    let mut keyboard = keyboard::Keyboard::new();
    let mut terminal = terminal::Terminal::new(uinput_device, terminal_config);
    let mut cmd_task: Option<JoinHandle<()>> = None;

    log::info!("Listening for keyboard events...");
    log::info!("Ctrl-C/ESC to exit");
    // Event loop
    while let Some((_, Ok(event))) = stream_map.next().await {
        // Event is passed to the keyboard class.
        // It is then passed to the terminal class.
        // The terminal class keeps track of the inputs and decides wether
        // to pass it to the virtual device or not.
        log::trace!("Event: {:?}", event);
        log::trace!("Keyboard state: {:?}", keyboard);
        match event.kind() {
            evdev::InputEventKind::Key(key) => {
                keyboard.handle_event(event, key);
                if event.value() == 0 {
                    // Re-emit all key releases
                    terminal.emit(&[event]).unwrap();
                } else if cmd_task.is_none() {
                    // don't update the terminal state if cmd is running
                    // Re-emit key presses based on the terminal state and capabilities
                    match terminal.handle_key(key, keyboard.is_shift()) {
                        terminal::EventFlag::Emit => {
                            log::debug!("Passing through {:?}", event);
                            // here we emit the event as a single key press regardless of if it was a held down
                            // key or not. This is because we are not handling key repeats. And allows the
                            // grabbed keyboard to decide the rates of the virtual device.
                            terminal.send_key(key, keyboard.is_shift())
                        }
                        terminal::EventFlag::Block => {}
                    }
                }

                if keyboard.is_ctrl_c() || keyboard.is_escape() {
                    log::info!("Ctrl-C/ESC detected, exiting...");
                    if let Some(task) = cmd_task {
                        log::info!("Killing running command");
                        task.abort();
                    }
                    utils::release_keyboards();
                    terminal.clear();
                    break;
                } else if keyboard.is_enter() && cmd_task.is_none() {
                    log::info!("Enter detected, running command and writing output...");
                    if let Err(e) = permissions::drop_privileges(uid) {
                        log::error!("Failed to drop privileges: {}", e);
                        terminal.clear();
                    };
                    log::debug!("Dropped privileges");
                    let mut runner = terminal.clone();
                    cmd_task = Some(spawn(async move {
                        let out = runner.run(uid).await;
                        // print the memmory address
                        match out {
                            Ok(out) => {
                                log::info!("Command ran successfully");
                                runner.write(out);
                                utils::release_keyboards();
                                std::process::exit(0);
                            }

                            Err(e) => {
                                log::error!("Command failed: {}", e);
                                runner.write(format!("Command failed: {}", e));
                                utils::release_keyboards();
                                std::process::exit(1);
                            }
                        }
                    }));
                }
            }
            evdev::InputEventKind::Synchronization(_) => terminal.emit(&[event]).unwrap(),
            _ => {}
        }
    }
    Ok(())
}
