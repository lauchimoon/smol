use crate::interpreter::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
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
            panic!("key '{name}' is being redefined");
        }
        self.values.insert(name, value);
    }

    pub fn update(&mut self, name: String, new_value: Value) {
        if !self.values.contains_key(&name) {
            panic!("undefined key '{name}'");
        }

        self.values.entry(name)
            .and_modify(|v| *v = new_value);
    }

    pub fn get(&mut self, name: String) -> Value {
        match self.values.get(&name) {
            Some(v) => v.clone(),
            None => panic!("undefined key '{name}'"),
        }
    }
}
