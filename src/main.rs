use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use universal_archiver::format::{parse_format, FileFormat};

#[derive(Debug, Clone, Parser)]
#[clap(name="universal-archiver", version=env!("CARGO_PKG_VERSION"), about=env!("CARGO_PKG_DESCRIPTION"))]
struct Args {
    /// The operation to perform
    #[clap(subcommand)]
    pub operation: Operation,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Operation {
    /// Extracts a given file
    Extract {
        /// The file to extract
        file: PathBuf,

        /// The output folder for the given file
        output: Option<PathBuf>,
    },
}

fn main() {
    color_eyre::install().unwrap();
    init_logger();
    let args: Args = Args::parse();
    match args.operation {
        Operation::Extract { output, file } => {
            let format = parse_format(&file).expect("Failed to parse file format");
            let output = output.unwrap_or_else(|| {
                file.file_stem()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("out"))
            });
            format
                .extract(&file, &output)
                .expect("Failed to extract file");
        }
    }
}

fn init_logger() {
    const DEFAULT_ENV_FILTER: &str = "info";
    let filter_string =
        std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_ENV_FILTER.to_string());
    let env_filter =
        EnvFilter::from_str(&*filter_string).expect("failed to parse env filter string");
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(env_filter)
        .with_writer(std::io::stdout)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .compact()
        .init();
}
