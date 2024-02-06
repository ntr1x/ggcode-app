use std::error::Error;

use ggcode_core::Config;

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

pub fn load_optional_config(path: String) -> Result<Config, Box<dyn Error>> {
    let f = std::fs::File::open(path)?;
    let config = serde_yaml::from_reader(f)?;
    Ok(config)
}
