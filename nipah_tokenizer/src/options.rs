use std::{sync::{atomic::AtomicI64}, rc::Rc};

use lazy_static::lazy_static;

pub struct TokenizerOptions {
    pub separators: Vec<Separator>,
    pub scopes: Vec<Scope>,
    pub end_of_lines: Vec<EndOfLine>,
    pub split_aggregators: Vec<SplitAggregator>,
    pub try_id: Rc<fn(&str) -> bool>
}
impl TokenizerOptions {
    fn new(separators: Vec<Separator>, scopes: Vec<Scope>, end_of_lines: Vec<EndOfLine>, split_aggregators: Vec<SplitAggregator>) -> Self {
        TokenizerOptions {
            separators,
            scopes,
            end_of_lines,
            split_aggregators,
            try_id: Rc::new(default_try_id)
        }
    }
}

lazy_static! {
    pub static ref DEFAULT_SEPARATORS: [Separator; 27] = [
        Separator::new(' ', IncludeMode::None),
        Separator::new('\t', IncludeMode::None),
    
        Separator::new_sep('*'), Separator::new_sep('/'), Separator::new_sep('+'), Separator::new_sep('-'),
        Separator::new_sep('('), Separator::new_sep(')'), Separator::new_sep('\n'), Separator::new_sep(','), Separator::new_sep(';'),
        Separator::new_sep('='), Separator::new_sep('{'), Separator::new_sep('}'),
        Separator::new_sep('['), Separator::new_sep(']'), Separator::new_sep(':'), Separator::new_sep('<'), Separator::new_sep('>'),
        Separator::new_sep('&'), Separator::new_sep('|'), Separator::new_sep('$'), Separator::new_sep('@'), Separator::new_sep('.'), Separator::new_sep('#'),
        Separator::new_sep('!'), Separator::new_sep('?')
    ];
    pub static ref DEFAULT_SCOPES: [Scope; 2] = [
        Scope::new('"', '"'),
        Scope::new('\'', '\'')
    ];
    pub static ref DEFAULT_END_OF_LINES: [EndOfLine; 2] = [
        EndOfLine::new('\n'),
        EndOfLine::new('\0')
    ];
}

pub fn default_try_id(entry: &str) -> bool {
    if entry.is_empty() {
        return false;
    }

    let mut chars = entry.chars();

    let first_valid = if let Some(frc) = chars.next() {
        if frc.is_ascii_digit() || frc == '_' {
            return false;
        }
        true
    } else { false };
    if !first_valid {
        return false;
    }
    
    for c in chars {
        if c.is_ascii_digit() || c.is_ascii_alphabetic() || c == '_' || c == '.' {
            continue;
        }
        return false;
    }
    true
}

pub fn default_split_aggregators() -> [SplitAggregator; 13] {
    [
        SplitAggregator::from_strings(vec!["=".to_string(), "=".to_string()]),
        SplitAggregator::from_strings(vec!["!".to_string(), "=".to_string()]),
        SplitAggregator::from_strings(vec![">".to_string(), "=".to_string()]),
        SplitAggregator::from_strings(vec!["<".to_string(), "=".to_string()]),
        SplitAggregator::from_strings(vec!["/".to_string(), "/".to_string()]),
        SplitAggregator::from_strings(vec!["/".to_string(), "*".to_string()]),
        SplitAggregator::from_strings(vec!["*".to_string(), "/".to_string()]),
        SplitAggregator::from_strings(vec!["-".to_string(), ">".to_string()]),
        SplitAggregator::from_strings(vec!["=".to_string(), ">".to_string()]),
        SplitAggregator::from_strings(vec!["&".to_string(), "&".to_string()]),
        SplitAggregator::from_strings(vec!["|".to_string(), "|".to_string()]),

        SplitAggregator::new(vec![Rc::new(|x: &String| is_numeric(x, false)), Rc::new(|x: &String| x == "."), Rc::new(|x: &String| is_numeric(x, true))]),
        SplitAggregator::new(vec![Rc::new(|x: &String| x == "-"), Rc::new(|x: &String| is_numeric_accept_float(x))])
    ]
}

fn is_numeric(x: &str, accept_f: bool) -> bool {
    let chars = x.chars();
    for c in chars {
        if c == '-' {
            continue;
        }
        if c == 'f' {
            if !accept_f {
                return false;
            }
            continue;
        }
        if !c.is_ascii_digit() {
            return false;
        }
    }
    true
}
fn is_numeric_accept_float(x: &str) -> bool {
    let chars = x.chars();
    let mut has_dot = false;
    for c in chars {
        if c == '.' {
            if has_dot {
                return false;
            }
            has_dot = true;
            continue;
        }
        if c == 'f' {
            continue;
        }
        if !c.is_ascii_digit() {
            return false;
        }
    }
    true
}

pub fn default() -> TokenizerOptions {
    TokenizerOptions::new(
        DEFAULT_SEPARATORS.to_vec(),
         DEFAULT_SCOPES.to_vec(),
          DEFAULT_END_OF_LINES.to_vec(),
           default_split_aggregators().to_vec()
    )
}

#[derive(Clone)]
pub struct Separator {
    pub cmatch: char,
    pub include: IncludeMode
}
#[derive(Clone)]
pub enum IncludeMode {
    None,
    Aggregate,
    Separate
}
impl Separator {
    fn new(cmatch: char, include: IncludeMode) -> Separator {
        Separator { cmatch, include }
    }
    
    fn new_sep(cmatch: char) -> Separator {
        Separator::new(cmatch, IncludeMode::Separate)
    }
}

#[derive(Clone)]
pub struct Scope {
    pub id: i64,
    pub begin: char,
    pub end: char
}
impl Scope {
    fn new(begin: char, end: char) -> Self {
        static COUNTER: AtomicI64 = AtomicI64::new(0);

        Scope {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            begin,
            end
        }
    }
}

#[derive(Clone)]
pub struct EndOfLine {
    pub eof: char
}
impl EndOfLine {
    fn new(eof: char) -> EndOfLine {
        EndOfLine {
            eof
        }
    }
}

pub type SplitAggregatorFn = Rc<dyn Fn(&String) -> bool + Send + Sync>;
#[derive(Clone)]
pub struct SplitAggregator {
    pub detectors: Vec<SplitAggregatorFn>
}
impl SplitAggregator {
    pub fn new(detectors: Vec<SplitAggregatorFn>) -> Self {
        SplitAggregator {
            detectors
        }
    }
    pub fn from_strings(detectors: Vec<String>) -> Self {
        let mut fin = Vec::<SplitAggregatorFn>::new();
        for detector in detectors {
            fin.push(Self::string_matcher(detector));
        }
        SplitAggregator {
            detectors: fin
        }
    }
    fn string_matcher(detector: String) -> SplitAggregatorFn {
        Rc::new(move |cmp: &String| *cmp == detector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_default_id_ok() {
        let id = "hello_world".to_string();
        assert!(default_try_id(&id));
    }

    #[test]
    fn try_default_id_fail() {
        let id = "1hello_world".to_string();
        assert!(!default_try_id(&id));
    }
}