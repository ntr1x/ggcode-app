use std::error::Error;

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
    let stdout = std::process::Command::new("bash")
        .arg("-c")
        .arg(&st)
        // .env("LS_COLORS", "rs=0:di=01;34:ln=01;36:mh=00:pi=40;33:so=01;35:do=01;35:bd=40;33;01:cd=40;33;01:or=40;31;01:mi=00:su=37;41:sg=30;43:ca=30;41:tw=30;42:ow=34;42:st=37;44:ex=01;32:*.tar=01;31:*.tgz=01;31:*.arc=01;31:*.arj=01;31:*.taz=01;31:*.lha=01;31:*.lz4=01;31:*.lzh=01;31:*.lzma=01;31:*.tlz=01;31:*.txz=01;31:*.tzo=01;31:*.t7z=01;31:*.zip=01;31:*.z=01;31:*.dz=01;31:*.gz=01;31:*.lrz=01;31:*.lz=01;31:*.lzo=01;31:*.xz=01;31:*.zst=01;31:*.tzst=01;31:*.bz2=01;31:*.bz=01;31:*.tbz=01;31:*.tbz2=01;31:*.tz=01;31:*.deb=01;31:*.rpm=01;31:*.jar=01;31:*.war=01;31:*.ear=01;31:*.sar=01;31:*.rar=01;31:*.alz=01;31:*.ace=01;31:*.zoo=01;31:*.cpio=01;31:*.7z=01;31:*.rz=01;31:*.cab=01;31:*.wim=01;31:*.swm=01;31:*.dwm=01;31:*.esd=01;31:*.jpg=01;35:*.jpeg=01;35:*.mjpg=01;35:*.mjpeg=01;35:*.gif=01;35:*.bmp=01;35:*.pbm=01;35:*.pgm=01;35:*.ppm=01;35:*.tga=01;35:*.xbm=01;35:*.xpm=01;35:*.tif=01;35:*.tiff=01;35:*.png=01;35:*.svg=01;35:*.svgz=01;35:*.mng=01;35:*.pcx=01;35:*.mov=01;35:*.mpg=01;35:*.mpeg=01;35:*.m2v=01;35:*.mkv=01;35:*.webm=01;35:*.webp=01;35:*.ogm=01;35:*.mp4=01;35:*.m4v=01;35:*.mp4v=01;35:*.vob=01;35:*.qt=01;35:*.nuv=01;35:*.wmv=01;35:*.asf=01;35:*.rm=01;35:*.rmvb=01;35:*.flc=01;35:*.avi=01;35:*.fli=01;35:*.flv=01;35:*.gl=01;35:*.dl=01;35:*.xcf=01;35:*.xwd=01;35:*.yuv=01;35:*.cgm=01;35:*.emf=01;35:*.ogv=01;35:*.ogx=01;35:*.aac=00;36:*.au=00;36:*.flac=00;36:*.m4a=00;36:*.mid=00;36:*.midi=00;36:*.mka=00;36:*.mp3=00;36:*.mpc=00;36:*.ogg=00;36:*.ra=00;36:*.wav=00;36:*.oga=00;36:*.opus=00;36:*.spx=00;36:*.xspf=00;36:")
        .output()
        .unwrap()
        .stdout;
    // let output = stdout.to_string();
    Ok(EvaluationResult::Insert(Value::String(String::from_utf8(stdout)?)))
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

// #[test]
// fn evaluate_bash_command_test() -> Result<(), Box<dyn Error>> {
//     let source_string = "
//         - input: pwd
//           output: !Shell pwd
//         - input: ls -la
//           output: !Shell ls -la
//     ";
//
//     let source_value: Value = serde_yaml::from_str(&source_string)?;
//     let target_value = evaluate(&source_value)?;
//
//     let expected_string = "
//         - one: 1
//         - two: 2
//     ";
//
//     let expected_value: Value = serde_yaml::from_str(expected_string)?;
//
//     assert_eq!(target_value, expected_value);
//
//     Ok(())
// }