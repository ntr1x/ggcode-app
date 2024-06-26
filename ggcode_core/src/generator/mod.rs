use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use console::style;
use relative_path::RelativePathBuf;
use serde_yaml::Value;

use crate::generator::GeneratorEvent::{Finish, Start};
use crate::renderer::builder::RendererBuilder;
use crate::ResolvedContext;
use crate::scroll::find_scroll_by_full_name;
use crate::storage::{load_templates, load_variables, resolve_inner_path, resolve_search_locations, save_target_file};
use crate::types::AppResult;
use crate::utils::merge_yaml;

#[derive(Clone)]
pub struct DefaultGenerator {
    pub context: ResolvedContext,
    pub wrapped_observers: Vec<Arc<Mutex<dyn GeneratorObserver>>>,
}


pub trait GeneratorObserver {
    fn on_notify(&mut self, event: &GeneratorEvent);
}

pub enum GeneratorEvent {
    Start(String),
    Message(String),
    Finish(String)
}

impl DefaultGenerator {
    pub fn add_observer(&mut self, observer: Arc<Mutex<dyn GeneratorObserver>>) {
        self.wrapped_observers.push(observer);
    }

    pub fn notify(&self, event: GeneratorEvent) {
        for wrapped_observer in self.wrapped_observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            observer.on_notify(&event);
        }
    }

    pub fn generate(&self, scroll_name: &String, target_path: &PathBuf, dry_run: bool, overrides: Option<Value>) -> AppResult<()> {
        let scroll = find_scroll_by_full_name(&self.context, scroll_name)?;

        let path = match scroll.dependency_name {
            None => resolve_inner_path(&scroll.scroll.path)?,
            Some(n) => RelativePathBuf::from("ggcode_modules").join(&n).join(&scroll.scroll.path)
        };

        // println!("Scroll name {} -> {} -> {}", scroll_name, &target_path.as_path().to_str().unwrap().to_string(), path.to_string());

        let values_directory_path = path.join("variables");
        let search_locations = resolve_search_locations(&self.context.current_config);

        let mut variables = load_variables(&values_directory_path, &search_locations)?;

        if let Some(o) = overrides {
            merge_yaml(&mut variables, o);
        }

        let mut builder = RendererBuilder::new();

        let variables_mapping = match variables.as_mapping() {
            Some(m) => m,
            None => return Err(format!("Cannot generate content using {} scroll. Invalid variables.", style(scroll_name).yellow()).into())
        };

        for (key, value) in variables_mapping {
            builder = builder.with_value(key.as_str().unwrap().to_string(), value);
        }

        // if let Some(path) = &overrides_path {
        //     if path.is_directory() {
        //         load_variables(path)
        //     }
        // }

        let templates_directory_path = path.join("templates");
        let templates = load_templates(&templates_directory_path);

        for (key, value) in &templates {
            builder = builder.with_file_template(key, value);
        }

        let tera = &builder.build_tera()?;
        let lua = &builder.build_lua()?;
        let noop = &builder.build_noop()?;

        for (key, _value) in &templates {
            let message = format!("Rendering {} template...", style(key).yellow());
            self.notify(Start(message));
            let file_path = lua.eval_string_template(format!("`{}`", key))?;
            let file_relative_path = RelativePathBuf::from(&file_path);

            let (target_file_relative_path, file_content) = match &file_relative_path.extension() {
                Some("tera") => (
                    file_relative_path.with_extension("".to_string()),
                    tera.render(key)?
                ),
                Some("luau") => (
                    file_relative_path.with_extension("".to_string()),
                    lua.render(key)?
                ),
                _ => (
                    file_relative_path.clone(),
                    noop.render(key)?
                )
            };

            match dry_run {
                true => {
                    let message = format!(
                        "{} Rendered template: {}",
                        style("[DONE]").green(),
                        &file_path);
                    self.notify(Finish(message.to_string()));
                }
                false => {
                    let file_path = target_file_relative_path.to_path(&target_path);
                    let message = format!(
                        "{} Generated file: {}",
                        style("[DONE]").green(),
                        file_path.to_str().unwrap().to_string());
                    self.notify(Finish(message));
                    let file_name = file_path.file_name().unwrap().to_str().unwrap();

                    if file_name.starts_with("!") {
                        // ignore
                    } else if file_name.starts_with("+") {
                        let alternate_name = file_name.strip_prefix("+").unwrap().to_string();
                        let alternate_path = target_file_relative_path.with_file_name(alternate_name);
                        save_target_file(&target_path, &alternate_path, &file_content, false)?;
                    } else {
                        save_target_file(&target_path, &target_file_relative_path, &file_content, true)?;
                    }
                }
            }
        }

        Ok(())
    }
}