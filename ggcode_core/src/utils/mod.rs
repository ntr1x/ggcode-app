use serde_yaml::Value;

pub mod errors;

pub fn merge_yaml(a: &mut Value, b: Value) {
    match (a, b) {
        (a @ &mut Value::Mapping(_), Value::Mapping(b)) => {
            let a = a.as_mapping_mut().unwrap();
            for (k, v) in b {
                /*if v.is_sequence() && a.contains_key(&k) && a[&k].is_sequence() {
                    let mut _b = a.get(&k).unwrap().as_sequence().unwrap().to_owned();
                    _b.append(&mut v.as_sequence().unwrap().to_owned());
                    a[&k] = Value::from(_b);
                    continue;
                }*/
                if !a.contains_key(&k) {
                    a.insert(k.to_owned(), v.to_owned());
                } else {
                    merge_yaml(&mut a[&k], v);
                }
            }
        }
        (a, b) => *a = b,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::error::Error;
    use std::io::{BufRead, BufReader};
    use std::sync::mpsc::sync_channel;
    use std::thread;

    use run_script::ScriptOptions;

    #[test]
    fn test_command() -> Result<(), Box<dyn Error>> {

        Ok(())
    }

    #[test]
    fn read_write_ascii_control_chars() -> Result<(), Box<dyn Error>> {
        println!("\\u001b[32mdev@pc\\u001b[00m:\\u001b[34m~/my-application\\u001b[00m$ ");
        println!("\x1b[32mdev@pc\x1b[00m:\x1b[34m~/my-application\x1b[00m$ ");
        Ok(())
    }

    #[test]
    fn spawn_script_test() -> Result<(), Box<dyn Error>>{
        let mut env_vars = HashMap::new();
        env_vars.insert("CLICOLOR_FORCE".to_string(), "1".to_string());
        env_vars.insert("CLICOLOR".to_string(), "1".to_string());
        env_vars.insert("COLORTERM".to_string(), "truecolor".to_string());
        env_vars.insert("TERM".to_string(), "xterm-256color".to_string());

        let mut options = ScriptOptions::new();
        options.env_vars = Some(env_vars);

        let script_string = r#"
            cd ~
            pwd
            ls -la --color
        "#;

        let mut script = run_script::spawn_script!(script_string, options)?;

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

        while let Ok(line) = receiver.recv() {
            println!("{}", line);
        }

        stderr_thread.join().unwrap();
        stdout_thread.join().unwrap();

        script.wait().unwrap();

        Ok(())
    }
}
