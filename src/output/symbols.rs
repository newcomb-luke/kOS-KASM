use kerbalobjects::{
    kofile::symbols::{SymBind, SymType},
    KOSValue,
};

/// Represents a symbol in KASM code, which is a simplified version of a KOSymbol from the KO file
/// format
pub struct Symbol {
    /// The type of the symbol. In KASM, only Func and NoType are supported
    pub sym_type: SymType,
    /// The binding of the symbol, Local, Global, or Extern
    pub sym_bind: SymBind,
    /// The value this symbol has. Can be None, or Some(KOSValue). For functions, the value will be
    /// a KOSValue::Int32() that will contain the instruction index of the function
    pub value: Option<KOSValue>,
}

impl Symbol {
    pub fn new(sym_type: SymType, sym_bind: SymBind) -> Self {
        Self {
            sym_type,
            sym_bind,
            value: None,
        }
    }

    pub fn with_value(sym_type: SymType, sym_bind: SymBind, value: KOSValue) -> Self {
        Self {
            sym_type,
            sym_bind,
            value: Some(value),
        }
    }
}
