use clap::Parser;

// #[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = stac_siphon_rs::Args::parse();
    stac_siphon_rs::run(&args.dst, &args.src)
}
