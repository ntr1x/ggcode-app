use std::error::Error;

use clap::ArgMatches;
use console::style;
use derive_builder::Builder;
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;

use ggcode_core::types::AppResult;

#[derive(Builder)]
#[builder(setter(into))]
pub struct TerminalInput<'a> {
    matches: &'a ArgMatches,
    name: String,
    prompt: String,
    #[builder(default = "self.default_default_value()")]
    default_value: Option<String>,
    required: bool,
}

impl <'a> TerminalInputBuilder<'a> {
    pub fn default_default_value(&self) -> Option<String> {
        None
    }
}

impl <'a> TerminalInput<'a> {
    pub fn builder() -> TerminalInputBuilder<'a> {
        TerminalInputBuilder::default()
    }

    pub fn read_string(&self) -> Result<Option<String>, Box<dyn Error>> {
        self.read(|s| Ok(s.clone()))
    }

    pub fn read<T, F: Fn(&String) -> Result<T, Box<dyn Error>>>(&self, convert: F) -> AppResult<Option<T>> {
        let path_input = self.matches.get_one::<String>(&self.name.as_str());

        loop {
            let path_option = match (path_input, &self.required) {
                (Some(path), _) => Some(path.clone()),
                (None, false) => None,
                (None, true) => {
                    let theme = ColorfulTheme::default();
                    let input = match &self.default_value {
                        Some(dv) => Input::with_theme(&theme)
                            .with_prompt(&self.prompt)
                            .default(dv.clone()),
                        None => Input::with_theme(&theme)
                            .with_prompt(&self.prompt)
                    };
                    Some(input.interact_text()?)
                }
            };

            match (path_option, &self.required) {
                (Some(source), _) => {
                    match convert(&source) {
                        Ok(target) => return Ok(Some(target)),
                        Err(e) => {
                            match path_input {
                                Some(_) => return Err(format!("Invalid input. {}", e).into()),
                                None => eprintln!("{} Invalid input. {}", style("[FAILURE]").red(), e)
                            }
                        },
                    }
                },
                (None, true) => eprintln!("{} Invalid input. {} is required", style("[FAILURE]").red(), style(&self.name).yellow()),
                (None, false) => return Ok(None)
            };
        }
    }
}
