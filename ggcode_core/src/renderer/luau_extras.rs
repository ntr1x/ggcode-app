use std::{cmp, thread};
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;

use console::{Color, style};
use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataMethods};
use mlua::Error::RuntimeError;
use run_script::ScriptOptions;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::generator::DefaultGenerator;
use crate::ResolvedContext;
use crate::storage::resolve_target;
use crate::types::AppResult;
use crate::utils::errors::{describe_error, ErrorDescription};

#[derive(Debug)]
pub struct LuauTemplate {
    pub st: String,
}

impl UserData for LuauTemplate {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function_mut("print", |_, (ud, value): (AnyUserData, String)| {
            ud.borrow_mut::<LuauTemplate>()?.st.push_str(value.as_str());
            Ok(())
        });

        methods.add_function_mut("println", |_, (ud, value): (AnyUserData, String)| {
            ud.borrow_mut::<LuauTemplate>()?.st.push_str(&format!("{}\n", value.as_str()));
            Ok(())
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub struct LuauShell;

impl LuauShell {
    fn exec(workdir: &String, command: &String) -> AppResult<String> {
        let mut env_vars = HashMap::new();
        env_vars.insert("CLICOLOR_FORCE".to_string(), "1".to_string());
        env_vars.insert("CLICOLOR".to_string(), "1".to_string());
        env_vars.insert("COLORTERM".to_string(), "truecolor".to_string());
        env_vars.insert("TERM".to_string(), "xterm-256color".to_string());

        let mut options = ScriptOptions::new();
        options.working_directory = Some(PathBuf::from(workdir).canonicalize()?);
        options.env_vars = Some(env_vars);

        let mut script = run_script::spawn_script!(command, options)?;

        let stdout = script.stdout.take().unwrap();
        let stderr = script.stderr.take().unwrap();

        let (sender, receiver) = sync_channel(1);
        let stdout_sender = sender.clone();
        let stderr_sender = sender.clone();
        drop(sender);

        let stdout_thread = thread::spawn(move || {
            let stdout_buf = BufReader::new(stdout);
            for line in stdout_buf.lines() {
                stdout_sender.send(line.unwrap()).unwrap();
            }
            drop(stdout_sender);
        });

        let stderr_thread = thread::spawn(move || {
            let stderr_buf = BufReader::new(stderr);
            for line in stderr_buf.lines() {
                stderr_sender.send(line.unwrap()).unwrap();
            }
            drop(stderr_sender);
        });

        let mut output = String::new();
        while let Ok(line) = receiver.recv() {
            output.push_str(line.as_str());
            output.push('\n');
        }

        stderr_thread.join().unwrap();
        stdout_thread.join().unwrap();

        script.wait().unwrap();

        return Ok(output)
    }
}

impl UserData for LuauShell {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("exec", |_, (_ud, path, command): (AnyUserData, String, String)| {
            match Self::exec(&path, &command) {
                Ok(stdout_string) => {
                    let encoded = stdout_string
                        .chars()
                        .collect::<Vec<char>>()
                        .iter()
                        .flat_map(|ch| match (ch, ch.is_control()) {
                            ('\n', _) | (_, false) => {
                                vec![ch.clone()]
                            }
                            ('\t', _) => {
                                "\\t".chars().collect::<Vec<char>>()
                            }
                            (_, true) => {
                                let mut b = [0; 1];
                                ch.encode_utf16(&mut b);
                                format!("\\u{:04x?}", &b[0]).chars().collect::<Vec<char>>()
                            }
                        })
                        .collect::<Vec<char>>()
                        .into_iter()
                        .collect::<String>();

                    Ok(encoded)
                },
                Err(e) => Err(RuntimeError(format!("Cannot evaluate script. {}", e).to_string()))
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct GenerationTarget {
    target_name: Option<String>,
    target_path: Option<String>,
    dry_run: Option<bool>,
}

impl UserData for GenerationTarget {}

#[derive(Clone)]
pub struct LuauEngine {
    pub context: ResolvedContext,
    pub generator: DefaultGenerator,
}

impl LuauEngine {
    pub fn generate(&self, scroll_name: &String, target: &GenerationTarget, overrides: Option<Value>) -> AppResult<()> {
        let resolved_target_path = resolve_target(
            &self.context,
            target.target_name.clone(),
            target.target_path.clone())?;

        self.generator.generate(
            scroll_name,
            &resolved_target_path,
            target.dry_run.unwrap_or(false),
            overrides)?;

        Ok(())
    }
}

impl UserData for LuauEngine {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("generate", |lua, (ud, scroll, target, variables): (AnyUserData, String, mlua::Value, mlua::Value)| {
            let variables_yaml: Option<Value> = lua.from_value(variables).ok();
            let target_object: GenerationTarget = lua.from_value(target)?;

            let engine = ud.borrow::<LuauEngine>()?;

            match engine.generate(&scroll, &target_object, variables_yaml) {
                Ok(()) => {},
                Err(e) => return Err(RuntimeError(format!("Cannot generate using scroll: {}. {}", scroll, e).to_string())),
            };
            Ok(())
        });
    }
}

pub fn trace_mlua_error(script: &String, e: &Box<dyn Error>) {
    let description = describe_error(&e);
    match description {
        Ok(Some(ErrorDescription::SourceError(data))) => {
            let vec: Vec<&str> = script.lines().collect();
            let mut area_vec = vec![];
            let lower = cmp::max(0, data.line as i32 - 1) as usize;
            let upper = cmp::min(data.line + 1, vec.len() - 1) + 1;
            for i in lower..upper {
                let row = format!("{: >6} │ {}", format!("L{}", i + 1), vec[i]);
                let styled = match i == data.line {
                    true => format!("{: <80}", style(row).white().bg(Color::Color256(52))),
                    false => format!("{: <80}", style(row).white().bg(Color::Color256(17))),
                };
                area_vec.push(styled.to_string());
            }
            let area_st = area_vec.join("\n");
            eprintln!("\n{}\n{}\n", style(data.details).bold(), area_st)
        }
        _ => {}
    }
}
