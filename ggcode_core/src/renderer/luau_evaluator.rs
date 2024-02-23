use std::collections::BTreeMap;
use std::path::PathBuf;

use mlua::{Lua, LuaSerdeExt, Table};
use serde::Serialize;
use serde_yaml::{to_value, Value};

use crate::renderer::luau_extras::{LuauShell, trace_mlua_error};
use crate::types::{AppResult, ErrorBox};

#[derive(Default)]
pub struct LuauEvaluator {
    lua: Lua
}

#[derive(Default)]
pub struct LuauEvaluatorBuilder {
    pub globals: BTreeMap<String, Value>,
    pub paths: Vec<PathBuf>,
    pub shell: Option<LuauShell>,
    // pub(crate) template: Option<LuauTemplate>,
}

impl LuauEvaluatorBuilder {
    pub fn new() -> LuauEvaluatorBuilder {
        LuauEvaluatorBuilder::default()
    }

    pub fn with_global<S: Into<String>, V: Serialize + ?Sized>(mut self, key: S, value: &V) -> LuauEvaluatorBuilder {
        self.globals.insert(key.into(), to_value(value).unwrap());
        self
    }

    pub fn enable_shell(mut self, shell: LuauShell) -> LuauEvaluatorBuilder {
        self.shell = Some(shell);
        self
    }

    // pub fn enable_template(mut self, template: LuauTemplate) -> LuauEvaluatorBuilder {
    //     self.template = Some(template);
    //     self
    // }

    pub fn with_path_entry(mut self, entry: &PathBuf) -> LuauEvaluatorBuilder{
        self.paths.push(entry.clone());
        self
    }

    pub fn build(&self) -> AppResult<LuauEvaluator> {
        let lua = Lua::new();

        {
            let globals = &lua.globals();

            let search_path = &self.paths
                .iter()
                .map(|p| p.to_str().unwrap().to_string())
                .collect::<Vec<String>>()
                .join(";");

            // println!("Search Path: {}", search_path);

            for (key, value) in &self.globals {
                let lua_value = lua.to_value(value)?;
                globals.set(key.as_str(), lua_value)?;
            }

            if let Some(shell) = &self.shell {
                let userdata = lua.create_userdata(shell.clone())?;
                globals.set("shell", userdata)?;
            }

            globals
                .get::<_, Table>("package")?
                .set("path", search_path.clone())?;
        }

        let evaluator = LuauEvaluator {
            lua
        };
        Ok(evaluator)
    }
}

impl LuauEvaluator {
    pub fn eval_value(&self, script: &String) -> AppResult<Value> {
        let config_lua: mlua::Value = self.lua.load(script)
            .eval::<mlua::Value>()
            .map_err::<ErrorBox, _>(|e| {
                trace_mlua_error(script, &e.clone().into());
                format!("Error parsing template: {e}").into()
            })?;
        let config = self.lua.from_value::<Value>(config_lua)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::Value;

    use crate::renderer::luau_evaluator::LuauEvaluatorBuilder;
    use crate::renderer::luau_extras::{LuauShell, LuauTemplate};
    use crate::types::AppResult;

    #[test]
    fn eval_value_test() -> AppResult<()> {
        let evaluator = LuauEvaluatorBuilder::new()
            .with_global("one", "1")
            .with_global("two", "2")
            .enable_shell(LuauShell)
            .enable_template(LuauTemplate { st: String::new() })
            .build()?;

        let actual = evaluator.eval_value(&"`{one}/{two}`".into())?;
        let expected = Value::String("1/2".into());
        assert_eq!(actual, expected);

        let actual = evaluator.eval_value(&"shell:exec(`echo 'Hello'`)".into())?;
        let expected = Value::String("Hello".into());
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn eval_value_failure_test() -> AppResult<()> {
        let evaluator = LuauEvaluatorBuilder::new()
            .enable_shell(LuauShell)
            .enable_template(LuauTemplate { st: String::new() })
            .build()?;

        let script = "
            local one = 1
            local two = 2;;
            local three = 3
            local eleven = 11
            return {
                one = one,
                two = two,
                three = three,
                eleven = eleven
            }
        ";

        let actual = evaluator.eval_value(&script.into())?;
        let expected = Value::String("1/2".into());
        assert_eq!(actual, expected);

        Ok(())
    }
}