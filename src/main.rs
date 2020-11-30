use anyhow::Result;
use cli::Command;

use config::Config;
use structopt::StructOpt;

pub mod cli;
pub mod config;

fn main() -> Result<()> {
    let cmd = Command::from_args();
    let mut config = Config::load()?;
    cmd.run(&mut config)?;
    Ok(())
}
