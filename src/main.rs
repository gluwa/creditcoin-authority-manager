use clap::Parser;
use color_eyre::Result;
use creditcoin_authority_manager::commands::Commands;
use creditcoin_authority_manager::Run;
use subxt::{ClientBuilder, DefaultConfig};

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    url: String,
    #[clap(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    sp_tracing::try_init_simple();

    let cli = Cli::parse();
    let client = ClientBuilder::new()
        .set_url(&cli.url)
        .build::<DefaultConfig>()
        .await?
        .to_runtime_api();
    cli.command.run(&client).await?;

    Ok(())
}
