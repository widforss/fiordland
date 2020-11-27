use super::super::error::Error;
use super::Span;
use nom::Err;

pub type ParseResult<'a, O, E = ParseError<'a>> = Result<(Span<'a>, O), Err<E>>;

#[derive(Debug)]
pub struct ParseError<'a> {
    pub input: Span<'a>,
    pub span: Option<Span<'a>>,
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
    pub fn new(input: Span<'a>, span: Option<Span<'a>>, error: ErrorKind) -> Self {
        ParseError { input, span, error }
    }
}

impl<'a> Error for ParseError<'a> {
    fn span(&self) -> Option<Span> {
        self.span
    }

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

impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input,
            span: None,
            error: ErrorKind::Nom(kind),
        }
    }

    fn append(_: Span<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> nom::error::ParseError<&'a str> for ParseError<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input: Span::new(input),
            span: None,
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
