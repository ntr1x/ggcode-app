use std::error::Error;
use std::io::Read;

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Sequence, Value};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TransparentSequence {
    pub sequence: Sequence
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MergeSequence {
    pub sequence: Sequence
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TransparentMapping {
    pub mapping: Mapping
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MergeMapping {
    pub mapping: Mapping
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum CustomTag {
    TransparentSequence(TransparentSequence),
    TransparentMapping(TransparentMapping),
    MergeSequence(MergeSequence),
    MergeMapping(MergeMapping),
    Shell(String)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum EvaluationResult {
    Insert(Value),
    Merge(Value),
}

pub fn evaluate(value: &Value) -> Result<Value, Box<dyn Error>> {
    let nested = evaluate_nested(value)?;
    match nested {
        EvaluationResult::Insert(v) => Ok(v),
        EvaluationResult::Merge(v) => Ok(v)
    }
}

pub fn evaluate_nested(value: &Value) -> Result<EvaluationResult, Box<dyn Error>> {
    let result = match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            EvaluationResult::Insert(value.clone())
        }
        Value::Sequence(_) => {
            let mut sequence = Value::Sequence(Sequence::new());
            process_yaml(value, &mut sequence)?;
            EvaluationResult::Insert(sequence)
        }
        Value::Mapping(_) => {
            let mut mapping = Value::Mapping(Mapping::new());
            process_yaml(value, &mut mapping)?;
            EvaluationResult::Insert(mapping)
        }
        Value::Tagged(_) => evaluate_custom(value)?
    };
    Ok(result)
}

fn process_yaml(source: &Value, target: &mut Value) -> Result<(), Box<dyn Error>> {
    match source {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Tagged(_) => {
            unreachable!()
        }
        Value::Sequence(_) => {
            for item in source.as_sequence().unwrap() {
                let er = evaluate_nested(item)?;
                match er {
                    EvaluationResult::Insert(new_value) => {
                        target.as_sequence_mut().unwrap().push(new_value)
                    }
                    EvaluationResult::Merge(mut new_value) => {
                        target.as_sequence_mut().unwrap().append(new_value.as_sequence_mut().unwrap())
                    }
                }
            }
        }
        Value::Mapping(_) => {
            for (key, value) in source.as_mapping().unwrap() {
                let er = evaluate_nested(value)?;

                match er {
                    EvaluationResult::Insert(new_value) => {
                        target.as_mapping_mut().unwrap().insert(key.clone(), new_value);
                    }
                    EvaluationResult::Merge(new_value) => {
                        merge_yaml(target, new_value);
                    }
                };
            }
        }
    };

    Ok(())
}

fn evaluate_custom(value: &Value) -> Result<EvaluationResult, Box<dyn Error>> {
    // let st = serde_yaml::to_string(value).unwrap().to_string();
    let tagged: CustomTag = serde_yaml::from_value(value.clone())?;
    let result = match tagged {
        CustomTag::TransparentSequence(sequence) => EvaluationResult::Insert(Value::Sequence(sequence.sequence)),
        CustomTag::TransparentMapping(mapping) => EvaluationResult::Insert(Value::Mapping(mapping.mapping)),
        CustomTag::MergeSequence(sequence) => EvaluationResult::Merge(Value::Sequence(sequence.sequence)),
        CustomTag::MergeMapping(mapping) => EvaluationResult::Merge(Value::Mapping(mapping.mapping)),
        CustomTag::Shell(st) => evaluate_shell(&st)?
    };
    Ok(result)
}

fn evaluate_shell(st: &String) -> Result<EvaluationResult, Box<dyn Error>> {
    let stdout: Vec<u8> = std::process::Command::new("bash")
        .arg("-c")
        .arg(&st)
        .env("CLICOLOR_FORCE", "1")
        .env("CLICOLOR", "1")
        .env("COLORTERM", "truecolor")
        .env("TERM", "xterm-256color")
        .output()
        .unwrap()
        .stdout;

    let stdout_string = String::from_utf8_lossy(&stdout);
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

    // println!(r#"Encoded: {}"#, encoded);

    Ok(EvaluationResult::Insert(Value::String(encoded)))
}

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
    use std::error::Error;

    use serde_yaml::Value;

    use crate::utils::evaluate;

    #[test]
    fn evaluate_transparent_sequence_test() -> Result<(), Box<dyn Error>> {
        let source_string = "
            - one
            - two
            - !TransparentSequence
                sequence:
                    - three-1
                    - three-2
            - four
        ";

        let source_value: Value = serde_yaml::from_str(&source_string)?;
        let target_value = evaluate(&source_value)?;

        let expected_string = "
            - one
            - two
            - [three-1, three-2]
            - four
        ";

        let expected_value: Value = serde_yaml::from_str(expected_string)?;

        assert_eq!(target_value, expected_value);

        Ok(())
    }

    #[test]
    fn evaluate_merge_sequence_test() -> Result<(), Box<dyn Error>> {
        let source_string = "
            - one
            - two
            - !MergeSequence
                sequence:
                    - three-1
                    - three-2
            - four
        ";

        let source_value: Value = serde_yaml::from_str(&source_string)?;
        let target_value = evaluate(&source_value)?;

        let expected_string = "
            - one
            - two
            - three-1
            - three-2
            - four
        ";

        let expected_value: Value = serde_yaml::from_str(expected_string)?;

        assert_eq!(target_value, expected_value);

        Ok(())
    }

    #[test]
    fn evaluate_transparent_mapping_test() -> Result<(), Box<dyn Error>> {
        let source_string = "
            one: 1
            two: 2
            three: !TransparentMapping
                mapping:
                   three-1: 3.1
                   three-2: 3.2
            four: 4
        ";

        let source_value: Value = serde_yaml::from_str(&source_string)?;
        let target_value = evaluate(&source_value)?;

        let expected_string = "
            one: 1
            two: 2
            three:
               three-1: 3.1
               three-2: 3.2
            four: 4
        ";

        let expected_value: Value = serde_yaml::from_str(expected_string)?;

        assert_eq!(target_value, expected_value);

        Ok(())
    }

    #[test]
    fn evaluate_merge_mapping_test() -> Result<(), Box<dyn Error>> {
        let source_string = "
            one: 1
            two: 2
            three: !MergeMapping
                mapping:
                   three-1: 3.1
                   three-2: 3.2
            four: 4
        ";

        let source_value: Value = serde_yaml::from_str(&source_string)?;
        let target_value = evaluate(&source_value)?;

        let expected_string = "
            one: 1
            two: 2
            three-1: 3.1
            three-2: 3.2
            four: 4
        ";

        let expected_value: Value = serde_yaml::from_str(expected_string)?;

        assert_eq!(target_value, expected_value);

        Ok(())
    }

    #[test]
    fn evaluate_bash_command_test() -> Result<(), Box<dyn Error>> {
        let source_string = "
            - input: echo 'Hello!'
              output: !Shell echo 'Hello!'
        ";

        let source_value: Value = serde_yaml::from_str(&source_string)?;
        let target_value = evaluate(&source_value)?;

        let expected_string = "
            - input: echo 'Hello!'
              output: \"Hello!\\n\"
        ";

        let expected_value: Value = serde_yaml::from_str(expected_string)?;

        assert_eq!(target_value, expected_value);

        Ok(())
    }

    #[test]
    fn read_write_ascii_control_chars() -> Result<(), Box<dyn Error>> {
        println!("\\u001b[32mdev@pc\\u001b[00m:\\u001b[34m~/my-application\\u001b[00m$ ");
        println!("\x1b[32mdev@pc\x1b[00m:\x1b[34m~/my-application\x1b[00m$ ");
        Ok(())
    }
}
