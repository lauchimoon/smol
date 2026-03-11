use crate::interpreter::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, value: Value) {
        if self.values.contains_key(&name) {
            panic!("{name} being redefined");
        }
        self.values.insert(name, value);
    }

    pub fn get(&mut self, name: String) -> Value {
        match self.values.get(&name) {
            Some(v) => v.clone(),
            None => panic!("value with name '{}' not found", name),
        }
    }
}
