use clap::Parser;
use clap_verbosity_flag::{LogLevel, Verbosity};

fn validate_shell_cmd(arg: &str) -> Result<String, String> {
    if shlex::split(arg).is_some() {
        Ok(arg.to_string())
    } else {
        Err(format!("Failed to parse command {}", arg))
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Level;

impl LogLevel for Level {
    fn default() -> Option<log::Level> {
        Some(log::Level::Info)
    }

    fn verbose_help() -> Option<&'static str> {
        Some("Increase verbosity")
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Decrease verbosity")
    }
}

#[derive(Parser, Debug)]
#[clap(author = "Loic Coyle")]
#[command(
    version,
    after_help = "Note: shiv requires priviledges to create and access keyboard devices.",
    verbatim_doc_comment
)]
/// Shiv: shell access everywhere.
///
/// Shiv allows you to run shell commands from any text box.
/// When started, it listens for keyboard inputs, on Enter it will run the command and write the output.
///
/// The recommended way to use shiv is to bind it to a key combination.
///
/// Examples:
///   • On demand python shell:
///     $ shiv "python -c"
///   • Query ChatGPT:
///     $ shiv "sgpt"
///   • On demand calculator and consersions:
///     $ shiv "qalc -t"
///   • ASCII art:
///     $ shiv "figlet"
pub struct Arguments {
    /// Prefix input with this command
    #[clap(value_parser=validate_shell_cmd, default_value = "bash -c")]
    pub pre_cmd: String,

    /// Type out the command output instead of pasting it
    #[clap(short = 'T', long)]
    pub type_output: bool,

    #[command(flatten)]
    pub verbose: Verbosity<Level>,
}
