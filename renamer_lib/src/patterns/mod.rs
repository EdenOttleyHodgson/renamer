use std::{collections::HashMap, error::Error, fs, iter::zip, path::PathBuf};

use regex::Regex;

pub use parser::PatternParseError;

use crate::error::SendableErr;
mod parser;

/**
1"^.{0,4}"2:".*\..*"|/1//RAND/./2/
2 capture groups, [CaptureGroup(1), Insert::Rand, Literal("."), CaptureGroup(2)]
*/

#[derive(Debug, Clone)]
pub struct RenamePattern {
    capture_groups: HashMap<usize, Regex>,
    elements: Vec<PatternElem>,
    preset_info: Option<&'static str>,
    input: Option<String>,
}
impl RenamePattern {
    pub fn randomize() -> Self {
        Self {
            capture_groups: HashMap::default(),
            elements: vec![PatternElem::Insert(PatternInsert::Random)],
            preset_info: Some("Randomize"),
            input: None,
        }
    }
    pub fn apply_to_file_name(&self, fpath: &PathBuf) -> Result<PathBuf, SendableErr> {
        let fpath = fpath.canonicalize()?;
        let fname = fpath.file_name().unwrap().to_string_lossy().to_string(); //TODO: Error handle
        let mut capture_group_texts: HashMap<usize, String> = HashMap::new();
        for (id, regex) in self.capture_groups.iter() {
            let cap_text = regex.find_iter(&fname).fold(String::new(), |mut acc, s| {
                acc.push_str(s.as_str());
                acc
            });
            capture_group_texts.insert(*id, cap_text);
        }
        let mut out_name = String::new();
        for element in self.elements.iter() {
            let to_push = match element {
                PatternElem::Literal(lit) => lit,
                PatternElem::Insert(pattern_insert) => match pattern_insert {
                    PatternInsert::Random => &rand::random::<u32>().to_string(),
                    PatternInsert::Original => &fname,
                    PatternInsert::CaptureGroup(id) => capture_group_texts
                        .get(id)
                        .expect("Capture groups existence ensured by the parser"),
                    PatternInsert::DateModified => {
                        let date_time: chrono::DateTime<chrono::Local> =
                            fs::metadata(&fpath).unwrap().modified().unwrap().into();
                        &date_time.to_rfc3339()
                    } //Error handle
                    PatternInsert::Now => &chrono::Local::now().to_rfc3339(),
                },
                PatternElem::Function(pattern_function) => todo!(),
            };
            out_name.push_str(to_push);
        }
        let mut new_path = fpath.clone();
        new_path.pop();
        new_path.push(out_name);
        Ok(new_path)
    }

    pub fn preset_info(&self) -> Option<&'static str> {
        self.preset_info
    }

    pub fn input(&self) -> Option<&String> {
        self.input.as_ref()
    }
}

impl PartialEq for RenamePattern {
    fn eq(&self, other: &Self) -> bool {
        if self.capture_groups.len() != other.capture_groups.len() {
            return false;
        }
        if !(self.elements == other.elements) {
            return false;
        } else {
            for id in self.capture_groups.keys() {
                if let Some(l_regex) = self.capture_groups.get(id) {
                    if let Some(r_regex) = other.capture_groups.get(id) {
                        if !(l_regex.as_str() == r_regex.as_str()) {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(PartialEq, Debug, Clone)]
enum PatternElem {
    Literal(String),
    Insert(PatternInsert),
    Function(PatternFunction),
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum PatternInsert {
    Random,
    Original,
    CaptureGroup(usize),
    DateModified,
    Now,
}
impl<'a> TryFrom<&'a str> for PatternInsert {
    type Error = parser::PatternParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "RAND" => Ok(Self::Random),
            "ORIG" | "ORIGINAL" => Ok(Self::Original),
            "DATE_MODIFIED" => Ok(Self::DateModified),
            "NOW" => Ok(Self::Now),
            _ => Err(parser::PatternParseError::NonexistentInsert(
                value.to_owned(),
            )),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
enum PatternFunction {
    Uppercase(PatternInsert),
    Lowercase(PatternInsert),
}

#[cfg(test)]
mod test {
    use std::{fs, path::PathBuf};

    use super::RenamePattern;

    #[test]
    fn basic_test() {
        let input_pattern = r#"1"^.{0,4}"2"\..*"|/cap1//RAND/#/cap2/"#;
        let input_files: Vec<PathBuf> = vec!["Ploggle.txt".into(), "Groggle.jpeg".into()];
        for file in input_files.iter() {
            fs::File::create(file);
        }
        let expected: Vec<(String, String)> = vec![
            ("Plog".into(), "#.txt".into()),
            ("Grog".into(), "#.jpeg".into()),
        ];
        let pattern = RenamePattern::try_from(input_pattern).unwrap();
        let result: Vec<(String, String, String)> = input_files
            .iter()
            .map(|path| pattern.apply_to_file_name(path).unwrap())
            .map(|x| {
                println!("{}", x.display());
                let left = x.file_name().unwrap().to_string_lossy().to_string()[0..4].to_owned();
                let (middle, last) = {
                    let mut s = x.file_stem().unwrap().to_string_lossy().to_string();
                    (s[4..s.len() - 2].to_owned(), s.pop().unwrap())
                };
                let right = format!("{last}.{}", x.extension().unwrap().to_string_lossy());
                (left, middle, right)
            })
            .collect();
        for file in input_files.iter() {
            fs::remove_file(file);
        }
        println!("{result:?}");
        for res in result.iter().map(|x| &x.1) {
            str::parse::<u32>(res).unwrap();
        }
        let result: Vec<_> = result.into_iter().map(|x| (x.0, x.2)).collect();
        assert!(result == expected, "{result:?} != {expected:?}")
    }
}
