use winnow::ascii::{multispace0, space0};
use winnow::{combinator::alt, PResult, Parser};
use winnow::{combinator::{cut_err, delimited, eof, fail, not, opt, preceded, terminated}, error::{StrContext, StrContextValue}, stream::AsChar, token::{any, take_until, take_while}};
use winnow::combinator::{repeat, seq};

use crate::SclangExpression;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SclangType {
    Num,
    Bool,
    Fun(Box<Self>, Box<Self>),
    Record(Vec<(String, Self)>),
}

impl SclangType {
    pub(crate) fn parse(input: &mut &str) -> PResult<Self> {
        alt((
            Self::parse_complex,
            Self::parse_atom,
        ))
        .parse_next(input)
    }

    fn parse_atom(input: &mut &str) -> PResult<Self> {
        alt((
            "num".value(SclangType::Num),
            "bool".value(SclangType::Bool),
        ))
        .parse_next(input)
    }

    fn parse_complex(input: &mut &str) -> PResult<Self> {
        alt((
            Self::parse_record,
            Self::parse_fun,
        ))
        .parse_next(input)
    }

    fn parse_fun(input: &mut &str) -> PResult<Self> {
        seq!(Self::parse_atom, _: "->", Self::parse_atom)
        .map(|(param_type, return_type)| SclangType::Fun(Box::new(param_type), Box::new(return_type)))
        .parse_next(input)
    }

    fn parse_record(input: &mut &str) -> PResult<Self> {
        delimited(
            ("{", multispace0),
            repeat(1.., Self::parse_record_field).fold(
                Vec::new,
            |mut acc, (name, ty)| {
                acc.push((name, ty));
                acc
            }),
            ("}", multispace0),
        )
        .map(SclangType::Record)
        .parse_next(input)
    }

    fn parse_record_field(input: &mut &str) -> PResult<(String, Self)> {
        seq!(
            SclangExpression::parse_variable,
            _: (space0, ":", space0),
            Self::parse,
            _: opt((space0, ",", space0))
        )
        .parse_next(input)
    }
}