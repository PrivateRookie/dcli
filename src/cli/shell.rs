use colored::*;
use std::{
    borrow::Cow::{self, Borrowed, Owned},
    collections::{HashMap, HashSet},
};

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{Cmd, CompletionType, Config, Context, EditMode, Editor, KeyEvent};
use rustyline_derive::Helper;

#[derive(Helper)]
pub struct MyHelper {
    pub completer: FilenameCompleter,
    pub highlighter: FooHighlighter,
    // pub highlighter: MatchingBracketHighlighter,
    pub validator: MatchingBracketValidator,
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
}

#[derive(Debug)]
pub struct DataBaseCompleter {
    pub database: HashSet<String>,
    pub tables: HashSet<String>,
    pub columns: HashMap<String, HashSet<String>>,
}

#[derive(Debug)]
pub struct FooHighlighter {}

impl Highlighter for FooHighlighter {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        if let Some(idx) = line.find("SELECT") {
            let mut copy = line.to_owned();
            if idx + 6 > copy.len() {
                copy.replace_range(idx.., &"SELECT"[..(copy.len() - idx)].red().to_string());
            } else {
                copy.replace_range(idx..(idx + 6), &"SELECT".red().to_string());
            }
            Owned(copy)
        } else {
            Borrowed(line)
        }
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
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

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        true
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
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
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

pub fn get_editor() -> Editor<MyHelper> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();
    let helper = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: FooHighlighter {},
        // highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_string(),
        validator: MatchingBracketValidator::new(),
    };
    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(helper));
    rl.bind_sequence(KeyEvent::alt('N'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('P'), Cmd::HistorySearchBackward);
    rl
}

#[test]
fn test_sql() {
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;
    let sql = "SELECT a, b";
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();
    dbg!(ast);
}
