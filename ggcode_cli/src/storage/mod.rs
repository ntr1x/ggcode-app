use std::{env, fs};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use console::style;
use glob::glob;
use relative_path::{RelativePath, RelativePathBuf};
use serde_yaml::{Mapping, Value};

use ggcode_core::config::{DEFAULT_CONFIG_NAME, PackageConfig, PackageData};
use ggcode_core::scroll::ScrollConfig;

use crate::renderer::luau_evaluator::LuauEvaluatorBuilder;
use crate::renderer::luau_extras::LuauShell;
use crate::types::AppResult;
use crate::utils::merge_yaml;

pub fn resolve_target_path(path: &String) -> Result<PathBuf, Box<dyn Error>> {
    let path_buf = PathBuf::from(path.to_string());
    let target_path = match path_buf.is_absolute() {
        true => {
            let target_path = path_buf
                .canonicalize()
                .map_err::<Box<dyn Error>, _>(|e| format!("Invalid path. {}", e).into())?;
            target_path
        },
        false => {
            let relative_path = RelativePath::new(path).normalize();
            let target_path = PathBuf::from(relative_path.to_string());
            target_path

        }
    };
    Ok(target_path)
}

pub fn resolve_package_path(path: &String) -> Result<RelativePathBuf, Box<dyn Error>> {
    let relative_path = RelativePath::new(path).normalize();
    if relative_path.starts_with("@") {
        Ok(RelativePath::new("@").relative(relative_path))
    } else {
        Ok(RelativePath::new("ggcode_modules").join(relative_path))
    }
}

pub fn resolve_inner_path(path: &String) -> Result<RelativePathBuf, Box<dyn Error>> {
    let relative_path = RelativePath::new(path).normalize();
    if relative_path.starts_with("..") {
        return Err(format!("Invalid path: {}. Could not leave base directory.", style(relative_path).yellow()).into());
    }
    let normalized_path = relative_path.as_str();
    match normalized_path {
        "" => return Err("Invalid path. Path should not be empty.".into()),
        "." => return Err(format!("Invalid path: {}. Path should not point to the project directory.", style(normalized_path).yellow()).into()),
        _ => Ok(relative_path)
    }
}

fn resolve_dependencies(config: &PackageConfig) -> AppResult<BTreeMap<String, PackageConfig>> {
    let mut dependencies: BTreeMap<String, PackageConfig> = BTreeMap::new();
    for dependency in &config.repositories {
        let relative_path = RelativePath::new("ggcode_modules")
            .join(&dependency.name)
            .join(&DEFAULT_CONFIG_NAME);
        let dependency_config = load_config(&relative_path)?;
        dependencies.insert(dependency.name.clone(), dependency_config);
    }
    Ok(dependencies)
}

pub fn resolve_search_locations(config: &PackageConfig) -> Vec<RelativePathBuf> {
    let mut locations: Vec<RelativePathBuf> = vec![];
    let path = RelativePathBuf::from("lib")
        .join("?.luau");
    locations.push(path);
    for dependency in &config.repositories {
        let path = RelativePathBuf::from("ggcode_modules")
            .join(&dependency.name)
            .join("lib")
            .join("?.luau");
        locations.push(path);
    }
    locations
}

pub fn resolve_package(config: &PackageConfig) -> AppResult<PackageData> {
    let dependencies = resolve_dependencies(config)?;
    let data = PackageData {
        config: config.clone(),
        dependencies,
    };
    Ok(data)
}

pub fn save_config(relative_path: &RelativePathBuf, config: PackageConfig) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open storage file");

    serde_yaml::to_writer(f, &config).unwrap();

    Ok(())
}

pub fn rm_scroll(relative_path: &RelativePathBuf) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);
    fs::remove_dir_all(path)?;
    Ok(())
}

pub fn save_scroll(relative_path: &RelativePathBuf, scroll: ScrollConfig) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let prefix = path.parent().unwrap();
    fs::create_dir_all(prefix).unwrap();

    let f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open scroll file");

    serde_yaml::to_writer(f, &scroll).unwrap();

    Ok(())
}

pub fn save_string(relative_path: &RelativePathBuf, content: String) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let prefix = path.parent().unwrap();
    fs::create_dir_all(prefix).unwrap();

    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open scroll file");

    f.write_all(content.as_bytes()).unwrap();
    Ok(())
}

pub fn save_target_file(target_dir: &PathBuf, relative_path: &RelativePathBuf, content: &String) -> Result<(), Box<dyn Error>> {
    let path = relative_path.to_path(target_dir);

    let prefix = path.parent().unwrap();
    fs::create_dir_all(prefix).unwrap();

    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open target file");

    f.write_all(content.as_bytes()).unwrap();
    Ok(())
}

