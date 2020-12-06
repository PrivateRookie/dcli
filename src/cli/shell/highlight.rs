use colored::*;
use sqlparser::ast::{
    Cte, Expr, Ident, Join, Query, Select, SelectItem, SetExpr, SetOperator, Statement, TableAlias,
    TableFactor, TableWithJoins, Values,
};

pub trait SQLHighLight {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String;
}

impl SQLHighLight for Query {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        let mut rendered = String::new();
        if !self.ctes.is_empty() {
            rendered.push_str(&format!("WITH {}", comma_separated(&self.ctes, schema)))
        }
        rendered.push_str(&format!("{}", self.body.render(schema)));
        rendered
    }
}

impl SQLHighLight for SetExpr {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        match self {
            SetExpr::Select(s) => s.render(schema),
            SetExpr::Query(q) => q.render(schema),
            SetExpr::SetOperation {
                op,
                all,
                left,
                right,
            } => {
                let all_str = if *all { " ALL" } else { "" };
                format!(
                    "{} {}{} {}",
                    left.render(schema),
                    op.render(schema),
                    all_str,
                    right.render(schema)
                )
            }
            SetExpr::Values(v) => v.render(schema),
        }
    }
}

impl SQLHighLight for SetOperator {
    fn render<S: Schema + Copy>(&self, _schema: &S) -> String {
        format!("{}", self.to_string().color(S::green()))
    }
}

impl SQLHighLight for Values {
    fn render<S: Schema + Copy>(&self, _schema: &S) -> String {
        self.to_string()
    }
}

impl SQLHighLight for Select {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        let mut rendered = "SELECT".color(S::red()).to_string();
        if self.distinct {
            rendered.push_str(&" DISTINCT".color(S::cyan()));
        }
        if let Some(ref top) = self.top {
            rendered.push_str(&format!(" {}", top));
        }
        rendered.push_str(&format!(" {}", comma_separated(&self.projection, schema)));
        if !self.from.is_empty() {
            rendered.push_str(&format!(
                "{} {}",
                " FROM".color(S::red()),
                comma_separated(&self.from, schema)
            ))
        }
        if let Some(ref selection) = self.selection {
            rendered.push_str(&format!(
                " {} {}",
                "WHERE".color(S::red()),
                selection.render(schema)
            ))
        }
        if !self.group_by.is_empty() {
            rendered.push_str(&format!(
                " {} {}",
                "GROUP BY".color(S::red()),
                comma_separated(&self.group_by, schema)
            ))
        }
        if let Some(ref having) = self.having {
            rendered.push_str(&format!(" {} {}", "HAVING".color(S::red()), having))
        }
        rendered
    }
}

impl SQLHighLight for TableWithJoins {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        format!(
            "{}{}",
            self.relation.render(schema),
            self.joins
                .iter()
                .map(|j| j.render(schema))
                .collect::<Vec<String>>()
                .join("")
        )
    }
}

impl SQLHighLight for TableFactor {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        match self {
            TableFactor::Table {
                name,
                alias,
                args,
                with_hints,
            } => {
                let mut rendered = format!("{}", name);
                if !args.is_empty() {
                    rendered.push_str(&format!("({})", comma_separated(args, schema)))
                }
                if let Some(alias) = alias {
                    rendered.push_str(&format!(" AS {}", alias.render(schema)));
                }
                if !with_hints.is_empty() {
                    rendered.push_str(&format!(" WITH ({})", comma_separated(with_hints, schema)))
                }
                rendered
            }
            TableFactor::Derived {
                lateral,
                subquery,
                alias,
            } => {
                let mut rendered = String::new();
                if *lateral {
                    rendered.push_str(&"LATERAL ".color(S::bright_purple()))
                }
                rendered.push_str(&format!("({})", subquery.render(schema)));
                if let Some(alias) = alias {
                    rendered.push_str(&format!(" AS {}", alias.render(schema)))
                }
                rendered
            }
            TableFactor::NestedJoin(table_ref) => {
                format!("({})", table_ref.render(schema))
            }
        }
    }
}

impl SQLHighLight for Join {
    fn render<S: Schema + Copy>(&self, _schema: &S) -> String {
        //TODO add high light
        self.to_string()
    }
}

impl SQLHighLight for SelectItem {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        match &self {
            SelectItem::UnnamedExpr(expr) => expr.render(schema),
            SelectItem::ExprWithAlias { expr, alias } => format!(
                "{} {} {}",
                expr.render(schema),
                "AS".color(S::red()),
                alias.render(schema)
            ),
            SelectItem::QualifiedWildcard(prefix) => {
                // TODO add highlight
                format!("{}.*", prefix.to_string())
            }
            SelectItem::Wildcard => "*".to_string(),
        }
    }
}

impl SQLHighLight for Expr {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        match self {
            Expr::Identifier(s) => s.render(schema),
            Expr::Wildcard => "*".to_string(),
            Expr::QualifiedWildcard(q) => comma_separated(&q, schema),
            Expr::CompoundIdentifier(s) => comma_separated(&s, schema),
            Expr::IsNull(ast) => format!("{} IS NULL", ast.render(schema)),
            Expr::IsNotNull(ast) => format!("{} IS NOT NULL", ast.render(schema)),
            Expr::InList {
                expr,
                list,
                negated,
            } => format!(
                "{} {}IN ({})",
                expr.render(schema),
                if *negated {
                    "NOT ".color(S::red()).to_string()
                } else {
                    "".to_string()
                },
                comma_separated(&list, schema)
            ),
            Expr::InSubquery {
                expr,
                subquery,
                negated,
            } => format!(
                "{} {}IN ({})",
                expr.render(schema),
                if *negated {
                    "NOT ".color(S::red()).to_string()
                } else {
                    "".to_string()
                },
                subquery.render(schema)
            ),
            // TODO add highlight
            _ => self.to_string(),
        }
    }
}

impl SQLHighLight for Cte {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        format!(
            "{} AS ({})",
            self.alias.render(schema),
            self.query.render(schema)
        )
    }
}

impl SQLHighLight for TableAlias {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        if self.columns.is_empty() {
            format!("{}", self.name.render(schema))
        } else {
            let columns: Vec<String> = self.columns.iter().map(|col| col.render(schema)).collect();
            format!("{} ({})", self.name.render(schema), columns.join(", "))
        }
    }
}

impl SQLHighLight for Ident {
    fn render<S: Schema>(&self, _: &S) -> String {
        format!("{}", self.to_string().color(S::red()))
    }
}

impl SQLHighLight for Statement {
    fn render<S: Schema + Copy>(&self, schema: &S) -> String {
        match self {
            Statement::Query(q) => q.render(schema),
            Statement::ShowVariable { variable } => {
                format!("{} {}", "SHOW".color(S::cyan()), variable)
            }
            _ => self.to_string(),
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

fn comma_separated<H: SQLHighLight, S: Schema + Copy>(values: &Vec<H>, schema: &S) -> String {
    let rendered: Vec<String> = values.iter().map(|val| val.render(schema)).collect();
    format!("{}", rendered.join(", "))
}
