use crate::convert::RawValue;
use std::collections::HashMap;

pub struct Memory {
    pub stack: Vec<HashMap<String, RawValue>>
}

impl Memory {

    pub fn new() -> Self {
        Self {
            stack: vec![HashMap::new()]
        }
    }

    pub fn extend(&mut self) {
        self.stack.push(HashMap::new());
    }

    pub fn destroy(&mut self) {
        self.stack.pop();
    }

    pub fn get(&self, val: &String) -> Option<&RawValue> {
        for thing in self.stack.iter().rev() {
            if let Some(res) = thing.get(val) { return Some(res) }; 
        };
        return None;
    }

    pub fn set(&mut self, key: String, val: RawValue) {
        let len = self.stack.len();
        self.stack[len - 1].insert(key, val);
    }

}