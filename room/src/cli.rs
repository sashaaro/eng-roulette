use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Number of times to greet
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,
}