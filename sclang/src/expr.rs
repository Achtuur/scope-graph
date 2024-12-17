use std::{io::Read, path::Path, str::FromStr};

use winnow::ascii::*;
use winnow::combinator::seq;
use winnow::{
    combinator::{alt, cut_err, delimited, eof, fail, not, opt, preceded, repeat},
    error::{ParserError, StrContext, StrContextValue},
    stream::{AsChar, Stream},
    token::take_while,
    PResult, Parser,
};

use crate::SclangType;

pub(crate) const RESERVED_KEYWORDS: [&str; 10] = [
    "let", "if", "else", "while", "return", "break", "continue", "fun", "true", "false",
];

#[derive(Debug, PartialEq, Clone)]
pub enum SclangExpression {
    Literal(i32),
    Boolean(bool),
    Var(String),
    // arg1, arg2
    Add(Box<Self>, Box<Self>),
    // parameter, parameter type, body
    Func {
        param: String,
        p_type: SclangType,
        body: Box<Self>,
    },
    Call {
        fun: Box<Self>,
        arg: Box<Self>,
    },
    Let {
        name: String,
        body: Box<Self>,
        tail: Box<Self>,
    },
    Record(Vec<(String, Self)>),

    RecordAccess {
        record: Box<Self>,
        field: String,
    },

    Extension {
        /// Added/overwritten fields
        extension: Box<Self>,
        /// Parent record
        parent: Box<Self>,
    },

    With {
        record: Box<Self>,
        body: Box<Self>,
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
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut contents = String::new();
        std::fs::File::open(path.as_ref())
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        Self::from_str(contents.as_str())
    }

    pub(crate) fn parse(input: &mut &str) -> PResult<Self> {
        delimited(multispace0, Self::parse_expr, eof).parse_next(input)
    }
}

// Expression parsing
impl SclangExpression {
    fn parse_expr(input: &mut &str) -> PResult<Self> {
        preceded(multispace0, alt((Self::parse_expr_ext, Self::parse_term))).parse_next(input)
    }

    fn parse_term(input: &mut &str) -> PResult<Self> {
        preceded(
            multispace0,
            alt((
                Self::parse_let,
                Self::parse_fun,
                Self::parse_record_decl,
                Self::parse_with,
                Self::parse_atom,
                fail.context(StrContext::Label("term")),
            )),
        )
        .parse_next(input)
    }

    fn parse_expr_ext(input: &mut &str) -> PResult<Self> {
        preceded(
            multispace0,
            alt((
                Self::parse_add,
                Self::parse_record_access,
                Self::parse_call,
                Self::parse_extension,
            )),
        )
        .parse_next(input)
    }

    fn parse_add(input: &mut &str) -> PResult<Self> {
        seq! {Self::Add (
            Self::parse_term.map(Box::new),
            _: (multispace0, "+", multispace0),
            Self::parse_expr.map(Box::new),
        )}
        .parse_next(input)
    }

    fn parse_call(input: &mut &str) -> PResult<Self> {
        seq! {Self::Call {
            fun: Self::parse_term.map(Box::new),
            _: (space0, '(', space0),
            arg: Self::parse_expr.map(Box::new),
            _: (space0, ')'),
        }}
        .parse_next(input)
    }

    fn parse_record_access(input: &mut &str) -> PResult<Self> {
        seq! {Self::RecordAccess{
            record: Self::parse_term.map(Box::new),
            _: '.',
            field: Self::parse_variable,
        }}
        .context(StrContext::Label("record access"))
        .parse_next(input)
    }

    fn parse_extension(input: &mut &str) -> PResult<Self> {
        seq! {Self::Extension {
            extension: Self::parse_record_decl.map(Box::new),
            _: (space1, "extends", space1),
            parent: Self::parse_expr.map(Box::new),
        }}
        .context(StrContext::Label("extension"))
        .parse_next(input)
    }
}

