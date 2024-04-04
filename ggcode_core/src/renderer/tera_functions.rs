use std::collections::HashMap;

use tera::{Result, to_value, Value};
use uuid::Uuid;

pub fn uuid_v4(_args: &HashMap<String, Value>) -> Result<Value> {
    let uuid: Uuid = Uuid::new_v4();
    let value: Value = to_value(uuid.to_string())?;
    Ok(value)
}