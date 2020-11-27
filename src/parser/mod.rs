mod error;

use super::Span;
use chrono::prelude::*;
use error::{ErrorKind, ParseError, ParseResult};
use nom::{
    branch, bytes::complete as bytes, character::complete as character, multi, sequence, Err,
};
use std::str::FromStr;

#[derive(Debug)]
pub enum Command<'a> {
    Create(Vec<Point<'a>>, Span<'a>),
    Edit(Vec<Point<'a>>, Span<'a>),
    Checkin(Point<'a>, Span<'a>),
    Complete(Span<'a>),
}

#[derive(Debug)]
pub struct Point<'a> {
    position: Position<'a>,
    action: Option<Action<'a>>,
    message: Option<Message<'a>>,
    time: Option<Time<'a>>,
}

#[derive(Debug)]
pub struct Position<'a> {
    projection: Projection<'a>,
    eastings: Coordinate<'a>,
    northings: Coordinate<'a>,
    span: Span<'a>,
}

#[derive(Debug)]
pub struct Coordinate<'a>(u32, Span<'a>);

#[derive(Debug)]
pub enum Projection<'a> {
    UTM32(Span<'a>),
    UTM33(Span<'a>),
    UTM34(Span<'a>),
    UTM35(Span<'a>),
}

#[derive(Debug)]
pub enum Action<'a> {
    Food(Span<'a>),
    Tent(Span<'a>),
    Hut(Span<'a>),
}

#[derive(Debug)]
pub struct Message<'a>(String, Span<'a>);

#[derive(Debug)]
pub enum Time<'a> {
    Date(NaiveDate, Span<'a>),
    Time(NaiveTime, Span<'a>),
    DateTime(NaiveDateTime, Span<'a>),
}

impl<'a> Command<'a> {
    pub fn parse(input: &'a str) -> Result<Self, ParseError> {
        match parse_command(Span::new(input)) {
            Ok((Span { fragment: "", .. }, command)) => Ok(command),
            Ok((input, _)) => Err(ParseError::new(
                input,
                Some(input),
                ErrorKind::NotRecognised,
            )),
            Err(Err::Failure(ParseError { input, span, error })) => {
                Err(ParseError::new(input, span, error))
            }
            Err(Err::Error(ParseError { input, span, .. })) => {
                Err(ParseError::new(input, span, ErrorKind::NotRecognised))
            }
            Err(_) => panic!(),
        }
    }
}

fn parse_command(input: Span) -> ParseResult<Command> {
    let parser = sequence::terminated(
        branch::alt((parse_create, parse_edit, parse_checkin, parse_complete)),
        character::multispace0,
    );
    let (input, command) = parser(input)?;

    Ok((input, command))
}

fn parse_create(input: Span) -> ParseResult<Command> {
    let (mut span, _) = character::multispace0(input)?;

    let tag = "create";
    let parser = sequence::tuple((bytes::tag_no_case(tag), multi::many1(parse_point)));
    let (input, (_, points)) = parser(input)?;
    span.fragment = &span.fragment[..tag.len()];

    Ok((input, Command::Create(points, span)))
}

fn parse_edit(input: Span) -> ParseResult<Command> {
    let (mut span, _) = character::multispace0(input)?;

    let tag = "edit";
    let parser = sequence::tuple((bytes::tag_no_case(tag), multi::many1(parse_point)));
    let (input, (_, points)) = parser(input)?;
    span.fragment = &span.fragment[..tag.len()];

    Ok((input, Command::Edit(points, span)))
}

fn parse_checkin(input: Span) -> ParseResult<Command> {
    let (mut span, _) = character::multispace0(input)?;

    let tag = "checkin";
    let parser = sequence::tuple((bytes::tag_no_case(tag), parse_point));
    let (input, (_, point)) = parser(input)?;
    span.fragment = &span.fragment[..tag.len()];

    Ok((input, Command::Checkin(point, span)))
}

fn parse_complete(input: Span) -> ParseResult<Command> {
    let (mut span, _) = character::multispace0(input)?;

    let tag = "complete";
    let parser = bytes::tag_no_case(tag);
    let (input, _) = parser(input)?;
    span.fragment = &span.fragment[..tag.len()];

    Ok((input, Command::Complete(span)))
}

fn parse_point<'a>(input: Span<'a>) -> ParseResult<Point> {
    let (mut span, _) = character::multispace0(input)?;

    let time_parser = branch::alt((parse_datetime, parse_date, parse_time));

    let some_action = |input: Span<'a>| {
        let (input, result) = parse_action(input)?;
        Ok((input, Some(result)))
    };
    let some_message = |input: Span<'a>| {
        let (input, result) = parse_message(input)?;
        Ok((input, Some(result)))
    };
    let some_time = |input: Span<'a>| {
        let (input, result) = time_parser(input)?;
        Ok((input, Some(result)))
    };
    let none_action = |input: Span<'a>| Ok((input, None));
    let none_message = |input: Span<'a>| Ok((input, None));
    let none_time = |input: Span<'a>| Ok((input, None));

    let with_all = branch::permutation((parse_position, some_action, some_message, some_time));
    let with_action_message =
        branch::permutation((parse_position, some_action, some_message, none_time));
    let with_action_time =
        branch::permutation((parse_position, some_action, none_message, some_time));
    let with_message_time =
        branch::permutation((parse_position, none_action, some_message, some_time));
    let with_action = branch::permutation((parse_position, some_action, none_message, none_time));
    let with_message = branch::permutation((parse_position, none_action, some_message, none_time));
    let with_time = branch::permutation((parse_position, none_action, none_message, some_time));
    let with_none = branch::permutation((parse_position, none_action, none_message, none_time));

    let parser = sequence::tuple((
        branch::alt((
            with_all,
            with_action_message,
            with_action_time,
            with_message_time,
            with_action,
            with_message,
            with_time,
            with_none,
        )),
        bytes::tag(""),
    ));
    let (input, ((position, action, message, time), end)) = parser(input)?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];

    Ok((
        input,
        Point {
            position,
            action,
            message,
            time,
        },
    ))
}

fn parse_position(input: Span) -> ParseResult<Position> {
    let (mut span, _) = character::multispace0(input)?;

    let parse_northings = make_parse_coordinate('N', 7);
    let parse_eastings = make_parse_coordinate('E', 6);
    let parser = sequence::tuple((
        branch::permutation((parse_projection, parse_northings, parse_eastings)),
        bytes::tag(""),
    ));
    let (input, ((projection, northings, eastings), end)) = parser(input)?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];

    let position = Position {
        projection,
        northings,
        eastings,
        span,
    };
    Ok((input, position))
}