// terms
impl SclangExpression {
    fn parse_let(input: &mut &str) -> PResult<Self> {
        seq! {Self::Let {
            _: ("let", space1),
            name: Self::parse_variable.map(|e| e),
            _: (space0, '=', space0),
            body: cut_err(
                Self::parse_expr.map(Box::new)
                    .context(StrContext::Label("body expression"))
                    .context(StrContext::Expected(StrContextValue::Description("expression")))
            ),
            // body: Self::parse.map(Box::new),
            _: cut_err(
                (space0, ';', space0)
                .context(StrContext::Label("missing semicolon"))
                .context(StrContext::Expected(StrContextValue::CharLiteral(';')))
            ),
            tail: Self::parse_expr.map(Box::new)
                .context(StrContext::Label("tail expression"))
                .context(StrContext::Expected(StrContextValue::Description("expression"))),
        }}
        .parse_next(input)
    }

    fn parse_fun(input: &mut &str) -> PResult<Self> {
        seq! {Self::Func {
            _: ("fun", space0, '(', space0),
            param: Self::parse_variable,
            _: (space0, ':', space0),
            p_type: SclangType::parse,
            _: (space0, ')'),
            body: in_curly_braces(Self::parse_expr.map(Box::new)),
        }}
        .parse_next(input)
    }

    fn parse_with(input: &mut &str) -> PResult<Self> {
        seq! {Self::With {
            _: ("with", space1),
            record: Self::parse_expr.map(Box::new),
            _: (space1, "do", space1),
            body: in_curly_braces(Self::parse_expr.map(Box::new)),
        }}
        .parse_next(input)
    }
}

// Atom parsing
impl SclangExpression {
    fn parse_atom(input: &mut &str) -> PResult<Self> {
        preceded(
            multispace0,
            alt((
                Self::parse_variable.map(SclangExpression::Var),
                Self::parse_literal,
                Self::parse_boolean,
                fail.context(StrContext::Label("atom")),
            )),
        )
        .parse_next(input)
    }

    fn parse_literal(input: &mut &str) -> PResult<Self> {
        // delimited(
        // not(alpha1),
        digit1
            // not(alpha1),
            // )
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
        alt(RESERVED_KEYWORDS).parse_next(input)
    }

    pub(crate) fn parse_variable(input: &mut &str) -> PResult<String> {
        not(Self::parse_reserved_keyword).parse_next(input)?;
        (
            alpha1, // start part, must be 1 or more letters
            // after that numbers are allowed as well
            take_while(0.., |c| AsChar::is_alphanum(c) || c == '_'),
        )
            .context(StrContext::Label("variable"))
            .parse_next(input)
            // concat the two parts
            .map(|(s, e)| format!("{}{}", s, e))
    }

    fn parse_record_decl(input: &mut &str) -> PResult<Self> {
        in_curly_braces(repeat(1.., Self::parse_record_field).fold(
            Vec::new,
            |mut acc, (name, ty)| {
                acc.push((name, ty));
                acc
            },
        ))
        .context(StrContext::Label("record"))
        .map(SclangExpression::Record)
        .parse_next(input)
    }

    fn parse_record_field(input: &mut &str) -> PResult<(String, Self)> {
        seq!(
            SclangExpression::parse_variable,
            _: (space0, "=", space0),
            Self::parse_expr,
            _: opt((space0, ",", space0))
        )
        .context(StrContext::Label("record field"))
        .parse_next(input)
    }
}

pub fn in_curly_braces<Input, Output, Error, ParseNext>(
    parser: ParseNext,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + winnow::stream::StreamIsPartial + winnow::stream::Compare<char>,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
    <Input as winnow::stream::Stream>::Token: winnow::stream::AsChar,
    <Input as winnow::stream::Stream>::Token: std::clone::Clone,
{
    delimited((multispace0, '{', multispace0), parser, (multispace0, '}'))
}
