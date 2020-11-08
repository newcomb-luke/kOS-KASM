use std::{collections::HashMap, error::Error};

use crate::ExpNode;

pub enum Definition {
    Empty,
    Constant(ExpNode)
}

pub struct DefinitionTable {
    definitions: HashMap<String, Definition>
}

impl DefinitionTable {

    pub fn new() -> DefinitionTable {
        DefinitionTable { definitions: HashMap::new() }
    }

    pub fn def(&mut self, identifier: &str, new_definition: Definition) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.definitions.insert(String::from(identifier), new_definition);
    }

    pub fn ifdef(&mut self, identifier: &str) -> bool {
        self.definitions.contains_key(identifier)
    }

    pub fn ifndef(&mut self, identifier: &str) -> bool {
        !self.ifdef(identifier)
    }

    pub fn undef(&mut self, identifier: &str) {
        self.definitions.remove(identifier);
    }

    pub fn get(&mut self, identifier: &str) -> Result<&Definition, Box<dyn Error>> {
        if self.ifdef(identifier) {
            Ok(self.definitions.get(identifier).unwrap())
        }
        else {
            Err(format!("Constant {} referenced before definition", identifier).into())
        }
    }

}