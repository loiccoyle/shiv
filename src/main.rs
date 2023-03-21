use std::error::Error;

use clap::Parser;
use evdev::{Device, EventStream};
use tokio::spawn;
use tokio::sync::oneshot::{channel, Sender};
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
    // When a command is running, this will be set to Some.
    let mut abort_signal: Option<Sender<()>> = None;

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
                } else if abort_signal.is_none() {
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
                    if let Some(signal) = abort_signal {
                        log::info!("Killing running command");
                        signal.send(()).map_err(|_| "Failed to send abort signal")?;
                    }
                    terminal.clear()?;
                    break;
                } else if keyboard.is_enter() && abort_signal.is_none() {
                    permissions::drop_privileges(uid)?;
                    log::debug!("Dropped privileges");
                    let runner = terminal.clone();
                    let (send, recv) = channel::<()>();
                    abort_signal = Some(send);
                    spawn(async move {
                        let child = runner.run(uid).await;
                        match child {
                            Ok(mut task) => {
                                log::debug!("Child process spawned successfully");
                                tokio::select! {
                                    _ = task.wait() => {
                                        let output = task.wait_with_output().await.expect("Failed to wait on child");
                                        let contents = String::from_utf8_lossy(&output.stdout) +
                                            String::from_utf8_lossy(&output.stderr);
                                        runner
                                            .write(contents.into())
                                            .unwrap_or_else( |e| {
                                                log::error!("Failed to write output: {}", e);
                                                std::process::exit(1);
                                            });
                                        std::process::exit(0);
                                    }
                                    _ = recv => task.kill().await.expect("kill failed"),
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to spawn process: {}", e);
                                std::process::exit(1);
                            }
                        }
                    });
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
