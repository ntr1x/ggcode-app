use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use console::style;
use glob::glob;
use relative_path::{RelativePath, RelativePathBuf};
use serde_yaml::{Mapping, Value};

use ggcode_core::config::Config;
use ggcode_core::scroll::Scroll;

use crate::utils::merge_yaml;

pub fn resolve_target_path(path: &String) -> Result<PathBuf, Box<dyn Error>> {
    let relative_path = RelativePath::new(path).normalize();
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let target_path = relative_path.to_path(&current_dir);
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

pub fn save_config(relative_path: &RelativePathBuf, config: Config) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open config file");

    serde_yaml::to_writer(f, &config).unwrap();

    Ok(())
}

pub fn rm_scroll(relative_path: &RelativePathBuf) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    std::fs::remove_dir_all(path).unwrap();
    Ok(())
}

pub fn save_scroll(relative_path: &RelativePathBuf, scroll: Scroll) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let f = std::fs::OpenOptions::new()
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
    std::fs::create_dir_all(prefix).unwrap();

    let mut f = std::fs::OpenOptions::new()
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
    std::fs::create_dir_all(prefix).unwrap();

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open target file");

    f.write_all(content.as_bytes()).unwrap();
    Ok(())
}

pub fn load_config(relative_path: &RelativePathBuf) -> Result<Config, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_scroll(relative_path: &RelativePathBuf) -> Result<Scroll, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_value(relative_path: &RelativePathBuf) -> Result<Value, Box<dyn Error>> {
    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let path = relative_path.to_path(current_dir);

    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_templates(templates_directory_path: RelativePathBuf) -> BTreeMap<String, PathBuf> {
    let pattern = format!("{}/**/*", templates_directory_path);

    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();

    let mut map: BTreeMap<String, PathBuf> = BTreeMap::new();
    if let Ok(paths) = glob(pattern.as_str()) {
        for entry in paths {
            if let Ok(entry_path) = entry {
                if entry_path.is_file() {
                    let relative_entry_path = RelativePathBuf::from_path(entry_path).unwrap();
                    let relative_templates_path = templates_directory_path.relative(&relative_entry_path);
                    map.insert(relative_templates_path.to_string(), relative_entry_path.to_path(&current_dir));
                }
            }
        }
    }

    map
}

pub fn load_variables(values_directory_path: &RelativePathBuf) -> Value {
    let pattern = format!("{}/**/*.yaml", values_directory_path);

    let mut merged_value: Value = Value::Mapping(Mapping::new());

    if let Ok(paths) = glob(pattern.as_str()) {
        for entry in paths {
            if let Ok(entry_path) = entry {
                if entry_path.is_file() {
                    let relative_entry_path = RelativePathBuf::from_path(entry_path).unwrap();
                    let relative_variables_path = values_directory_path.relative(&relative_entry_path);

                    if let Ok(value) = &load_value(&relative_entry_path) {
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
    merged_value
}
