use clap::Parser;

fn command_split(arg: &str) -> Result<String, String> {
    if shlex::split(arg).is_some() {
        Ok(arg.to_string())
    } else {
        Err(format!("Failed to parse command {}", arg))
    }
}

#[derive(Parser, Debug)]
#[clap(author = "Loic Coyle")]
#[command(author, version, about, long_about = None)]
/// Shiv: shell access everywhere.
pub struct Arguments {
    /// Prefix input with this command
    #[clap(short = 'p', long, value_parser=command_split, default_value="bash -c")]
    pub pre_cmd: String,
}
