use std::{collections::HashMap, error::Error};

use nom::{
    self, Parser,
    branch::alt,
    bytes::complete::tag,
    character::{char, complete::satisfy, digit1},
    combinator::{eof, opt},
    error::ParseError,
    multi::{many_till, many1},
};
use regex::Regex;
use thiserror::Error;

type PatternParseResult<'a, I, O> = Result<(I, O), nom::Err<PatternParseError>>;

#[derive(Debug, Error)]
pub enum PatternParseError {
    #[error("Parse Error!: {0}")]
    NomError(nom::error::Error<String>),
    #[error("Regex Compile Error!: {0}")]
    RegexError(regex::Error),
    #[error("Unrecognized Insert!: {0}")]
    NonexistentInsert(String),
    #[error("Unrecognized Capture Group! {0}")]
    NonexistentCapGroup(usize),
    #[error("{0}")]
    Other(Box<dyn Error>),
}

impl From<Box<dyn Error>> for PatternParseError {
    fn from(v: Box<dyn Error>) -> Self {
        Self::Other(v)
    }
}

impl From<nom::error::Error<String>> for PatternParseError {
    fn from(v: nom::error::Error<String>) -> Self {
        Self::NomError(v)
    }
}

impl From<regex::Error> for PatternParseError {
    fn from(v: regex::Error) -> Self {
        Self::RegexError(v)
    }
}

impl<'a> ParseError<&'a str> for PatternParseError {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Self::NomError(nom::error::Error::new(input.to_owned(), kind))
    }

    fn append(_: &str, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

use super::{ActionOptions, PatternElem, PatternInsert, RenamePattern};

impl<'a> TryFrom<&'a str> for RenamePatternIntermediate {
    type Error = nom::Err<PatternParseError>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let pattern = parse_pattern.parse(value).map(|x| x.1)?;
        println!("{pattern:?}");
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

impl RenamePattern {
    pub fn parse(inp: &str, options: ActionOptions) -> Result<Self, Box<dyn Error>> {
        let intermediate = RenamePatternIntermediate::try_from(inp)?;
        Ok(Self {
            capture_groups: intermediate.capture_groups,
            elements: intermediate.elements,
            preset_info: None,
            input: Some(inp.to_owned()),
            options,
        })
    }
}

#[derive(Debug)]
struct RenamePatternIntermediate {
    capture_groups: HashMap<usize, Regex>,
    elements: Vec<PatternElem>,
}

fn parse_pattern(inp: &str) -> PatternParseResult<&str, RenamePatternIntermediate> {
    let (inp, capture_groups) = opt(parse_capture_groups).parse(inp)?;
    let capture_groups = capture_groups.unwrap_or_default();
    let (inp, elements) = parse_pattern_elems
        .parse_complete(inp)
        .inspect_err(|e| println!("elems err {e}"))?;
    Ok((inp, RenamePatternIntermediate {
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
    log::trace!("Parsing cap group: {inp}");
    let (inp, parsed_id) = digit1().parse(inp)?;
    let group_id: usize = match str::parse::<usize>(parsed_id) {
        Ok(r) => r,
        Err(e) => {
            let x: Box<dyn Error> = Box::new(e);
            return Err(nom::Err::Failure(PatternParseError::from(x)));
        }
    };
    let (inp, _) = char('"').parse(inp)?;
    let (inp, regex) = parse_cap_group_regex.parse(inp)?;
    log::trace!("Finishing parsing cap group : {inp}, {regex:?}");
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

fn compile_regex(inp: String) -> Result<Regex, PatternParseError> {
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
                orig_inp.to_owned(),
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use regex::Regex;

    use crate::patterns::{ActionOptions, PatternElem, PatternInsert, RenamePattern};

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
            preset_info: None,
            input: Some(input.to_owned()),
            options: ActionOptions::default(),
        };
        let res = RenamePattern::parse(input, ActionOptions::default()).unwrap();
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
            preset_info: None,
            input: Some(input.to_owned()),
            options: ActionOptions::default(),
        };
        let res = RenamePattern::parse(input, ActionOptions::default()).unwrap();
        assert!(res == expected)
    }
    #[test]
    fn parse_literal() {
        let input = "hello";
        let expected = RenamePattern {
            capture_groups: HashMap::default(),
            elements: vec![PatternElem::Literal("hello".to_owned())],
            preset_info: None,
            input: Some(input.to_owned()),
            options: ActionOptions::default(),
        };
        let res = RenamePattern::parse(input, ActionOptions::default()).unwrap();
        assert!(res == expected)
    }
}
