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

    fn verbose_long_help() -> Option<&'static str> {
        None
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Decrease verbosity")
    }

    fn quiet_long_help() -> Option<&'static str> {
        None
    }
}

#[derive(Parser, Debug)]
#[clap(author = "Loic Coyle")]
#[command(version, about, long_about)]
/// Shiv: shell access everywhere.
///
/// Examples:
///
/// * On demand python shell:
/// $ shiv -p "python -c"
///
/// * Query GPT:
/// $ shiv -p "sgpt"
pub struct Arguments {
    /// Prefix input with this command
    #[clap(short = 'p', long, value_parser=validate_shell_cmd, default_value="bash -c")]
    pub pre_cmd: String,

    #[command(flatten)]
    pub verbose: Verbosity<Level>,
}
