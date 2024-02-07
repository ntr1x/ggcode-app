use std::env;
use std::error::Error;
use std::io::Write;
use std::path::Path;

use ggcode_core::config::Config;
use ggcode_core::scroll::Scroll;

pub fn save_config(path: &String, config: Config) -> Result<(), Box<dyn Error>> {
    let f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("Couldn't open config file");

    serde_yaml::to_writer(f, &config).unwrap();

    Ok(())
}

pub fn rm_scroll(path: &String) -> Result<(), Box<dyn Error>> {
    let p = Path::new(path);

    assert!(!path.starts_with(".."), "The \"scroll\" directory should be nested within the project directory.");
    assert!(p.is_relative(), "The \"scroll\" directory should be defined relative to the project directory.");

    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();
    let scroll_path = current_dir.join(path).canonicalize().unwrap();

    assert!(scroll_path.starts_with(current_dir), "The \"scroll\" directory should be nested within the project directory.");

    std::fs::remove_dir_all(scroll_path).unwrap();

    Ok(())
}

pub fn save_scroll(path: &String, scroll: Scroll) -> Result<(), Box<dyn Error>> {
    let p = Path::new(path);

    assert!(!path.starts_with(".."), "The \"scroll\" directory should be nested within the project directory.");
    assert!(p.is_relative(), "The \"scroll\" directory should be defined relative to the project directory.");

    let current_dir = env::current_dir().unwrap().canonicalize().unwrap();

    let p = current_dir.join(path);

    let prefix = p.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(p)
        .expect("Couldn't open scroll file");

    serde_yaml::to_writer(f, &scroll).unwrap();

    Ok(())
}

pub fn save_string(path: &String, content: String) -> Result<(), Box<dyn Error>> {
    let p = Path::new(path);

    let prefix = p.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(p)
        .expect("Couldn't open scroll file");

    f.write_all(content.as_bytes()).unwrap();
    Ok(())
}


pub fn load_config(path: String) -> Result<Config, Box<dyn Error>> {
    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}

pub fn load_scroll(path: String) -> Result<Scroll, Box<dyn Error>> {
    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}
