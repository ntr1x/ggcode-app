use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

use mlua::{Lua, LuaSerdeExt};
use serde_yaml::Value;

use crate::luau::luau_json::LuauJson;
use crate::luau::luau_uuid::LuauUuid;
use crate::luau::luau_yaml::LuauYaml;
use crate::renderer::builder::RendererBuilder;
use crate::renderer::luau_extras::{LuauTemplate, trace_mlua_error};
use crate::types::ErrorBox;

#[derive(Debug)]
pub struct LuaRenderer {
    values: BTreeMap<String, Value>,
    templates: BTreeMap<String, String>,
}

impl LuaRenderer {
    pub fn render<N: Into<String>>(&self, name: N) -> Result<String, Box<dyn Error>> {
        let name_string = &name.into();
        let lua = Lua::new();

        let globals = lua.globals();

        for (key, value) in &self.values {
            let lua_value = lua.to_value(value)?;
            globals.set(key.as_str(), lua_value)?;
        }

        let template = lua.create_userdata(LuauTemplate {
            st: String::new()
        })?;

        globals.set("template", &template)?;

        globals.set("null", lua.null())?;
        globals.set("array_mt", lua.array_metatable())?;
        globals.set("yaml", &lua.create_userdata(LuauYaml)?)?;
        globals.set("json", &lua.create_userdata(LuauJson)?)?;
        globals.set("uuid", lua.create_userdata(LuauUuid)?)?;


        let script = self.templates
            .get(name_string)
            .ok_or::<Box<dyn Error>>(format!("No template: {}", name_string).into())?;

        lua.load(script).exec()
            .map_err::<ErrorBox, _>(|e| {
                trace_mlua_error(script, &e.clone().into());
                format!("Error parsing template: {e}").into()
            })?;

        let result = template.borrow::<LuauTemplate>()?.st.clone();

        Ok(result)
    }

    pub fn eval_string_template<S: Into<String>>(&self, raw: S) -> Result<String, Box<dyn Error>> {
        let lua = Lua::new();

        let globals = lua.globals();

        for (key, value) in &self.values {
            let lua_value = lua.to_value(value)?;
            globals.set(key.as_str(), lua_value)?;
        }

        let script = raw.into();

        let res = lua.load(script.as_str()).eval::<String>()?;
        Ok(res)
    }
}

impl RendererBuilder {
    pub fn build_lua(&self) -> Result<LuaRenderer, Box<dyn Error>> {
        let mut templates: BTreeMap<String, String> = BTreeMap::new();

        for (key, value) in &self.raw_templates {
            templates.insert(key.clone(), value.clone());
        }

        for (key, path) in &self.file_templates {
            let raw = fs::read_to_string(path)?;
            templates.insert(key.clone(), raw);
        }

        let mut values: BTreeMap<String, Value> = BTreeMap::new();

        for (key, value) in &self.values {
            values.insert(key.clone(), value.clone());
        }

        let renderer = LuaRenderer {
            values,
            templates,
        };

        Ok(renderer)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::renderer::builder::RendererBuilder;
    use crate::renderer::luau_renderer::LuaRenderer;

    #[test]
    fn luau_renderer_render_test() -> Result<(), Box<dyn Error>> {
        let builder = RendererBuilder::new()
            .with_value("foo", "one")
            .with_value("bar", "two")
            .with_value("baz", "3")
            .with_raw_template(
                "SAMPLE.txt",
                "template:print(`foo: one = {foo}, bar: two = {bar}, baz: 3 = {baz}`)");

        let renderer: LuaRenderer = builder.build_lua()?;
        let result = renderer.render("SAMPLE.txt")?;
        assert_eq!(result, "foo: one = one, bar: two = two, baz: 3 = 3");

        Ok(())
    }

    #[test]
    fn luau_renderer_eval_test() -> Result<(), Box<dyn Error>> {
        let builder = RendererBuilder::new()
            .with_value("foo", "one")
            .with_value("bar", "two")
            .with_value("baz", "3");

        let renderer: LuaRenderer = builder.build_lua()?;
        let result: String = renderer.eval_string_template(r#"`{foo}/{bar}/{baz}`"#)?;
        assert_eq!(result, "one/two/3");

        Ok(())
    }
}
