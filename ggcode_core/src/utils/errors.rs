use std::cmp;
use std::error::Error;
use console::style;

use lazy_static::lazy_static;
use mlua::ExternalError;
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
    pub is_pointed: bool,
}

pub fn describe_error(e: &ErrorBox) -> AppResult<Option<ErrorDescription>> {
    if e.is::<mlua::Error>() {
        return describe_mlua_error(e);
    }
    Ok(None)
}

fn describe_mlua_error(e: &Box<dyn Error>) -> AppResult<Option<ErrorDescription>> {
    let option: Option<(String, &Box<dyn Error>)>= match e.downcast_ref::<mlua::Error>().unwrap() {
        mlua::Error::SyntaxError { message, .. } => Some((message.clone(), e)),
        mlua::Error::RuntimeError(message) => Some((message.clone(), e)),
        mlua::Error::CallbackError { traceback, cause } => Some((cause.to_string(), e)),
        mlua::Error::MemoryError(_me) => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "MemoryError");
            None
        }
        // mlua::Error::GarbageCollectorError(_) => {
        //     eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "GarbageCollectorError");
        //     None
        // }
        mlua::Error::SafetyError(_) => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "SafetyError");
            None
        }
        mlua::Error::MemoryLimitNotAvailable => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "MemoryLimitNotAvailable");
            None
        }
        mlua::Error::RecursiveMutCallback => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "RecursiveMutCallback");
            None
        }
        mlua::Error::CallbackDestructed => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "CallbackDestructed");
            None
        }
        mlua::Error::StackError => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "StackError");
            None
        }
        mlua::Error::BindError => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "BindError");
            None
        }
        mlua::Error::BadArgument { .. } => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "BadArgument");
            None
        }
        mlua::Error::ToLuaConversionError { .. } => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "ToLuaConversionError");
            None
        }
        mlua::Error::FromLuaConversionError { .. } => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "FromLuaConversionError");
            None
        }
        mlua::Error::CoroutineInactive => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "CoroutineInactive");
            None
        }
        mlua::Error::UserDataTypeMismatch => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "UserDataTypeMismatch");
            None
        }
        mlua::Error::UserDataDestructed => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "UserDataDestructed");
            None
        }
        mlua::Error::UserDataBorrowError => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "UserDataBorrowError");
            None
        }
        mlua::Error::UserDataBorrowMutError => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "UserDataBorrowMutError");
            None
        }
        mlua::Error::MetaMethodRestricted(_) => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "MetaMethodRestricted");
            None
        }
        mlua::Error::MetaMethodTypeError { .. } => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "MetaMethodTypeError");
            None
        }
        mlua::Error::MismatchedRegistryKey => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "MismatchedRegistryKey");
            None
        }
        mlua::Error::CallbackError { .. } => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "CallbackError");
            None
        }
        mlua::Error::PreviouslyResumedPanic => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "PreviouslyResumedPanic");
            None
        }
        mlua::Error::SerializeError(_) => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "SerializeError");
            None
        }
        mlua::Error::DeserializeError(_) => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "DeserializeError");
            None
        }
        mlua::Error::ExternalError(_) => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "ExternalError");
            None
        }
        mlua::Error::WithContext { .. } => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "WithContext");
            None
        },
        _ => {
            eprintln!("{} {} handler is not implemented yet", style("[DEBUG]").cyan(), "UnknownError");
            None
        }
    };

    if let Some((message, _)) = option {
        lazy_static! {
            static ref RE_ERROR_V1: Regex = Regex::new(r#"(?m)^((?P<type>[^:]*): )?\[string "(?P<location>[^"]*)"]:(?P<line>[\d]+): (?P<details>.*)$"#).unwrap();
            static ref RE_ERROR_V2: Regex = Regex::new(r#"(?m)^((?P<type>[^:]*): )?(?P<location>[^:]*):(?P<line>[\d]+): (?P<details>.*)$"#).unwrap();
            static ref RE_ERROR_STACK_OVERFLOW: Regex = Regex::new(r#"(?m)^C stack overflow$"#).unwrap();
        }

        if let Some(v) = &RE_ERROR_STACK_OVERFLOW.captures(&message) {
            return Ok(Some(ErrorDescription::SourceError(SourceErrorData { location: "".into(), line: 0, details: message, is_pointed: false })))
        }

        if let Some(v) = &RE_ERROR_V1.captures(&message) {
            let location = v.name("location").unwrap().as_str().to_string();
            let details = v.name("details").unwrap().as_str().to_string();
            let line = v.name("line").unwrap().as_str().parse::<usize>()?;
            let line = cmp::max(0, (line as i32) - 1) as usize;
            return Ok(Some(ErrorDescription::SourceError(SourceErrorData { location, line, details, is_pointed: false })))
        }

        if let Some(v) = &RE_ERROR_V2.captures(&message) {
            let location = v.name("location").unwrap().as_str().to_string();
            let details = v.name("details").unwrap().as_str().to_string();
            let line = v.name("line").unwrap().as_str().parse::<usize>()?;
            let line = cmp::max(0, (line as i32) - 1) as usize;
            return Ok(Some(ErrorDescription::SourceError(SourceErrorData { location, line, details, is_pointed: true })))
        }
    }
    Ok(None)
}