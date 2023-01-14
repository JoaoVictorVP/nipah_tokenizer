use std::str::Chars;

use crate::{options::{TokenizerOptions, IncludeMode, Scope, EndOfLine, SplitAggregator, SplitAggregatorFn}, split::SplitItem, token::{TokenPosition, Token}};


pub fn tokenize(entry: String, options: &TokenizerOptions) -> Vec<Token> {
    let pieces = split_string(entry, options);
    let mut tokens = vec![];
    for piece in pieces {
        let token = Token::build(piece, options);
        tokens.push(token);
    }
    tokens
}

type StringBuilder = Vec<char>;
trait StringBuilderPush {
    fn push_string(&mut self, text: &str) -> StringBuilder;
}
impl StringBuilderPush for StringBuilder {
    fn push_string(&mut self, text: &str) -> StringBuilder {
        self.extend(text.chars());
        self.clone()
    }
}

pub fn split_string(text: String, options: &TokenizerOptions) -> Vec<SplitItem> {
    let mut list = vec![];
    
    let mut position = 0_i32;
    let mut line = 0_i32;

    split_string_normal_mode(text, &mut position, &mut line, options, &mut list);

    list.retain(|f| !f.text.is_empty());

    let mut any_changed: bool;
    loop {
        (list, any_changed) = apply_aggregators(list.as_slice(), options.split_aggregators.as_slice());
        if !any_changed {
            break;
        }
    }

    list
}

fn apply_aggregators(mut inputs: &[SplitItem], aggregators: &[SplitAggregator]) -> (Vec<SplitItem>, bool) {
    let mut changed_any = false;
    let mut outputs = vec![];
    let mut carry = StringBuilder::new();

    while !inputs.is_empty() {
        let mut is_match = false;
        for aggregator in aggregators {
            carry.clear();
            if apply_aggregator(inputs, &aggregator.detectors, &mut carry, &mut outputs, None) {
                inputs = &inputs[aggregator.detectors.len()..];
                is_match = true;
                changed_any = true;
                break;
            }
        }
        if !is_match {
            outputs.push(inputs[0].clone());
            inputs = &inputs[1..];
        }
    }
    (outputs, changed_any)
}
fn apply_aggregator(inputs: &[SplitItem], aggregator: &[SplitAggregatorFn], carry: &mut StringBuilder, outputs: &mut Vec<SplitItem>, last_matched: Option<&SplitItem>) -> bool {
    let inp_len = inputs.len();
    let agr_len = aggregator.len();

    if agr_len == 0 {
        if !carry.is_empty() {
            outputs.push(SplitItem::new(carry.iter().collect(),
             if let Some(lm) = last_matched { lm.position } else { TokenPosition::new(0, 0) }));
        }
        true
    }
    else if agr_len == 0 && inp_len == 0 {
        return true;
    }
    else if inp_len >= agr_len && aggregator[0](&inputs[0].text) {
        return if !aggregator.is_empty() {
            apply_aggregator(&inputs[1..], &aggregator[1..], &mut carry.push_string(&inputs[0].text), outputs, Some(&inputs[0]))
        } else {
            outputs.push(SplitItem::new(carry.iter().collect(),
             if let Some(lm) = last_matched { lm.position } else { TokenPosition::new(0, 0) }));
            true
        }
    } else {
        false
    }
}

fn split_string_normal_mode(text: String, position: &mut i32, line: &mut i32, options: &TokenizerOptions, list: &mut Vec<SplitItem>) {
    let separators = &options.separators;
    let scopes = &options.scopes;
    let eofs = &options.end_of_lines;

    let chars = &mut text.chars();

    let mut current = StringBuilder::new();

    while let Some(c) = &chars.next() {
        current.push(*c);

        // Check for separators
        for sep in separators {
            let is_match = sep.cmatch == *c;
            if is_match {
                match sep.include {
                    IncludeMode::Aggregate => {
                        let item = SplitItem::new_pl(current.iter().collect(), *position, *line);
                        list.push(item);
                        current.clear();
                    },
                    IncludeMode::Separate => {
                        let item = SplitItem::new_pl(
                            if current.is_empty() { "".to_string() } else { current[..current.len() - 1].iter().collect() },
                            *position, *line);
                        let sep_item = SplitItem::new_pl(sep.cmatch.to_string(), *position, *line);
                        list.push(item);
                        list.push(sep_item);
                        current.clear();
                    },
                    IncludeMode::None => {
                        let item = SplitItem::new_pl(
                            if current.is_empty() { "".to_string() } else { current[..current.len() - 1].iter().collect() },
                            *position, *line);
                        list.push(item);
                        current.clear();
                    }
                }
                break;
            }
        }

        // Check for scopes
        for scope in scopes {
            if scope.begin == *c {
                current.clear();
                split_string_scoped_mode(chars, position, line, options, list, scope);
                break;
            }
        }

        // Check for escaping
        if *c == '\\' {
            current.pop();
            split_string_escaped_mode(chars, position, line, options, &mut current);
        }

        process_position_and_eof(*c, position, line, eofs);
    }
    if !current.is_empty() {
        let item = SplitItem::new_pl(current.iter().collect(), *position, *line);
        list.push(item);
    }
}

