use serde_yaml::Value;

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