fn parse_projection(input: Span) -> ParseResult<Projection> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;
    let mut span = input;

    let parser = sequence::tuple((
        branch::alt((bytes::tag_no_case("UTM"), bytes::tag(""))),
        character::digit1,
        bytes::tag(""),
    ));
    let parsed = transform_parsed(parser(input), orig_input, span);
    let (input, (_, digits, end)) = parsed?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];

    let projection = match digits.fragment {
        "32" => Projection::UTM32(span),
        "33" => Projection::UTM33(span),
        "34" => Projection::UTM34(span),
        "35" => Projection::UTM35(span),
        str if str.len() == 2 => {
            return Err(Err::Failure(ParseError::new(
                input,
                Some(span),
                ErrorKind::ParseProjection,
            )))
        }
        _ => {
            return Err(Err::Error(ParseError::new(
                orig_input,
                Some(span),
                ErrorKind::ParseProjection,
            )))
        }
    };
    Ok((input, projection))
}

fn make_parse_coordinate<'a>(
    c_char: char,
    c_len: usize,
) -> impl Fn(Span<'a>) -> ParseResult<Coordinate<'a>> {
    let c_str = c_char.to_string();
    move |input: Span<'a>| {
        let orig_input = input;
        let (input, _) = character::multispace1(input)?;
        let mut span = input;

        let parser = sequence::tuple((
            branch::alt((bytes::tag_no_case(&c_str[..]), bytes::tag(""))),
            parse_int::<u32>,
            branch::alt((bytes::tag(","), bytes::tag("."), bytes::tag(""))),
            character::digit0,
            bytes::tag(""),
        ));
        let parsed = transform_parsed(parser(input), orig_input, span);
        let (input, (prefix, (number, n_str), point, _, end)) = parsed?;
        span.fragment = &span.fragment[..(end.offset - span.offset)];
        let n_str = n_str.fragment;

        if n_str.len() == c_len {
            Ok((input, Coordinate(number, span)))
        } else if prefix.fragment.len() == 1
            && point.fragment.len() == 0
            && n_str.len() <= c_len
            && n_str.len() > 2
        {
            let missing_pow = c_len - n_str.len();
            let number = number * 10u32.pow(missing_pow as u32);
            Ok((input, Coordinate(number, span)))
        } else if n_str.len() <= c_len && point.fragment.len() == 1 {
            Err(Err::Failure(ParseError::new(
                orig_input,
                Some(span),
                ErrorKind::ParseCoordinate,
            )))
        } else {
            Err(Err::Error(ParseError::new(
                orig_input,
                Some(span),
                ErrorKind::ParseCoordinate,
            )))
        }
    }
}

