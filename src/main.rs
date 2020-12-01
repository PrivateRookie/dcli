use anyhow::Result;
use cli::Command;

use config::Config;
use structopt::StructOpt;

pub mod cli;
pub mod config;
pub mod mysql;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = Command::from_args();
    let mut config = Config::load()?;
    pretty_env_logger::init();
    cmd.run(&mut config).await?;
    Ok(())
}
