use std::collections::{hash_map::Iter, HashMap};

use kerbalobjects::{kofile::symbols::SymBind, KOSValue};

use crate::errors::Span;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolType {
    Func,
    Value,
    Default,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolValue {
    Value(KOSValue),
    Function,
    Undefined,
}

#[derive(Debug, Clone)]
pub struct DeclaredSymbol {
    pub declared_span: Span,
    pub binding: SymBind,
    pub sym_type: SymbolType,
    pub value: SymbolValue,
}

impl DeclaredSymbol {
    pub fn new(span: Span, binding: SymBind, sym_type: SymbolType, value: SymbolValue) -> Self {
        Self {
            declared_span: span,
            binding,
            sym_type,
            value,
        }
    }
}

pub struct SymbolManager {
    map: HashMap<String, DeclaredSymbol>,
}

impl SymbolManager {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn contains(&self, identifier: &String) -> bool {
        self.map.contains_key(identifier)
    }

    pub fn get(&self, identifier: &String) -> Option<&DeclaredSymbol> {
        self.map.get(identifier)
    }

    pub fn get_mut(&mut self, identifier: &String) -> Option<&mut DeclaredSymbol> {
        self.map.get_mut(identifier)
    }

    pub fn insert(&mut self, identifier: String, declared: DeclaredSymbol) {
        self.map.insert(identifier, declared);
    }

    pub fn symbols(&self) -> Iter<String, DeclaredSymbol> {
        self.map.iter()
    }
}

impl Default for SymbolManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Label {
    pub value: usize,
    pub span: Span,
}

impl Label {
    pub fn new(value: usize, span: Span) -> Self {
        Self { value, span }
    }
}

pub struct LabelManager {
    map: HashMap<String, Label>,
}

impl LabelManager {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn contains(&self, name: &String) -> bool {
        self.map.contains_key(name)
    }

    pub fn get(&self, name: &String) -> Option<&Label> {
        self.map.get(name)
    }

    pub fn get_mut(&mut self, name: &String) -> Option<&mut Label> {
        self.map.get_mut(name)
    }

    pub fn insert(&mut self, name: String, label: Label) {
        self.map.insert(name, label);
    }

    pub fn labels(&self) -> Iter<String, Label> {
        self.map.iter()
    }
}

impl Default for LabelManager {
    fn default() -> Self {
        Self::new()
    }
}
