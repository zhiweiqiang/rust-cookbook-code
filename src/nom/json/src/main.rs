extern crate nom;

use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alphanumeric1 as alphanumeric, char, one_of},
    combinator::{cut, map, opt, value},
    error::{context, ContextError, ErrorKind, ParseError},
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

use nom::bytes::complete::tag_no_case;
use std::collections::HashMap;
use std::str;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Str(String),
    Boolean(bool),
    Num(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &str, E> {
    let chars = "\t\r\n ";
    take_while(move |c| chars.contains(c))(i)
}

fn parse_str<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    escaped(alphanumeric, '\\', one_of("\"n\\"))(i)
}

fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    let parse_true = value(true, tag_no_case("true"));
    let parse_false = value(false, tag_no_case("false"));
    alt((parse_true, parse_false))(input)
}

fn string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    context(
        "string",
        preceded(char('\"'), cut(terminated(parse_str, char('\"')))),
    )(i)
}

fn array<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Vec<JsonValue>, E> {
    context(
        "array",
        preceded(
            char('['),
            cut(terminated(
                separated_list0(preceded(sp, char(',')), json_value),
                preceded(sp, char(']')),
            )),
        ),
    )(i)
}

fn key_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, JsonValue), E> {
    separated_pair(
        preceded(sp, string),
        cut(preceded(sp, char(':'))),
        json_value,
    )(i)
}

fn hash<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, HashMap<String, JsonValue>, E> {
    context(
        "map",
        preceded(
            char('{'),
            cut(terminated(
                map(
                    separated_list0(preceded(sp, char(',')), key_value),
                    |tuple_vec| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| (String::from(k), v))
                            .collect()
                    },
                ),
                preceded(sp, char('}')),
            )),
        ),
    )(i)
}

fn json_value<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, JsonValue, E> {
    preceded(
        sp,
        alt((
            map(hash, JsonValue::Object),
            map(array, JsonValue::Array),
            map(string, |s| JsonValue::Str(String::from(s))),
            map(double, JsonValue::Num),
            map(boolean, JsonValue::Boolean),
        )),
    )(i)
}

fn root<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, JsonValue, E> {
    delimited(
        sp,
        alt((map(hash, JsonValue::Object), map(array, JsonValue::Array))),
        opt(sp),
    )(i)
}

fn main() {
    let data = "{\"name\":[\"iamazy\",1,falsE,{\"age\":1}]}";

    println!(
        "will try to parse valid JSON data: \n\n*************\n{}\n*************\n",
        data
    );

    println!(
        "parsing a valid file:\n{:#?}\n",
        root::<(&str, ErrorKind)>(data)
    );
}