pub fn load_config(relative_path: &RelativePathBuf) -> Result<PackageConfig, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_scroll(relative_path: &RelativePathBuf) -> Result<ScrollConfig, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_yaml(relative_path: &RelativePathBuf) -> Result<Value, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_luau(relative_path: &RelativePathBuf, search_locations: &Vec<RelativePathBuf>) -> Result<Value, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(&current_dir);

    let source = fs::read_to_string(path)?;

    let mut builder = LuauEvaluatorBuilder::new()
        .enable_shell(LuauShell);

    for rp in search_locations {
        builder = builder.with_path_entry(&rp.to_path(&current_dir));
    }

    let evaluator = builder.build()?;

    let config = evaluator.eval_value(&source)?;

    Ok(config)
}

pub fn load_variables(values_directory_path: &RelativePathBuf, search_locations: &Vec<RelativePathBuf>) -> Result<Value, Box<dyn Error>> {
    let pattern = format!("{}/**/*", values_directory_path);

    let mut merged_value: Value = Value::Mapping(Mapping::new());

    if let Ok(paths) = glob(pattern.as_str()) {
        for entry in paths {
            if let Ok(entry_path) = entry {
                if entry_path.is_file() {
                    let relative_entry_path = RelativePathBuf::from_path(entry_path).unwrap();
                    let relative_variables_path = values_directory_path.relative(&relative_entry_path);

                    let config = match relative_entry_path.extension() {
                        Some("yaml") => Some(load_yaml(&relative_entry_path)?),
                        Some("luau") => Some(load_luau(&relative_entry_path, search_locations)?),
                        _ => None
                    };

                    if let Some(value) = &config {
                        let file_stem = relative_variables_path.file_stem().unwrap();
                        let parent = relative_variables_path.parent().unwrap();

                        let mut proto = Value::Mapping(Mapping::new());
                        let mut cursor = &mut proto;
                        for component in parent.components() {
                            let mapping = cursor.as_mapping_mut().unwrap();
                            let nested = Value::Mapping(Mapping::new());
                            mapping.insert(component.as_str().into(), nested.into());
                            cursor = mapping.get_mut::<String>(component.as_str().into()).unwrap();
                        }

                        cursor.as_mapping_mut().unwrap().insert(file_stem.into(), value.clone());

                        merge_yaml(&mut merged_value, proto);
                    }
                }
            }
        }
    }
    Ok(merged_value)
}

pub fn load_templates(templates_directory_path: &RelativePathBuf) -> BTreeMap<String, PathBuf> {
    let pattern = format!("{}/**/*", templates_directory_path);
    return load_glob(&pattern, templates_directory_path);
}

pub fn load_glob(pattern: &String, scripts_directory_path: &RelativePathBuf) -> BTreeMap<String, PathBuf> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let mut map: BTreeMap<String, PathBuf> = BTreeMap::new();
    if let Ok(paths) = glob(pattern.as_str()) {
        for entry in paths {
            if let Ok(entry_path) = entry {
                if entry_path.is_file() {
                    let relative_entry_path = RelativePathBuf::from_path(entry_path).unwrap();
                    let relative_templates_path = scripts_directory_path.relative(&relative_entry_path);
                    map.insert(relative_templates_path.to_string(), relative_entry_path.to_path(&current_dir));
                }
            }
        }
    }
    return map
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use indoc::indoc;
    use mlua::{Lua, LuaSerdeExt};
    use serde_yaml::Value;

    use crate::storage::resolve_target_path;

    #[test]
    fn resolve_target_path_test() -> Result<(), Box<dyn Error>> {
        let path = resolve_target_path(&".".to_string())?;
        assert_eq!(path.to_str().unwrap(), "");

        let path = resolve_target_path(&"/".to_string())?;
        assert_eq!(path.to_str().unwrap(), "/");

        let path = resolve_target_path(&"../some".to_string())?;
        assert_eq!(path.to_str().unwrap(), "../some");

        let path = resolve_target_path(&"../some/".to_string())?;
        assert_eq!(path.to_str().unwrap(), "../some");

        let path = resolve_target_path(&"./some/inner".to_string())?;
        assert_eq!(path.to_str().unwrap(), "some/inner");

        let path = resolve_target_path(&"some/../other/inner".to_string())?;
        assert_eq!(path.to_str().unwrap(), "other/inner");

        Ok(())
    }

    #[test]
    fn load_lua_config_test() -> Result<(), Box<dyn Error>> {
        let lua = Lua::new();
        let script = indoc! {"
            local name = \"John Smith\"\
            return { name = name, greeting = `Hello {name}!` }
        "};
        let config_lua: mlua::Value = lua.load(script).eval::<mlua::Value>()?;
        let config: Value = lua.from_value(config_lua)?;
        assert_eq!(config["greeting"].as_str().unwrap(), "Hello John Smith!");
        Ok(())
    }
}
