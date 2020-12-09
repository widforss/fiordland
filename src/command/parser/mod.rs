mod error;

use super::*;
use chrono::prelude::{NaiveDate, NaiveTime};
use error::{ErrorKind, ParseError, ParseResult};
use nom::{
    branch, bytes::complete as bytes, character::complete as character, multi, sequence, Err,
};
use std::str::FromStr;

impl<'a> Command {
    pub fn parse(input: &'a str) -> Result<Self, ParseError> {
        match parse_command(input) {
            Ok(("", command)) => Ok(command),
            Ok((input, _)) => Err(ParseError::new(input, ErrorKind::NotRecognised)),
            Err(Err::Failure(ParseError { input, error })) => Err(ParseError::new(input, error)),
            Err(Err::Error(ParseError { input, .. })) => {
                Err(ParseError::new(input, ErrorKind::NotRecognised))
            }
            Err(_) => panic!(),
        }
    }
}

fn parse_command(input: &str) -> ParseResult<Command> {
    let parser = sequence::terminated(
        branch::alt((parse_create, parse_edit, parse_checkin, parse_complete)),
        character::multispace0,
    );
    let (input, command) = parser(input)?;

    Ok((input, command))
}

fn parse_create(input: &str) -> ParseResult<Command> {
    let tag = "create";
    let parser = sequence::tuple((bytes::tag_no_case(tag), multi::many0(parse_point)));
    let (input, (_, points)) = parser(input)?;

    Ok((input, Command::Create(points)))
}

fn parse_edit(input: &str) -> ParseResult<Command> {
    let tag = "edit";
    let parser = sequence::tuple((bytes::tag_no_case(tag), multi::many1(parse_point)));
    let (input, (_, points)) = parser(input)?;

    Ok((input, Command::Edit(points)))
}

fn parse_checkin(input: &str) -> ParseResult<Command> {
    let tag = "checkin";
    let parser = sequence::tuple((bytes::tag_no_case(tag), parse_point));
    let (input, (_, point)) = parser(input)?;

    Ok((input, Command::Checkin(point)))
}

fn parse_complete(input: &str) -> ParseResult<Command> {
    let tag = "complete";
    let parser = bytes::tag_no_case(tag);
    let (input, _) = parser(input)?;

    Ok((input, Command::Complete))
}

fn parse_point<'a>(input: &'a str) -> ParseResult<'a, Point> {
    let time_parser = branch::alt((parse_datetime, parse_date, parse_time));

    let some_action = |input: &'a str| {
        let (input, result) = parse_action(input)?;
        Ok((input, Some(result)))
    };
    let some_message = |input: &'a str| {
        let (input, result) = parse_message(input)?;
        Ok((input, Some(result)))
    };
    let some_time = |input: &'a str| {
        let (input, result) = time_parser(input)?;
        Ok((input, Some(result)))
    };
    let none_action = |input: &'a str| Ok((input, None));
    let none_message = |input: &'a str| Ok((input, None));
    let none_time = |input: &'a str| Ok((input, None));

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

    let parser = branch::alt((
        with_all,
        with_action_message,
        with_action_time,
        with_message_time,
        with_action,
        with_message,
        with_time,
        with_none,
    ));
    let (input, (position, action, message, datetime)) = parser(input)?;
    let (date, time) = match datetime {
        Some((date, time)) => (date, time),
        None => (None, None),
    };

    Ok((
        input,
        Point {
            position,
            action,
            message,
            date,
            time,
        },
    ))
}

fn parse_position(input: &str) -> ParseResult<Position> {
    let parse_northings = make_parse_coordinate('N', 7);
    let parse_eastings = make_parse_coordinate('E', 6);
    let parser = branch::permutation((parse_projection, parse_northings, parse_eastings));
    let (input, (projection, northings, eastings)) = parser(input)?;

    let position = Position {
        projection,
        northings,
        eastings,
    };
    Ok((input, position))
}

fn parse_projection(input: &str) -> ParseResult<Projection> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;

    let parser = sequence::tuple((
        branch::alt((bytes::tag_no_case("UTM"), bytes::tag(""))),
        character::digit1,
    ));
    let parsed = transform_parsed(parser(input), orig_input);
    let (input, (_, digits)) = parsed?;

    let projection = match digits {
        "32" => Projection::UTM32,
        "33" => Projection::UTM33,
        "34" => Projection::UTM34,
        "35" => Projection::UTM35,
        str if str.len() == 2 => {
            return Err(Err::Failure(ParseError::new(
                input,
                ErrorKind::ParseProjection,
            )))
        }
        _ => {
            return Err(Err::Error(ParseError::new(
                orig_input,
                ErrorKind::ParseProjection,
            )))
        }
    };
    Ok((input, projection))
}

