use std::cmp;
use std::error::Error;

use lazy_static::lazy_static;
use regex::Regex;

use crate::types::{AppResult, ErrorBox};

pub enum ErrorDescription {
    SourceError(SourceErrorData)
}


#[derive(Default, Clone)]
pub struct SourceErrorData {
    pub location: String,
    pub line: usize,
    pub details: String,
}

pub fn describe_error(e: &ErrorBox) -> AppResult<Option<ErrorDescription>> {
    if e.is::<mlua::Error>() {
        return describe_mlua_error(e);
    }
    Ok(None)
}

fn describe_mlua_error(e: &Box<dyn Error>) -> AppResult<Option<ErrorDescription>> {
    match e.downcast_ref::<mlua::Error>().unwrap() {
        mlua::Error::SyntaxError { message, .. } => {
            lazy_static! {
                static ref RE_SYNTAX_ERROR: Regex = Regex::new(r#"(?m)^\[string "(?P<location>[^"]*)"]:(?P<line>[\d]+): (?P<details>.*)$"#).unwrap();
            }
            match &RE_SYNTAX_ERROR.captures(message) {
                None => Ok(None),
                Some(v) => {
                    let location = v.name("location").unwrap().as_str().to_string();
                    let details = v.name("details").unwrap().as_str().to_string();
                    let line = v.name("line").unwrap().as_str().parse::<usize>()?;
                    let line = cmp::min(0, (line as i32) - 1) as usize;
                    Ok(Some(ErrorDescription::SourceError(SourceErrorData { location, line, details })))
                }
            }
        }
        mlua::Error::RuntimeError(message) => {
            lazy_static! {
                static ref RE_RUNTIME_ERROR: Regex = Regex::new(r#"(?m)^\[string "(?P<location>[^"]*)"]:(?P<line>[\d]+): (?P<details>.*)$"#).unwrap();
            }
            match &RE_RUNTIME_ERROR.captures(message) {
                None => Ok(None),
                Some(v) => {
                    let location = v.name("location").unwrap().as_str().to_string();
                    let details = v.name("details").unwrap().as_str().to_string();
                    let line = v.name("line").unwrap().as_str().parse::<usize>()?;
                    let line = cmp::min(0, (line as i32) - 1) as usize;
                    Ok(Some(ErrorDescription::SourceError(SourceErrorData { location, line, details })))
                }
            }
        }
        _ => Ok(None)
    }
}