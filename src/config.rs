use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "oniondownoda",
    version,
    about = "🧅 OnionDownOda — Download files from .onion URLs via Tor"
)]
pub struct CliArgs {
    /// SOCKS5 proxy address for Tor
    #[arg(short, long, default_value = "socks5h://127.0.0.1:9050")]
    pub proxy: String,

    /// Output directory for downloaded files
    #[arg(short, long, default_value = "./downloads")]
    pub output_dir: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct FileConfig {
    pub proxy: Option<String>,
    pub output_dir: Option<PathBuf>,
}

pub struct Config {
    pub proxy: String,
    pub output_dir: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let cli = CliArgs::parse();
        let file_config = dirs::config_dir()
            .map(|d| d.join("oniondownoda").join("config.toml"))
            .and_then(|p| std::fs::read_to_string(&p).ok())
            .and_then(|s| toml::from_str::<FileConfig>(&s).ok())
            .unwrap_or_default();

        Config {
            proxy: file_config.proxy.unwrap_or(cli.proxy),
            output_dir: file_config.output_dir.unwrap_or(cli.output_dir),
        }
    }
}