fn make_parse_coordinate<'a>(
    c_char: char,
    c_len: usize,
) -> impl Fn(&str) -> ParseResult<Coordinate> {
    let c_str = c_char.to_string();
    move |input: &str| {
        let orig_input = input;
        let (input, _) = character::multispace1(input)?;

        let parser = sequence::tuple((
            branch::alt((bytes::tag_no_case(&c_str[..]), bytes::tag(""))),
            parse_int::<u32>,
            branch::alt((bytes::tag(","), bytes::tag("."), bytes::tag(""))),
            character::digit0,
        ));
        let parsed = transform_parsed(parser(input), orig_input);
        let (input, (prefix, (number, n_str), point, _)) = parsed?;

        if n_str.len() == c_len {
            Ok((input, number))
        } else if prefix.len() == 1 && point.len() == 0 && n_str.len() <= c_len && n_str.len() > 2 {
            let missing_pow = c_len - n_str.len();
            let number = number * 10u32.pow(missing_pow as u32);
            Ok((input, number))
        } else if n_str.len() <= c_len && point.len() == 1 {
            Err(Err::Failure(ParseError::new(
                orig_input,
                ErrorKind::ParseCoordinate,
            )))
        } else {
            Err(Err::Error(ParseError::new(
                orig_input,
                ErrorKind::ParseCoordinate,
            )))
        }
    }
}

fn parse_action(input: &str) -> ParseResult<Action> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;

    let parser = branch::alt((
        bytes::tag_no_case("tent"),
        bytes::tag_no_case("hut"),
        bytes::tag_no_case("food"),
    ));
    let parsed = transform_parsed(parser(input), orig_input);
    let (input, action) = parsed?;

    let action = match action {
        "tent" => Action::Tent,
        "hut" => Action::Hut,
        "food" => Action::Food,
        _ => panic!(),
    };
    Ok((input, action))
}

fn parse_message(input: &str) -> ParseResult<Message> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;

    let parser = sequence::delimited(
        bytes::tag("\""),
        bytes::escaped(character::none_of("\"\\"), '\\', character::one_of("\"\\")),
        bytes::tag("\""),
    );
    let (input, message) = transform_parsed(parser(input), orig_input)?;
    let message = message.to_string().replace("\\\"", "\"");

    Ok((input, message))
}

fn parse_datetime(input: &str) -> ParseResult<(Option<Date>, Option<Time>)> {
    let parser = sequence::tuple((parse_date, parse_time));
    let (input, ((date, _), (_, time))) = parser(input)?;
    Ok((input, (date, time)))
}

fn parse_date(input: &str) -> ParseResult<(Option<Date>, Option<Time>)> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;

    let parser = sequence::tuple((
        parse_int::<i32>,
        bytes::tag("-"),
        parse_int::<u32>,
        bytes::tag("-"),
        parse_int::<u32>,
    ));
    let parsed = transform_parsed(parser(input), orig_input);
    let (input, ((year, _), _, (month, _), _, (day, _))) = parsed?;

    let date = match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) if 1970 <= year && year < 2030 => date,
        Some(_) => {
            return Err(Err::Failure(ParseError::new(
                orig_input,
                ErrorKind::DateOutOfRange,
            )))
        }
        None => {
            return Err(Err::Failure(ParseError::new(
                orig_input,
                ErrorKind::ParseDate,
            )))
        }
    };
    Ok((input, (Some(Date(date)), None)))
}

fn parse_time<'a>(input: &'a str) -> ParseResult<(Option<Date>, Option<Time>)> {
    let orig_input = input;
    let (input, _) = character::multispace1(input)?;

    let parser = sequence::tuple((
        parse_int::<u32>,
        bytes::tag(":"),
        parse_int::<u32>,
        branch::alt((
            sequence::preceded(bytes::tag(":"), parse_int::<u32>),
            |input: &'a str| Ok((input, (0u32, &input[0..0]))),
        )),
    ));
    let parsed = transform_parsed(parser(input), orig_input)?;
    let (input, ((hour, _), _, (minute, _), (second, _))) = parsed;

    let time = match NaiveTime::from_hms_opt(hour, minute, second) {
        Some(time) => time,
        None => {
            return Err(Err::Failure(ParseError::new(
                orig_input,
                ErrorKind::ParseTime,
            )))
        }
    };
    Ok((input, (None, Some(Time(time)))))
}

fn parse_int<T: FromStr>(input: &str) -> ParseResult<(T, &str)> {
    let orig_input = input;

    let (input, digits) = character::digit1(input)?;
    let span = &orig_input[..(orig_input.len() - input.len())];

    let number = match digits.parse::<T>() {
        Ok(number) => Ok(number),
        Err(_) => Err(Err::Failure(ParseError::new(
            orig_input,
            ErrorKind::ParseNumber,
        ))),
    }?;
    Ok((input, (number, span)))
}

fn transform_parsed<'a, T>(output: ParseResult<'a, T>, orig_input: &'a str) -> ParseResult<'a, T> {
    match output {
        Ok(res) => Ok(res),
        Err(Err::Error(ParseError { error, .. })) => {
            Err(Err::Error(ParseError::new(orig_input, error)))
        }
        Err(Err::Failure(ParseError { error, .. })) => {
            Err(Err::Failure(ParseError::new(orig_input, error)))
        }
        _ => panic!(),
    }
}
