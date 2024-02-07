use std::env;
use std::error::Error;
use std::io::Write;
use relative_path::{RelativePath, RelativePathBuf};

use ggcode_core::config::Config;
use ggcode_core::scroll::Scroll;

pub fn resolve_inner_path(path: &String) -> Result<RelativePathBuf, Box<dyn Error>> {
    let relative_path = RelativePath::new(path).normalize();
    assert!(!relative_path.starts_with(".."), "Could not leave base directory");
    Ok(relative_path)
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
