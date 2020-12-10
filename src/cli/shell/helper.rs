use colored::*;
use sqlparser::dialect::MySqlDialect;
use sqlx::MySqlPool;
use std::{
    borrow::Cow::{self, Borrowed, Owned},
    collections::{HashMap, HashSet},
};

use crate::mysql::{all_columns, all_databases, all_tables};

use super::highlight::{MonoKaiSchema, SQLHighLight, Schema};
use rustyline::completion::{Completer, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{self, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::Helper;

#[derive(Helper)]
pub struct MyHelper {
    pub databases: HashSet<String>,
    pub tables: HashSet<String>,
    pub columns: HashMap<String, HashSet<String>>,
    pub highlighter: DBHighlighter,
    pub colored_prompt: String,
}

#[derive(Debug)]
pub struct DBHighlighter {}

impl Highlighter for DBHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let dialect = MySqlDialect {};
        let schema = MonoKaiSchema {};
        let rendered = match sqlparser::tokenizer::Tokenizer::new(&dialect, line).tokenize() {
            Ok(tokens) => tokens
                .iter()
                .map(|t| t.render(&schema))
                .collect::<Vec<String>>()
                .join(""),
            Err(_) => line.to_string(),
        };
        Owned(rendered)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        let mut copy = prompt.to_owned();
        copy.replace_range(.., &"HIGT".red().to_string());
        Owned(copy)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Borrowed(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: CompletionType,
    ) -> Cow<'c, str> {
        let _ = completion;
        Borrowed(candidate)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let pattern = line[..pos]
            .split_ascii_whitespace()
            .last()
            .unwrap_or(&line[..pos]);
        let mut pairs: Vec<Pair> = vec![];

        for kw in crate::mysql::KEYWORDS.iter() {
            if kw.starts_with(&pattern.to_ascii_uppercase()) {
                pairs.push(Pair {
                    display: format!("{} {}", "[KEY]".color(MonoKaiSchema::red()), kw),
                    replacement: kw.to_string(),
                })
            }
        }

        for db in self.databases.iter() {
            if db.contains(pattern) {
                pairs.push(Pair {
                    display: format!("{} {}", "[DB]".color(MonoKaiSchema::cyan()), db),
                    replacement: db.to_string(),
                })
            }
        }

        for tab in self.tables.iter() {
            if tab.contains(pattern) {
                pairs.push(Pair {
                    display: format!("{} {}", "[TABLE]".color(MonoKaiSchema::purple()), tab),
                    replacement: tab.to_string(),
                })
            }
        }

        for (tab, cols) in self.columns.iter() {
            for col in cols.iter() {
                if col.contains(pattern) {
                    pairs.push(Pair {
                        display: format!(
                            "{} {}.{}",
                            "[COL]".color(MonoKaiSchema::blue()),
                            tab,
                            col
                        ),
                        replacement: col.to_string(),
                    })
                }
            }
        }

        let idx = line.find(pattern).unwrap_or(0);
        Ok((idx, pairs))
    }
}

impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for MyHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        let input = ctx.input();
        if !input.starts_with("%") {
            if input.chars().all(|c| c.is_whitespace()) || input.ends_with(";") {
                Ok(validate::ValidationResult::Valid(None))
            } else {
                Ok(validate::ValidationResult::Incomplete)
            }
        } else {
            Ok(validate::ValidationResult::Valid(None))
        }
    }
}

pub async fn get_editor(mut conn: &mut MySqlPool) -> anyhow::Result<Editor<MyHelper>> {
    let databases = all_databases(&mut conn).await?;
    let tables = all_tables(&mut conn).await?;
    let columns = all_columns(&mut conn, &tables).await?;
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();
    let helper = MyHelper {
        databases,
        tables,
        columns,
        highlighter: DBHighlighter {},
        colored_prompt: "".to_string(),
    };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(helper));
    Ok(rl)
}
