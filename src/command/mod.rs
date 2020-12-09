mod parser;

use chrono::prelude::{NaiveDate, NaiveTime};
use serde::{Serialize, Serializer};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Command {
    Create(Vec<Point>),
    Edit(Vec<Point>),
    Checkin(Point),
    Complete,
}

#[derive(Serialize)]
pub struct Point {
    position: Position,
    action: Option<Action>,
    message: Option<Message>,
    date: Option<Date>,
    time: Option<Time>,
}

#[derive(Serialize)]
pub struct Position {
    #[serde(rename = "srid")]
    projection: Projection,
    eastings: Coordinate,
    northings: Coordinate,
}

type Coordinate = u32;

pub enum Projection {
    UTM32,
    UTM33,
    UTM34,
    UTM35,
}

#[derive(Serialize)]
pub enum Action {
    Food,
    Tent,
    Hut,
}

type Message = String;

pub struct Date(NaiveDate);
pub struct Time(NaiveTime);

impl Serialize for Projection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use Projection::*;
        match self {
            UTM32 => serializer.serialize_i32(25832),
            UTM33 => serializer.serialize_i32(25833),
            UTM34 => serializer.serialize_i32(25834),
            UTM35 => serializer.serialize_i32(25835),
        }
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Date(date) = self;
        serializer.serialize_str(&format!("{}", date)[..])
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Time(time) = self;
        serializer.serialize_str(&format!("{}", time)[..])
    }
}
