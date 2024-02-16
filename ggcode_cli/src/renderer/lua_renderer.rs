use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

use mlua::{AnyUserData, Lua, LuaSerdeExt, UserData};
use serde_yaml::Value;

use crate::renderer::builder::RendererBuilder;

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

        let template = lua.create_userdata(Template {
            st: String::new()
        })?;

        globals.set("template", &template)?;

        let script = self.templates
            .get(name_string)
            .ok_or::<Box<dyn Error>>(format!("No template: {}", name_string).into())?;

        lua.load(script).exec()?;

        let result = template.borrow::<Template>()?.st.clone();

        Ok(result)
    }

    pub fn eval_string_template<S: Into<String>>(&self, raw: S) -> Result<String, Box<dyn Error>> {
        let lua = Lua::new();

        let globals = lua.globals();

        for (key, value) in &self.values {
            let lua_value = lua.to_value(value)?;
            globals.set(key.as_str(), lua_value)?;
        }

        let res = lua.load(format!("{}", raw.into().as_str())).eval::<String>()?;
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

#[derive(Debug)]
struct Template {
    pub st: String,
}

impl UserData for Template {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function_mut("print", |_, (ud, value): (AnyUserData, String)| {
            ud.borrow_mut::<Template>()?.st.push_str(value.as_str());
            Ok(())
        });

        methods.add_function_mut("println", |_, (ud, value): (AnyUserData, String)| {
            ud.borrow_mut::<Template>()?.st.push_str(&format!("{}\n", value.as_str()));
            Ok(())
        });
    }
}

#[test]
fn luau_renderer_render_test() -> Result<(), Box<dyn Error>> {
    let builder = LuaRenderer::builder()
        .with_value("foo", "one")
        .with_value("bar", "two")
        .with_value("baz", "3")
        .with_template(
            "SAMPLE.txt",
            "template:print(`foo: one = {foo}, bar: two = {bar}, baz: 3 = {baz}`)");

    let renderer: LuaRenderer = builder.build()?;
    let result = renderer.render("SAMPLE.txt")?;
    assert_eq!(result, "foo: one = one, bar: two = two, baz: 3 = 3");

    Ok(())
}

#[test]
fn luau_renderer_eval_test() -> Result<(), Box<dyn Error>> {
    let builder = LuaRenderer::builder()
        .with_value("foo", "one")
        .with_value("bar", "two")
        .with_value("baz", "3");

    let renderer: LuaRenderer = builder.build()?;
    let result: String = renderer.eval_string_template(r#"`{foo}/{bar}/{baz}`"#)?;
    assert_eq!(result, "one/two/3");

    Ok(())
}