fn split_string_scoped_mode(chars: &mut Chars, position: &mut i32, line: &mut i32, options: &TokenizerOptions, list: &mut Vec<SplitItem>, scope: &Scope) {
    let eofs = &options.end_of_lines;

    let mut current = StringBuilder::new();

    current.push(scope.begin);
    
    while let Some(c) = chars.next() {
        current.push(c);

        // Check for the end of scope
        if scope.end == c {
            let item = SplitItem::new_pl(current.iter().collect(), *position, *line);
            list.push(item);
            process_position_and_eof(c, position, line, eofs);
            current.clear();
            return;
        }

        // Check for escaping
        if c == '\\' {
            current.pop();
            split_string_escaped_mode(chars, position, line, options, &mut current);
        }

        process_position_and_eof(c, position, line, eofs);
    }
    if !current.is_empty() {
        let item = SplitItem::new_pl(current.iter().collect(), *position, *line);
        list.push(item);
    }
}
fn split_string_escaped_mode<'c>(chars: &'c mut Chars, position: &mut i32, line: &mut i32, options: &TokenizerOptions, current: &mut Vec<char>) {
    if let Some(c) = chars.next() {
        if c == 'n' {
            current.push('\n');
        } else if c == 'r' {
            current.push('\r');
        } else if c == 't' {
            current.push('\t');
        } else {
            current.push(c);
        }
        process_position_and_eof(c, position, line, &options.end_of_lines);
    } else {
        // Should panic here or handle errors somehow
    }
}

fn process_position_and_eof(c: char, position: &mut i32, line: &mut i32, eofs: &Vec<EndOfLine>) {
    *position += 1;
    for eof in eofs {
        if eof.eof == c {
            *position = 0;
            *line += 1;
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::TokenType;

    use super::*;

    #[test]
    fn test_split_string() {
        let text = "Hello World!".to_string();
        let split = split_string(text, &crate::options::default());
        assert_eq!(split.len(), 3);
        assert_eq!(split[0].text, "Hello");
        assert_eq!(split[1].text, "World");
        assert_eq!(split[2].text, "!");
    }
    
    #[test]
    fn test_split_string_scoped() {
        let text = "\"Hello World!\"".to_string();
        let split = split_string(text, &crate::options::default());
        assert_eq!(split.len(), 1);
        assert_eq!(split[0].text, "\"Hello World!\"");
    }

    #[test]
    fn test_split_string_scoped_with_escape() {
        let text = "\"Hello World!\nYeah!\"".to_string();
        let split = split_string(text, &crate::options::default());
        assert_eq!(split.len(), 1);
        assert_eq!(split[0].text, "\"Hello World!\nYeah!\"");
    }

    #[test]
    fn test_split_with_aggregators() {
        let text = "Hello => World".to_string();
        let split = split_string(text, &crate::options::default());
        assert_eq!(split.len(), 3);
        assert_eq!(split[0].text, "Hello");
        assert_eq!(split[1].text, "=>");
        assert_eq!(split[2].text, "World");
    }

    #[test]
    fn test_split_with_aggregators_at_begin() {
        let text = "=> Hello World".to_string();
        let split = split_string(text, &crate::options::default());
        assert_eq!(split.len(), 3);
        assert_eq!(split[0].text, "=>");
        assert_eq!(split[1].text, "Hello");
        assert_eq!(split[2].text, "World");
    }

    #[test]
    fn test_split_with_aggregators_at_end() {
        let text = "Hello World =>".to_string();
        let split = split_string(text, &crate::options::default());
        assert_eq!(split.len(), 3);
        assert_eq!(split[0].text, "Hello");
        assert_eq!(split[1].text, "World");
        assert_eq!(split[2].text, "=>");
    }

    // Tokenizer
    #[test]
    fn can_tokenize() {
        let text = "Hello World!".to_string();
        let tokens = tokenize(text, &crate::options::default());
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].1.text, "Hello");
        assert_eq!(tokens[1].1.text, "World");
        assert_eq!(tokens[2].1.text, "!");
        assert_eq!(tokens[0].0, TokenType::Id("Hello".to_string()));
        assert_eq!(tokens[1].0, TokenType::Id("World".to_string()));
        assert_eq!(tokens[2].0, TokenType::Exclamation);
    }
}