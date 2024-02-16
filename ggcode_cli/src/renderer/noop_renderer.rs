use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

use serde_yaml::Value;

use crate::renderer::builder::RendererBuilder;

#[derive(Debug)]
pub struct NoopRenderer {
    values: BTreeMap<String, Value>,
    templates: BTreeMap<String, String>,
}

impl NoopRenderer {
    pub fn render<N: Into<String>>(&self, name: N) -> Result<String, Box<dyn Error>> {
        let name_string = &name.into();
        let template = self.templates
            .get(name_string)
            .ok_or::<Box<dyn Error>>(format!("No template: {}", name_string).into())?;
        Ok(template.clone())
    }
}

impl RendererBuilder {
    pub fn build_noop(&self) -> Result<NoopRenderer, Box<dyn Error>> {
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

        let renderer = NoopRenderer {
            values,
            templates,
        };

        Ok(renderer)
    }
}

#[test]
fn noop_renderer_render_test() -> Result<(), Box<dyn Error>> {
    let builder = NoopRenderer::builder()
        .with_value("foo", "one")
        .with_value("bar", "two")
        .with_value("baz", "3")
        .with_template(
            "NOOP.txt",
            "Some template content");

    let renderer: NoopRenderer = builder.build()?;
    let result = renderer.render("SAMPLE.txt")?;
    assert_eq!(result, "Some template content");

    Ok(())
}
