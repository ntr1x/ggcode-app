use std::error::Error;

use clap::ArgMatches;
use console::style;
use derive_builder::Builder;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;

use ggcode_core::types::AppResult;

#[derive(Builder)]
#[builder(setter(into))]
pub struct TerminalFlag<'a> {
    matches: &'a ArgMatches,
    name: String,
    prompt: String,
    #[builder(default = "self.default_default_value()")]
    default_value: Option<bool>,
    required: bool,
}

impl <'a> TerminalFlagBuilder<'a> {
    pub fn default_default_value(&self) -> Option<bool> {
        None
    }
}

impl <'a> TerminalFlag<'a> {
    pub fn builder() -> TerminalFlagBuilder<'a> {
        TerminalFlagBuilder::default()
    }

    pub fn read_bool(&self) -> Result<Option<bool>, Box<dyn Error>> {
        self.read(|s| Ok(s.clone()))
    }

    pub fn read<T, F: Fn(bool) -> AppResult<T>>(&self, convert: F) -> AppResult<Option<T>> {
        let input = self.matches.get_one::<bool>(&self.name.as_str());

        loop {
            let option = match (input, &self.required) {
                (Some(path), _) => Some(path.clone()),
                (None, false) => None,
                (None, true) => {
                    let theme = ColorfulTheme::default();
                    let input = match &self.default_value {
                        Some(dv) => Confirm::with_theme(&theme)
                            .with_prompt(&self.prompt)
                            .default(dv.clone()),
                        None => Confirm::with_theme(&theme)
                            .with_prompt(&self.prompt)
                    };
                    Some(input.interact()?)
                }
            };

            match (option, &self.required) {
                (Some(source), _) => {
                    match convert(source) {
                        Ok(target) => return Ok(Some(target)),
                        Err(e) => {
                            match input {
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
