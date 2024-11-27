use std::str::FromStr;

use winnow::{combinator::{self, alt, cut_err, eof, fail, not, opt}, error::{StrContext, StrContextValue}, stream::AsChar, token::{any, take_until, take_while}, PResult, Parser};
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

#[derive(Debug, PartialEq)]
pub enum SclangExpression {
    Literal(i32),
    Boolean(bool),
    Variable(String),
    Let {
        name: String,
        body: Box<Self>,
        tail: Box<Self>,
    },
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
        // Self::parse_let(&mut input.chars().peekable())
        println!("[body] input: {0:?}", input);
        (
            alt((
                cut_err(Self::parse_let),
                cut_err(Self::parse_boolean),
                cut_err(Self::parse_literal),
                cut_err(Self::parse_variable),
                fail
                    .context(StrContext::Label("Invalid expression"))
            )),
            Self::end,
        )
        .map(|(expr, _)| expr)
        .parse_next(input)
    }

    fn end<'s>(input: &mut &'s str) -> PResult<&'s str> {
        alt((multispace1, eof)).parse_next(input)
    }

    fn parse_literal(input: &mut &str) -> PResult<Self> {
        digit1
        .context(StrContext::Label("literal"))
        .context(StrContext::Expected(StrContextValue::Description("number")))
        .parse_next(input)
        .map(|n| SclangExpression::Literal(n.parse().unwrap()))
    }

    fn parse_boolean(input: &mut &str) -> PResult<Self> {
        alt(("true", "false"))
        .context(StrContext::Label("boolean"))
        .parse_next(input)
        .map(|b| SclangExpression::Boolean(b == "true"))
    }

    fn parse_reserved_keyword<'s>(input: &mut &'s str) -> PResult<&'s str> {
        alt((
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
        ))
        .parse_next(input)
    }

    fn parse_variable(input: &mut &str) -> PResult<Self> {
        println!("[var] input: {0:?}", input);
        not(Self::parse_reserved_keyword).parse_next(input)?;
        // must start with letter, after that can be anything
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
            // body: take_until(1, ";")
            //     .and_then(Self::parse.map(Box::new)),
            body: Self::parse.map(Box::new),
            _: (space0, ';', space0)
                .context(StrContext::Label("missing semicolon"))
                .context(StrContext::Expected(StrContextValue::CharLiteral(';'))),
            tail: Self::parse.map(Box::new),
        }}
        // .context(StrContext::Label("let"))
        .parse_next(input)
    }
}