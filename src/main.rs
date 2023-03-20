use std::{error::Error, time::Duration};

use clap::Parser;
use evdev::{Device, EventStream};
use tokio::{spawn, task::JoinHandle, time::sleep};
use tokio_stream::{StreamExt, StreamMap};

mod cli;
mod keyboard;
mod permissions;
mod terminal;
mod uinput;
mod utils;

async fn handle_events(
    uid: u32,
    mut keyboard: keyboard::Keyboard,
    mut terminal: terminal::Terminal,
    mut stream_map: StreamMap<usize, EventStream>,
) -> Result<(), Box<dyn Error>> {
    let mut cmd_task: Option<JoinHandle<()>> = None;
    log::info!("Listening for keyboard events...");
    log::info!("Ctrl-C/ESC to exit");
    // Event loop
    while let Some((_, Ok(event))) = stream_map.next().await {
        // Event is passed to the keyboard class.
        // It is then passed to the terminal class.
        // The keyboard class keeps track of the state of the keyboard.
        // The terminal class keeps track of the inputs and decides wether
        // to pass it to the virtual device or not.
        log::trace!("Event: {:?}", event);
        log::trace!("Keyboard state: {:?}", keyboard);
        match event.kind() {
            evdev::InputEventKind::Key(key) => {
                keyboard.handle_event(event, key);
                if event.value() == 0 {
                    // Re-emit all key releases
                    terminal.emit(&[event]).unwrap_or_else(|e| {
                        log::error!("Failed to emit key: {}", e);
                    });
                } else if cmd_task.is_none() {
                    // don't update the terminal state if cmd is running
                    // Re-emit key presses based on the terminal state and capabilities
                    match terminal.handle_key(key, keyboard.is_shift())? {
                        terminal::EventFlag::Emit => {
                            log::debug!("Passing through {:?}", event);
                            // here we emit the event as a single key press regardless of if it was a held down
                            // key or not. This is because we are not handling key repeats. And allows the
                            // grabbed keyboard to decide the rates of the virtual device.
                            terminal.send_key(key, keyboard.is_shift())?
                        }
                        terminal::EventFlag::Block => {}
                    }
                }

                if keyboard.is_ctrl_c() || keyboard.is_escape() {
                    log::info!("Ctrl-C/ESC detected, exiting...");
                    if let Some(task) = cmd_task {
                        log::info!("Killing running command");
                        // TODO: this does not actually stop the command from running
                        task.abort();
                    }
                    terminal.clear()?;
                    break;
                } else if keyboard.is_enter() && cmd_task.is_none() {
                    permissions::drop_privileges(uid)?;
                    log::debug!("Dropped privileges");
                    let runner = terminal.clone();
                    cmd_task = Some(spawn(async move {
                        let out = runner.run(uid).await;
                        // TODO: improve this, this is a hack to allow for the task to be aborted
                        sleep(Duration::from_millis(10)).await;
                        match out {
                            Ok(out) => {
                                log::info!("Command ran successfully");
                                runner.write(out).unwrap_or_else(|e| {
                                    log::error!("Failed to write output: {}", e);
                                });
                                // TODO: It would be good to handle these exits in the main function
                                utils::release_keyboards();
                                std::process::exit(0);
                            }
                            Err(e) => {
                                log::error!("Command failed: {}", e);
                                runner
                                    .write(format!("Command failed: {}", e))
                                    .unwrap_or_else(|e| {
                                        log::error!("Failed to write output: {}", e);
                                    });
                                // TODO: It would be good to handle these exits in the main function
                                utils::release_keyboards();
                                std::process::exit(1);
                            }
                        }
                    }));
                }
            }
            evdev::InputEventKind::Synchronization(_) => terminal.emit(&[event])?,
            _ => {}
        }
    }
    Ok(())
}

async fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Arguments::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();
    log::debug!("args: {:?}", args);

    let pre_cmd = shlex::split(&args.pre_cmd).ok_or("Failed to parse command")?;

    let uid = permissions::get_caller_uid()?;
    log::debug!("Caller UID: {}", uid);

    // setup uinput virtual device
    let virt_device = uinput::create_uinput_device()?;

    // fetch keyboards
    let keyboard_devices: Vec<Device> = evdev::enumerate()
        .map(|(_, device)| device)
        .filter(utils::check_device_is_keyboard)
        .collect();

    if keyboard_devices.is_empty() {
        return Err("No keyboard found".into());
    }

    log::info!("Found {} keyboards", keyboard_devices.len());
    if log::log_enabled!(log::Level::Debug) {
        for device in keyboard_devices.iter() {
            log::debug!("Device: {:?}", device.name());
        }
    }

    let mut stream_map = StreamMap::new();
    // Grab the keyboards and feed their streams into `stream_map`.
    for (i, mut device) in keyboard_devices.into_iter().enumerate() {
        device.grab()?;
        stream_map.insert(i, device.into_event_stream()?);
    }
    let config = terminal::TerminalConfig {
        pre_cmd,
        output_method: if args.type_output {
            terminal::OutputMethod::Type
        } else {
            terminal::OutputMethod::Paste
        },
        key_delay: args.key_delay,
    };
    let keyboard = keyboard::Keyboard::new();
    let terminal = terminal::Terminal::new(virt_device, config)?;
    handle_events(uid, keyboard, terminal, stream_map).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let status = _main().await;
    utils::release_keyboards();
    status.unwrap_or_else(|e| {
        log::error!("Failed to run: {}", e);
        std::process::exit(1);
    });
}
