use std::collections::HashMap;

use regex::Regex;

mod parser;

// 1"^.{0,4}"2:".*\..*"|

struct RenamePattern {
    capture_groups: HashMap<usize, Regex>,
    elements: Vec<PatternElem>,
}

enum PatternElem {
    Literal(String),
    Insert(PatternInsert),
    Function(PatternFunction),
}

enum PatternInsert {
    Random,
    Original,
    CaptureGroup(usize),
    DateModified,
    Today,
}

enum PatternFunction {
    Uppercase(PatternInsert),
    Lowercase(PatternInsert),
}
