use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use serde::Serialize;
use tera::{Context, Tera, to_value, Value};

#[derive(Debug)]
pub struct Renderer {
    context: Context,
    tera: Tera,
}

impl Renderer {
    pub fn builder() -> RendererBuilder {
        RendererBuilder::new()
    }

    pub fn render<N: Into<String>>(&self, name: N) -> Result<String, Box<dyn Error>> {
        match self.tera.render(name.into().as_str(), &self.context) {
            Ok(string) => Ok(string),
            Err(e) => {
                println!("{}", e.source().unwrap().to_string());
                panic!("{}", e)
            }
        }
    }
}

#[derive(Default)]
pub struct RendererBuilder {
    values: BTreeMap<String, Value>,
    template_strings: BTreeMap<String, String>,
    template_files: BTreeMap<String, PathBuf>,
}

impl RendererBuilder {
    pub fn new() -> RendererBuilder {
        RendererBuilder::default()
    }

    pub fn with_value<T: Serialize + ?Sized, S: Into<String>>(mut self, key: S, value: &T) -> RendererBuilder {
        self.values.insert(key.into(), to_value(value).unwrap());
        self
    }

    pub fn with_template_string<S: Into<String>, V: Into<String>>(mut self, name: S, raw: V) -> RendererBuilder {
        self.template_strings.insert(name.into(), raw.into());
        self
    }

    pub fn with_template_file<S: Into<String>, V: Into<PathBuf>>(mut self, name: S, path: V) -> RendererBuilder {
        self.template_files.insert(name.into(), path.into());
        self
    }

    pub fn build(self) -> Result<Renderer, Box<dyn Error>> {
        let mut context = Context::new();
        let mut tera = Tera::default();

        for (key, value) in &self.values {
            context.insert(key, &value);
        }

        for (name, raw) in self.template_strings {
            tera.add_raw_template(&format!("raw:{}", name), &raw)?;
        }

        for (name, path) in self.template_files {
            tera.add_template_file(path, Some(&format!("path:{}", name)))?;
        }

        let renderer = Renderer {
            context,
            tera
        };

        Ok(renderer)
    }
}

#[test]
fn builder_test() -> Result<(), Box<dyn Error>> {
    let builder = Renderer::builder()
        .with_value("foo", "one")
        .with_value("bar", "two")
        .with_value("baz", "3")
        .with_template_string(
            "SAMPLE.txt",
            "foo: one = {{foo}}, bar: two = {{bar}}, baz: 3 = {{baz}}");

    let renderer: Renderer = builder.build()?;
    let result = renderer.render("raw:SAMPLE.txt")?;
    assert_eq!(result, "foo: one = one, bar: two = two, baz: 3 = 3");

    Ok(())
}
