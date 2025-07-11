use std::{collections::HashMap, error::Error};

use nom::{
    self, Parser,
    branch::alt,
    bytes::complete::tag,
    character::{anychar, char, complete::satisfy, digit1},
    combinator::{eof, opt},
    error::ParseError,
    multi::{self, many_till, many0, many1},
    sequence::{self, terminated},
};
use regex::Regex;
use thiserror::Error;

type PatternParseResult<'a, I, O> = Result<(I, O), nom::Err<PatternParseError<'a>>>;

#[derive(Debug, Error)]
pub enum PatternParseError<'a> {
    #[error("Parse Error!: {0}")]
    NomError(nom::error::Error<&'a str>),
    #[error("Regex Compile Error!: {0}")]
    RegexError(regex::Error),
    #[error("Unrecognized Insert!: {0}")]
    NonexistentInsert(&'a str),
    #[error("Unrecognized Capture Group! {0}")]
    NonexistentCapGroup(usize),
    #[error("{0}")]
    Other(Box<dyn Error>),
}

impl<'a> From<Box<dyn Error>> for PatternParseError<'a> {
    fn from(v: Box<dyn Error>) -> Self {
        Self::Other(v)
    }
}

impl<'a> From<nom::error::Error<&'a str>> for PatternParseError<'a> {
    fn from(v: nom::error::Error<&'a str>) -> Self {
        Self::NomError(v)
    }
}

impl<'a> From<regex::Error> for PatternParseError<'a> {
    fn from(v: regex::Error) -> Self {
        Self::RegexError(v)
    }
}

impl<'a> ParseError<&'a str> for PatternParseError<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Self::NomError(nom::error::Error::new(input, kind))
    }

    fn append(input: &str, kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

use super::{PatternElem, PatternFunction, PatternInsert, RenamePattern};

impl<'a> TryFrom<&'a str> for super::RenamePattern {
    type Error = nom::Err<PatternParseError<'a>>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let pattern = parse_pattern.parse(value).map(|x| x.1)?;
        for elem in pattern.elements.iter() {
            if let PatternElem::Insert(PatternInsert::CaptureGroup(cap_group)) = elem {
                if !pattern.capture_groups.contains_key(cap_group) {
                    return Err(nom::Err::Failure(PatternParseError::NonexistentCapGroup(
                        *cap_group,
                    )));
                }
            }
        }
        Ok(pattern)
    }
}

fn parse_pattern(inp: &str) -> PatternParseResult<&str, super::RenamePattern> {
    // let (capture_groups, elements) = match nom::sequence::separated_pair(
    //     parse_capture_groups,
    //     nom::character::char('|'),
    //     parse_pattern_elems,
    // )
    // .parse(inp)
    // {
    //     Ok(r) => {
    //         println!("{}", r.0);
    //         r.1
    //     }
    //     Err(e) => (HashMap::new(), parse_pattern_elems.parse(inp)?.1),
    // };
    //
    println!("orig input: {inp}");
    let (inp, capture_groups) = opt(parse_capture_groups).parse(inp)?;
    // let (inp, capture_groups) = match cap_group_res {
    //     Ok((inp, caps)) => (inp, caps),
    //     Err(e) => {
    //         let x = e.map(|x| match x {
    //             PatternParseError::NomError(error) => {
    //                 println!("scraggle {}|{:?}", error.input, error.code);
    //                 format!("scraggle {}|{:?}", error.input, error.code)
    //             }
    //             PatternParseError::RegexError(error) => todo!(),
    //             PatternParseError::NonexistentInsert(_) => todo!(),
    //             PatternParseError::Other(error) => todo!(),
    //         });
    //         println!("cap group error: {x:?}, ");
    //         ("", None)
    //     }
    // };
    let capture_groups = capture_groups.unwrap_or_default();
    let (inp, elements) = parse_pattern_elems
        .parse_complete(inp)
        .inspect_err(|e| println!("elems err {e}"))?;
    Ok((inp, RenamePattern {
        capture_groups,
        elements,
    }))
}

fn parse_capture_groups(inp: &str) -> PatternParseResult<&str, HashMap<usize, Regex>> {
    many_till(parse_capture_group, char('|'))
        .parse(inp)
        .map(|(inp, (res, _))| (inp, res.into_iter().collect()))
}

fn parse_capture_group(inp: &str) -> PatternParseResult<&str, (usize, Regex)> {
    println!("Parsing cap group: {inp}");
    let (inp, parsed_id) = digit1().parse(inp)?;
    println!("Parsed id");
    let group_id: usize = match str::parse::<usize>(parsed_id) {
        Ok(r) => r,
        Err(e) => {
            let x: Box<dyn Error> = Box::new(e);
            return Err(nom::Err::Failure(PatternParseError::from(x)));
        }
    };
    let (inp, _) = char('"').parse(inp)?;
    let (inp, regex) = parse_cap_group_regex.parse(inp)?;
    println!("Finishing parsing cap group : {inp}, {regex:?}");
    Ok((inp, (group_id, regex)))
}

