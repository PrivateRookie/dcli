use std::sync::{Arc, Mutex};

use anyhow::Result;
use cli::DCliCommand;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

use config::Config;
// use log::LevelFilter;
// use log4rs::{
//     append::file::FileAppender,
//     config::{Appender, Config as LogConfig, Root},
// };
use structopt::StructOpt;

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Translations;

pub mod cli;
pub mod config;
pub mod mysql;
pub mod utils;
pub mod output;

pub static LOADER: Lazy<Arc<Mutex<FluentLanguageLoader>>> = Lazy::new(|| {
    let translations = Translations {};
    let language_loader: FluentLanguageLoader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(&language_loader, &translations, &requested_languages);
    language_loader.set_use_isolating(false);
    Arc::new(Mutex::new(language_loader))
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::LOADER.lock().unwrap(), $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::LOADER.lock().unwrap(), $message_id, $($args), *)
    }};
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = DCliCommand::from_args();
    // init_log();
    let mut config = Config::load()?;
    if let Some(lang) = &config.lang {
        utils::reset_loader(lang)
    }
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
