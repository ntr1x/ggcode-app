use mlua::{AnyUserData, UserData, UserDataMethods};
use mlua::Error::RuntimeError;
use uuid::Uuid;

use crate::types::AppResult;

#[derive(Debug, Copy, Clone)]
pub struct LuauUuid;

impl LuauUuid {
    fn v4() -> AppResult<String> {
        let v4 = Uuid::new_v4();
        return Ok(v4.to_string())
    }
}

impl UserData for LuauUuid {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("v4", |_, _ud: AnyUserData| {
            return Self::v4().or_else(|e| Err(RuntimeError(format!("Cannot generate UUID.V4 value. {}", e).to_string())));
        })
    }
}
