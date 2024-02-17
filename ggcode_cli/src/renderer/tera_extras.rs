use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde_json::value::{to_value, Value};
use tera::try_get_value;

struct AnsiReplacement {
    code: String,
}

pub fn format_ansi(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("format_ansi", "value", String, value);

    lazy_static! {
        static ref RE_STYLE: Regex = Regex::new(r"\[((style:(?P<style>[rbdius]+))|(color:(?P<color>[0rgybmcw]+)))\]").unwrap();

        static ref MAP_STYLE_BY_LETTER: BTreeMap<char, AnsiReplacement> = BTreeMap::from([
            ('r', AnsiReplacement { code: "00".into() }), // reset
            ('b', AnsiReplacement { code: "01".into() }), // bold
            ('d', AnsiReplacement { code: "02".into() }), // dim
            ('i', AnsiReplacement { code: "03".into() }), // italic
            ('u', AnsiReplacement { code: "04".into() }), // underline
            ('s', AnsiReplacement { code: "09".into() }), // strike
        ]);

        static ref MAP_COLOR_BY_LETTER: BTreeMap<char, AnsiReplacement> = BTreeMap::from([
            ('0', AnsiReplacement { code: "30".into() }), // black
            ('r', AnsiReplacement { code: "31".into() }), // red
            ('g', AnsiReplacement { code: "32".into() }), // green
            ('y', AnsiReplacement { code: "33".into() }), // yellow
            ('b', AnsiReplacement { code: "34".into() }), // blue
            ('m', AnsiReplacement { code: "35".into() }), // magenta
            ('c', AnsiReplacement { code: "36".into() }), // cyan
            ('w', AnsiReplacement { code: "37".into() }), // white
        ]);
    }

    let replacement = |caps: &Captures| -> Result<String, &'static str> {

        let style_option = caps.name("style");
        let color_option = caps.name("color");

        let (sequence, map): (&str, &BTreeMap<char, AnsiReplacement>) = match (style_option, color_option) {
            (Some(style), None) => (style.as_str(), &MAP_STYLE_BY_LETTER),
            (None, Some(color)) => (color.as_str(), &MAP_COLOR_BY_LETTER),
            _ => unreachable!()
        };

        let mut st = String::from("");
        for letter in sequence.chars() {
            if !st.is_empty() {
                st.push_str(";");
            }
            st.push_str(map.get(&letter).unwrap().code.as_str());
        }
        st.push_str("m");

        Ok(format!("\\u001b[{}", st))
    };

    let r = replace_all(&RE_STYLE, &s, &replacement)?;

    Ok(to_value(r).unwrap())
}

fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::error::Error;

    use serde_json::{from_value, to_value};

    use crate::renderer::tera_extras::format_ansi;

    #[test]
    fn format_ansi_style_test() -> Result<(), Box<dyn Error>> {
        let value = to_value("[style:bu]-h, --help[style:r]\t\tDisplay help")?;
        let result_value = format_ansi(&value, &HashMap::new()).unwrap();
        let result_string = from_value::<String>(result_value)?;
        assert_eq!(result_string, "\\u001b[01;04m-h, --help\\u001b[00m\t\tDisplay help");
        Ok(())
    }

    #[test]
    fn format_ansi_color_test() -> Result<(), Box<dyn Error>> {
        let value = to_value("[color:g]dev@pc[style:r]")?;
        let result_value = format_ansi(&value, &HashMap::new()).unwrap();
        let result_string = from_value::<String>(result_value)?;
        assert_eq!(result_string, "\\u001b[32mdev@pc\\u001b[00m");
        Ok(())
    }
}