use std::collections::HashMap;
use regex::{CaptureMatches, Captures, Regex};

pub mod markdown_content;

pub mod html;


/// If the [string] contains the [pattern], replace it by [replace_by]. Otherwise returns
/// the original string without allocating a new one.
fn replace_if_contains(string: String, pattern: &str, replace_by: &str) -> String{
    if !string.contains(pattern) {
        return string;
    }
    string.replace(pattern, replace_by)
}

pub struct RegexHelper {
    replace_tokens_regex: Regex,
    identify_template_regex: Regex,
}

impl RegexHelper {
    pub fn new() -> RegexHelper {
        let replace_tokens_regex = Regex::new(r"\{\{(?<token>.*)}}").unwrap();
        let identify_template_regex = Regex::new(r"(?<whitespace>\s*)\[\[(?<template>.*)]]").unwrap();
        RegexHelper { replace_tokens_regex, identify_template_regex}
    }

    fn replace_tokens(&self, line: &str, token_values: &HashMap<&str, &String>) -> String {
         self.replace_tokens_regex.replace_all(line, |caps: &Captures| {
            token_values.get(&caps["token"]).unwrap()
        }).to_string()
    }

}

#[cfg(test)]
pub mod test {

    pub(crate) mod fixtures;
}
