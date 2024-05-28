#[derive(clap::Parser)]
#[command(about = "A pleb-friendly Bitcoin mining pool.", author = env!("CARGO_PKG_AUTHORS"), version = env!("CARGO_PKG_VERSION"))]
pub struct CLIArgs {
    #[arg(
        short,
        long,
        help = "Use the <file name> as the location of the config file",
        default_value = "config.toml",
        required = false
    )]
    pub config: String,
}