fn parse_action(input: Span) -> ParseResult<Action> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;
    let mut span = input;

    let parser = sequence::tuple((
        branch::alt((
            bytes::tag_no_case("tent"),
            bytes::tag_no_case("hut"),
            bytes::tag_no_case("food"),
        )),
        bytes::tag(""),
    ));
    let parsed = transform_parsed(parser(input), orig_input, span);
    let (input, (action, end)) = parsed?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];

    let action = match action.fragment {
        "tent" => Action::Tent(span),
        "hut" => Action::Hut(span),
        "food" => Action::Food(span),
        _ => panic!(),
    };
    Ok((input, action))
}

fn parse_message(input: Span) -> ParseResult<Message> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;
    let mut span = input;

    let parser = sequence::tuple((
        sequence::delimited(
            bytes::tag("\""),
            bytes::escaped(character::none_of("\"\\"), '\\', character::one_of("\"\\")),
            bytes::tag("\""),
        ),
        bytes::tag(""),
    ));
    let (input, (message, end)) = transform_parsed(parser(input), orig_input, span)?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];
    let message = message.fragment.to_string().replace("\\\"", "\"");

    Ok((input, Message(message, span)))
}

fn parse_datetime<'a>(input: Span<'a>) -> ParseResult<Time> {
    let (mut span, _) = character::multispace0(input)?;

    let parser = sequence::tuple((parse_date, parse_time, bytes::tag("")));
    let (input, (date, time, end)) = parser(input)?;
    if let (Time::Date(date, _), Time::Time(time, _)) = (date, time) {
        span.fragment = &span.fragment[..(end.offset - span.offset)];

        let ndt = NaiveDateTime::new(date, time);
        Ok((input, Time::DateTime(ndt, span)))
    } else {
        panic!()
    }
}

fn parse_date(input: Span) -> ParseResult<Time> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;
    let mut span = input;

    let parser = sequence::tuple((
        parse_int::<i32>,
        bytes::tag("-"),
        parse_int::<u32>,
        bytes::tag("-"),
        parse_int::<u32>,
        bytes::tag(""),
    ));
    let parsed = transform_parsed(parser(input), orig_input, span);
    let (input, ((year, _), _, (month, _), _, (day, _), end)) = parsed?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];

    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) if 1970 <= year && year < 2030 => Ok((input, Time::Date(date, span))),
        Some(_) => Err(Err::Failure(ParseError::new(
            orig_input,
            Some(span),
            ErrorKind::DateOutOfRange,
        ))),
        None => Err(Err::Failure(ParseError::new(
            orig_input,
            Some(span),
            ErrorKind::ParseDate,
        ))),
    }
}

fn parse_time<'a>(input: Span<'a>) -> ParseResult<Time> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;
    let mut span = input;

    let parser = sequence::tuple((
        parse_int::<u32>,
        bytes::tag(":"),
        parse_int::<u32>,
        branch::alt((
            sequence::preceded(bytes::tag(":"), parse_int::<u32>),
            |input: Span<'a>| {
                let mut span = input;
                span.fragment = &span.fragment[0..0];
                Ok((input, (0u32, span)))
            },
        )),
        bytes::tag(""),
    ));
    let parsed = transform_parsed(parser(input), orig_input, span);
    let (input, ((hour, _), _, (minute, _), (second, _), end)) = parsed?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];

    match NaiveTime::from_hms_opt(hour, minute, second) {
        Some(time) => Ok((input, Time::Time(time, span))),
        None => Err(Err::Failure(ParseError::new(
            orig_input,
            Some(span),
            ErrorKind::ParseTime,
        ))),
    }
}

fn parse_int<T: FromStr>(input: Span) -> ParseResult<(T, Span)> {
    let orig_input = input;
    let mut span = input;

    let parser = sequence::tuple((character::digit1, bytes::tag("")));
    let (input, (digits, end)) = parser(input)?;
    span.fragment = &span.fragment[..(end.offset - span.offset)];
    let digits = &span.fragment[(digits.offset - span.offset)..];

    let number = match digits.parse::<T>() {
        Ok(number) => Ok(number),
        Err(_) => Err(Err::Failure(ParseError::new(
            orig_input,
            Some(span),
            ErrorKind::ParseNumber,
        ))),
    }?;
    Ok((input, (number, span)))
}

fn transform_parsed<'a, T>(
    output: ParseResult<'a, T>,
    orig_input: Span<'a>,
    span: Span<'a>,
) -> ParseResult<'a, T> {
    match output {
        Ok(res) => Ok(res),
        Err(Err::Error(ParseError { error, .. })) => {
            Err(Err::Error(ParseError::new(orig_input, Some(span), error)))
        }
        Err(Err::Failure(ParseError { error, .. })) => {
            Err(Err::Failure(ParseError::new(orig_input, Some(span), error)))
        }
        _ => panic!(),
    }
}
