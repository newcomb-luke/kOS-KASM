use std::{collections::HashMap, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelType {
    DEF,
    FUNC,
    INIT,
    UNDEFFUNC,
    UNDEFINIT,
    UNDEF
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelInfo {
    GLOBAL,
    LOCAL,
    EXTERN
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelValue {
    NONE,
    STRING(String)
}

#[derive(Debug, Clone)]
pub struct Label {
    id: String,
    lt: LabelType,
    li: LabelInfo,
    lv: LabelValue,
}

pub struct LabelManager {
    labels: HashMap<String, Label>,
}

impl Label {
    pub fn new(identifier: &str, lt: LabelType, li: LabelInfo, lv: LabelValue) -> Label {
        Label {
            id: identifier.to_owned(),
            lt,
            li,
            lv
        }
    }

    pub fn label_type(&self) -> LabelType {
        self.lt
    }

    pub fn label_info(&self) -> LabelInfo {
        self.li
    }

    pub fn label_value(&self) -> &LabelValue {
        &self.lv
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn as_str(&self) -> String {
        format!("{}: {:?}, {:?}, {:?}", self.id, self.lt, self.li, self.lv)
    }
}

impl LabelManager {
    pub fn new() -> LabelManager {
        LabelManager { labels: HashMap::new() }
    }

    pub fn def(&mut self, identifier: &str, label: Label) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.labels
        .insert(String::from(identifier), label);
    }

    pub fn ifdef(&self, identifier: &str) -> bool {
        self.labels.contains_key(identifier)
    }

    pub fn get(&self, identifier: &str) -> Result<&Label, Box<dyn Error>> {
        if self.ifdef(identifier) {
            Ok(self.labels.get(identifier).unwrap())
        } else {
            Err(format!("Label {} referenced before definition", identifier).into())
        }
    }

    pub fn as_vec(&self) -> Vec<&Label> {
        let mut all_values = Vec::new();

        for label in self.labels.values() {
            all_values.push(label);
        }

        all_values
    }

    /// Returns an option containing the label which contains the given value, if one is found
    pub fn contains_value(&self, value: LabelValue) -> Option<Label> {

        // Loop through each value
        for label in self.labels.values() {
            // If it does
            if *label.label_value() == value {
                // Return that
                return Some(label.clone());
            }
        }

        None
    }
}