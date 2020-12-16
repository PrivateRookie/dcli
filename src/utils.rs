use crate::{config::Lang, fl, Translations};
use anyhow::{Context, Result};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    unic_langid::LanguageIdentifier,
};
use std::io::Read;

pub fn read_file(path: &str) -> Result<String> {
    let mut file =
        std::fs::File::open(path).with_context(|| fl!("open-file-failed", file = path))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .with_context(|| fl!("read-file-failed", file = path))?;
    Ok(content)
}

pub fn reset_loader(lang: &Lang) {
    let lang = match lang {
        Lang::ZhCN => lang.to_string().parse::<LanguageIdentifier>().unwrap(),
        Lang::EnUS => lang.to_string().parse::<LanguageIdentifier>().unwrap(),
    };
    let translations = Translations {};
    let language_loader: FluentLanguageLoader = fluent_language_loader!();
    let _result = i18n_embed::select(&language_loader, &translations, &[lang]);
    language_loader.set_use_isolating(false);
    *crate::LOADER.lock().unwrap() = language_loader
}
