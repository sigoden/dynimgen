use clap::Parser;

/// A self-hosted poster generator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to workdir
    #[clap(parse(from_os_str), name = "PATH")]
    pub workdir: Option<std::path::PathBuf>,

    /// Server bind port
    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Only urls matched allow-urls is allowd to be fetched
    #[clap(long = "allow-url", name = "URL")]
    pub allow_urls: Vec<String>,
}
