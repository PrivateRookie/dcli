use anyhow::Result;
use cli::DCliCommand;

use config::Config;
// use log::LevelFilter;
// use log4rs::{
//     append::file::FileAppender,
//     config::{Appender, Config as LogConfig, Root},
// };
use structopt::StructOpt;

pub mod cli;
pub mod config;
pub mod mysql;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = DCliCommand::from_args();
    // init_log();
    let mut config = Config::load()?;
    cmd.run(&mut config).await?;
    Ok(())
}

// fn init_log() {
//     let logfile = FileAppender::builder().build("dcli.log").unwrap();
//     let config = LogConfig::builder()
//         .appender(Appender::builder().build("logfile", Box::new(logfile)))
//         .build(Root::builder().appender("logfile").build(LevelFilter::Info))
//         .unwrap();
//     log4rs::init_config(config).unwrap();
// }
