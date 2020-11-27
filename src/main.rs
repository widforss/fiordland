mod error;
mod parser;

use nom_locate::LocatedSpan;
use parser::Command;

pub type Span<'a> = LocatedSpan<&'a str>;

const TEST: &'static str = "create
33 E618 7428602.235 2020-11-10 07:12:36 tent
\"Start of hike! Getting of the bus here.\"";

fn main() {
    println!("{:#?}", Command::parse(TEST));
}
