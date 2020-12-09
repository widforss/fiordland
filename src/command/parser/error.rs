use crate::error::Error;
use nom::Err;

pub type ParseResult<'a, O, E = ParseError<'a>> = Result<(&'a str, O), Err<E>>;

#[derive(Debug)]
pub struct ParseError<'a> {
    pub input: &'a str,
    pub error: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    NotRecognised,
    ParseProjection,
    ParseCoordinate,
    ParseNumber,
    ParseDate,
    ParseTime,
    DateOutOfRange,
    Nom(nom::error::ErrorKind),
}

impl<'a> ParseError<'a> {
    pub fn new(input: &'a str, error: ErrorKind) -> Self {
        ParseError { input, error }
    }
}

impl<'a> Error for ParseError<'a> {
    fn description(&self) -> String {
        use ErrorKind::*;

        let string: &'a str = match self.error {
            NotRecognised => "Failed to parse input",
            ParseProjection => "Failed to parse projection",
            ParseCoordinate => "Failed to parse coordinate",
            ParseNumber => "Failed to parse number",
            ParseDate => "Failed to parse date",
            ParseTime => "Failed to parse time",
            DateOutOfRange => "Date out of range",
            Nom(_) => panic!(),
        };
        String::from(string)
    }
}

impl<'a> nom::error::ParseError<&'a str> for ParseError<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input,
            error: ErrorKind::Nom(kind),
        }
    }

    fn append(_: &'a str, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> From<nom::Err<ParseError<'a>>> for ParseError<'a> {
    fn from(item: nom::Err<ParseError<'a>>) -> Self {
        match item {
            Err::Failure(err) => err,
            Err::Error(err) => err,
            Err::Incomplete(_) => panic!(),
        }
    }
}
