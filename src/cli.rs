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
    after_help = "Note: This program requires priviledges to create and access keyboard devices."
)]
/// Shiv: shell access everywhere.
///
///
/// Examples:
///
/// * On demand python shell:
/// $ shiv -p "python -c"
///
/// * Query GPT:
/// $ shiv -p "sgpt"
///
/// * On demand calculator and consersions:
/// $ shiv -p "qalc -t"
pub struct Arguments {
    /// Prefix input with this command
    #[clap(short = 'p', long, value_parser=validate_shell_cmd, default_value="bash -c")]
    pub pre_cmd: String,

    #[command(flatten)]
    pub verbose: Verbosity<Level>,
}
