use std::str::FromStr;

use winnow::{combinator::{self, alt, cut_err, delimited, eof, fail, not, opt, terminated}, error::{StrContext, StrContextValue}, stream::AsChar, token::{any, take_until, take_while}, PResult, Parser};
use winnow::ascii::*;
use winnow::combinator::seq;

pub(crate) const RESERVED_KEYWORDS: [&str; 10] = [
    "let",
    "if",
    "else",
    "while",
    "return",
    "break",
    "continue",
    "fn",
    "true",
    "false",
];

#[derive(Debug, PartialEq, Clone)]
pub enum SclangExpression {
    Literal(i32),
    Boolean(bool),
    Variable(String),
    Let {
        name: String,
        body: Box<Self>,
        tail: Box<Self>,
    },
    Add(Box<Self>, Box<Self>),
}

/// Uses Debug implementation
impl std::fmt::Display for SclangExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for SclangExpression {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SclangExpression::parse
        .parse(s)
        .map_err(|e| format!("{}", e))
    }
}

// utility
impl SclangExpression {
    pub fn as_var(&self) -> Option<&str> {
        match self {
            SclangExpression::Variable(v) => Some(v),
            _ => None,
        }
    }
}


impl SclangExpression {
    pub fn parse(input: &mut &str) -> PResult<Self> {
        delimited(multispace0, Self::parse_expr, multispace0).parse_next(input)
    }

    pub fn parse_expr(input: &mut &str) -> PResult<Self> {
        println!("[body] input: {0:?}", input);
        alt((
            Self::parse_let,
            // Self::parse_add,
            Self::parse_variable,
            Self::parse_literal,
            Self::parse_boolean,
            fail
                .context(StrContext::Label("Invalid expression"))
        ))
        .map(|expr| expr)
        .parse_next(input)
    }

    fn parse_literal(input: &mut &str) -> PResult<Self> {
        delimited(
            not(alpha1),
            digit1,
            not(alpha1),
        )
        .try_map(|n: &str| n.parse::<i32>())
        .context(StrContext::Label("literal"))
        .context(StrContext::Expected(StrContextValue::Description("number")))
        .parse_next(input)
        .map(SclangExpression::Literal)
    }

    fn parse_boolean(input: &mut &str) -> PResult<Self> {
        let parse_true = "true".value(SclangExpression::Boolean(true));
        let parse_false = "false".value(SclangExpression::Boolean(false));
        alt((parse_true, parse_false))
        .context(StrContext::Label("boolean"))
        .parse_next(input)
    }

    fn parse_reserved_keyword<'s>(input: &mut &'s str) -> PResult<&'s str> {
        alt(RESERVED_KEYWORDS)
        .parse_next(input)
    }

    fn parse_variable(input: &mut &str) -> PResult<Self> {
        println!("[var] input: {0:?}", input);
        not(Self::parse_reserved_keyword).parse_next(input)?;
        (
            alpha1, // start part, must be 1 or more letters
            // after that numbers are allowed as well
            take_while(0.., |c| AsChar::is_alphanum(c) || c == '_'),
        )
        .context(StrContext::Label("variable"))
        .parse_next(input)
        // concat the two parts
        .map(|(s, e)| SclangExpression::Variable(format!("{}{}", s, e)))
    }

    fn parse_let(input: &mut &str) -> PResult<Self> {
        seq!{Self::Let {
            _: ("let", space1),
            name: Self::parse_variable.map(|e| e.as_var().unwrap().to_string()),
            _: (space0, "=", space0),
            body: cut_err(
                take_until(1.., ";"))
                .context(StrContext::Label("missing semicolon"))
                .context(StrContext::Expected(StrContextValue::CharLiteral(';'))
            )
            .and_then(
                cut_err(
                    Self::parse.map(Box::new)
                    .context(StrContext::Label("body expression"))
                    .context(StrContext::Expected(StrContextValue::Description("expression")))
                )
            ),
            // body: Self::parse.map(Box::new),
            _: cut_err((space0, ';', space0)),
            tail: Self::parse.map(Box::new)
                .context(StrContext::Label("tail expression"))
                .context(StrContext::Expected(StrContextValue::Description("expression"))),
        }}
        // .context(StrContext::Label("let"))
        .parse_next(input)
    }

    fn parse_add(input: &mut &str) -> PResult<Self> {
        (
            Self::parse_expr,
            space0, "+", space0,
            Self::parse_expr,
        )
        .parse_next(input)
        .map(|(lhs, _, _, _, rhs)| SclangExpression::Add(Box::new(lhs), Box::new(rhs)))
    }
}