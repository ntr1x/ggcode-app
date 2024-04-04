use std::error::Error;

use tera::{Context, Tera};

use crate::renderer::builder::RendererBuilder;
use crate::renderer::tera_extras::format_ansi;
use crate::renderer::tera_functions::uuid_v4;

#[derive(Debug)]
pub struct TeraRenderer {
    context: Context,
    tera: Tera,
}

impl TeraRenderer {
    pub fn render<N: Into<String>>(&self, name: N) -> Result<String, Box<dyn Error>> {
        match self.tera.render(name.into().as_str(), &self.context) {
            Ok(string) => Ok(string),
            Err(e) => {
                return match e.source() {
                    Some(e1) => Err(format!("{}. {}", e, e1).into()),
                    None => Err(format!("{}", e).into()),
                }
            }
        }
    }
}

impl RendererBuilder {

    pub fn build_tera(&self) -> Result<TeraRenderer, Box<dyn Error>> {
        let mut context = Context::new();
        let mut tera = Tera::default();

        tera.register_filter("format_ansi", format_ansi);

        for (key, value) in &self.values {
            context.insert(key, &value.clone());
        }

        for (name, raw) in &self.raw_templates {
            tera.add_raw_template(&name, &raw)?;
        }

        for (name, path) in &self.file_templates {
            tera.add_template_file(path, Some(&name))?;
        }

        tera.register_function("uuid_v4", uuid_v4);

        let renderer = TeraRenderer {
            context,
            tera
        };

        Ok(renderer)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::renderer::builder::RendererBuilder;
    use crate::renderer::tera_renderer::TeraRenderer;

    #[test]
    fn tera_renderer_test() -> Result<(), Box<dyn Error>> {
        let builder = RendererBuilder::new()
            .with_value("foo", "one")
            .with_value("bar", "two")
            .with_value("baz", "3")
            .with_raw_template(
                "SAMPLE.txt",
                "foo: one = {{foo}}, bar: two = {{bar}}, baz: 3 = {{baz}}");

        let renderer: TeraRenderer = builder.build_tera()?;
        let result = renderer.render("SAMPLE.txt")?;
        assert_eq!(result, "foo: one = one, bar: two = two, baz: 3 = 3");

        Ok(())
    }
}