fn parse_cap_group_regex(inp: &str) -> PatternParseResult<&str, Regex> {
    let (inp, regex_text) = many_till(satisfy(|c| c != '"'), char('"')).parse(inp)?;
    Ok((
        inp,
        compile_regex(regex_text.0.into_iter().collect::<String>())
            .map_err(|e| nom::Err::Failure(e))?,
    ))
}

fn compile_regex(inp: String) -> Result<Regex, PatternParseError<'static>> {
    let parsed_regex = Regex::new(&inp).map_err(|x| PatternParseError::from(x))?;
    Ok(parsed_regex)
}

fn parse_pattern_elems(inp: &str) -> PatternParseResult<&str, Vec<PatternElem>> {
    many_till(parse_pattern_elem, eof)
        .parse_complete(inp)
        .map(|(inp, (res, _))| (inp, res))
}

fn parse_pattern_elem(inp: &str) -> PatternParseResult<&str, PatternElem> {
    alt((
        // parse_function,
        parse_capture_group_insert,
        parse_insert,
        parse_literal,
    ))
    .parse_complete(inp)
}

fn parse_literal(inp: &str) -> PatternParseResult<&str, PatternElem> {
    many1(satisfy(|c| c != '/'))
        .parse_complete(inp)
        .map(|(inp, res)| (inp, PatternElem::Literal(res.into_iter().collect())))
}

fn parse_insert(inp: &str) -> PatternParseResult<&str, PatternElem> {
    let orig_inp = inp;
    let (inp, _) = char('/').parse(inp)?;

    let (inp, insert_chars) = many1(satisfy(|c| c != '/')).parse(inp)?;
    let insert_string: String = insert_chars.into_iter().collect();
    let insert = match PatternInsert::try_from(insert_string.as_str()) {
        Ok(i) => i,
        Err(_) => {
            return Err(nom::Err::Failure(PatternParseError::NonexistentInsert(
                orig_inp,
            )));
        }
    };
    let (inp, _) = char('/').parse(inp)?;

    Ok((inp, PatternElem::Insert(insert)))
}
fn parse_capture_group_insert(inp: &str) -> PatternParseResult<&str, PatternElem> {
    let (inp, _) = char('/').parse(inp)?;
    let (inp, _) = tag("cap").parse(inp)?;
    let (inp, group_id) = digit1().parse(inp)?;
    let (inp, _) = char('/').parse(inp)?;
    Ok((
        inp,
        PatternElem::Insert(PatternInsert::CaptureGroup(str::parse(group_id).unwrap())),
    ))
}

fn parse_function(inp: &str) -> PatternParseResult<&str, PatternElem> {
    todo!()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use regex::Regex;

    use crate::patterns::{PatternElem, PatternInsert, RenamePattern};

    /**
    1"^.{0,4}"2:".*\..*"|/1//RAND/./2/
    2 capture groups, [CaptureGroup(1), Insert::Rand, Literal("."), CaptureGroup(2)]
    */
    #[test]
    fn basic_test() {
        let input = "1\"^.{0,4}\"2\".*\\..*\"|/cap1//RAND/./cap2/";
        let expected = RenamePattern {
            capture_groups: vec![
                (1, Regex::new(r"^.{0,4}").unwrap()),
                (2, Regex::new(".*\\..*").unwrap()),
            ]
            .into_iter()
            .collect(),
            elements: vec![
                PatternElem::Insert(PatternInsert::CaptureGroup(1)),
                PatternElem::Insert(PatternInsert::Random),
                PatternElem::Literal(".".to_owned()),
                PatternElem::Insert(PatternInsert::CaptureGroup(2)),
            ],
        };
        let res = super::parse_pattern(input).unwrap().1;
        if res != expected {
            panic!("res: {res:?} != expected: {expected:?}")
        }
    }
    #[test]
    fn no_capture_groups() {
        let input = "/RAND/hello/RAND/";
        let expected = RenamePattern {
            capture_groups: HashMap::default(),
            elements: vec![
                PatternElem::Insert(PatternInsert::Random),
                PatternElem::Literal("hello".to_owned()),
                PatternElem::Insert(PatternInsert::Random),
            ],
        };
        let res = super::parse_pattern(input).unwrap();
        assert!(res.1 == expected)
    }
    #[test]
    fn parse_literal() {
        let input = "hello";
        let expected = RenamePattern {
            capture_groups: HashMap::default(),
            elements: vec![PatternElem::Literal("hello".to_owned())],
        };
        let res = super::parse_pattern(input).unwrap();
        assert!(res.1 == expected)
    }
}
