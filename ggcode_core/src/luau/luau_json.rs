use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataMethods};
use mlua::Error::RuntimeError;
use serde_json::Value;

use crate::types::AppResult;

#[derive(Debug, Copy, Clone)]
pub struct LuauJson;

impl LuauJson {
    fn stringify(value: &Value) -> AppResult<String> {
        let output = serde_json::to_string(value)?;
        return Ok(output)
    }
}

impl UserData for LuauJson {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("stringify", |lua, (_ud, value): (AnyUserData, mlua::Value)| {
            let result = lua.from_value(value);
            match result {
                Ok(yaml) => Self::stringify(&yaml).or_else(|e| Err(RuntimeError(format!("Cannot serialize value. {}", e).to_string()))),
                Err(e) => return Err(RuntimeError(format!("Cannot serialize value. {}", e).to_string())),
            }
        })
    }
}
