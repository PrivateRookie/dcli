use colored::*;
use sqlparser::{
    dialect::keywords::Keyword,
    tokenizer::{Token, Word},
};

pub trait SQLHighLight {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String;
}

impl SQLHighLight for Token {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        match self {
            Token::EOF => "EOF".color(S::red()).to_string(),
            Token::Word(w) => w.render(schema),
            Token::Number(n) => n.color(S::green()).to_string(),
            Token::Char(c) => c.to_string(),
            Token::SingleQuotedString(s) => {
                format!("'{}'", s.color(S::bright_yellow()).to_string())
            }
            Token::NationalStringLiteral(s) => {
                format!("N'{}'", s.color(S::bright_yellow()).to_string())
            }
            Token::HexStringLiteral(s) => format!("X'{}'", s.color(S::bright_yellow()).to_string()),
            _ => self.to_string(),
        }
    }
}

impl SQLHighLight for Word {
    fn render<S: Schema + Copy>(&self, _schema: &S) -> String {
        match self.keyword {
            Keyword::NoKeyword => self.to_string().color(S::blue()).to_string(),
            _ => self.value.color(S::green()).to_string(),
        }
    }
}

pub trait Schema {
    fn black() -> Color;
    fn red() -> Color;
    fn green() -> Color;
    fn yellow() -> Color;
    fn blue() -> Color;
    fn purple() -> Color;
    fn cyan() -> Color;
    fn white() -> Color;
    fn bright_black() -> Color;
    fn bright_red() -> Color;
    fn bright_green() -> Color;
    fn bright_yellow() -> Color;
    fn bright_blue() -> Color;
    fn bright_purple() -> Color;
    fn bright_cyan() -> Color;
    fn bright_white() -> Color;
    fn background() -> Color;
    fn foreground() -> Color;
}

#[derive(Debug, Clone, Copy)]
pub struct MonoKaiSchema;

// TODO use macro to reduce code
impl Schema for MonoKaiSchema {
    fn black() -> Color {
        Color::TrueColor { r: 0, g: 0, b: 0 }
    }

    fn red() -> Color {
        Color::TrueColor {
            r: 216,
            g: 30,
            b: 0,
        }
    }

    fn green() -> Color {
        Color::TrueColor {
            r: 94,
            g: 167,
            b: 2,
        }
    }

    fn yellow() -> Color {
        Color::TrueColor {
            r: 207,
            g: 174,
            b: 0,
        }
    }

    fn blue() -> Color {
        Color::TrueColor {
            r: 66,
            g: 122,
            b: 179,
        }
    }

    fn purple() -> Color {
        Color::TrueColor {
            r: 137,
            g: 101,
            b: 142,
        }
    }

    fn cyan() -> Color {
        Color::TrueColor {
            r: 0,
            g: 167,
            b: 170,
        }
    }

    fn white() -> Color {
        Color::TrueColor {
            r: 219,
            g: 222,
            b: 216,
        }
    }

    fn bright_black() -> Color {
        Color::TrueColor {
            r: 104,
            g: 106,
            b: 102,
        }
    }

    fn bright_red() -> Color {
        Color::TrueColor {
            r: 245,
            g: 66,
            b: 53,
        }
    }

    fn bright_green() -> Color {
        Color::TrueColor {
            r: 253,
            g: 235,
            b: 97,
        }
    }

    fn bright_yellow() -> Color {
        Color::TrueColor {
            r: 253,
            g: 235,
            b: 97,
        }
    }

    fn bright_blue() -> Color {
        Color::TrueColor {
            r: 132,
            g: 176,
            b: 216,
        }
    }

    fn bright_purple() -> Color {
        Color::TrueColor {
            r: 188,
            g: 148,
            b: 183,
        }
    }

    fn bright_cyan() -> Color {
        Color::TrueColor {
            r: 55,
            g: 230,
            b: 232,
        }
    }

    fn bright_white() -> Color {
        Color::TrueColor {
            r: 241,
            g: 241,
            b: 240,
        }
    }

    fn background() -> Color {
        Color::TrueColor {
            r: 40,
            g: 42,
            b: 58,
        }
    }

    fn foreground() -> Color {
        Color::TrueColor {
            r: 234,
            g: 242,
            b: 241,
        }
    }
}
