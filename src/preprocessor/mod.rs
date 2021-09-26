use std::collections::HashMap;

use kerbalobjects::kofile::symbols::{SymBind, SymType};

use crate::{
    errors::{AssemblyError, ErrorKind, ErrorManager, SourceFile},
    lexer::{
        parse_token,
        token::{Token, TokenKind},
        TokenIter,
    },
    output::symbols::Symbol,
    preprocessor::{
        definitions::UnDefinition,
        macros::{MacroArgs, UnMacro},
        repeat::Repeat,
    },
};

use self::{definitions::Definition, macros::Macro};

pub mod phase0;

pub mod expressions;

pub mod definitions;
pub mod macros;
pub mod repeat;

/*
mod processing;
pub use processing::{DefinitionTable, MacroTable, Preprocessor, PreprocessorSettings};

mod macros;
pub use macros::Macro;

mod definition;
pub use definition::Definition;

mod evaluator;
pub use evaluator::ExpressionEvaluator;

mod labels;
pub use labels::{Label, LabelInfo, LabelManager, LabelType, LabelValue};

mod errors;
pub use errors::*;
*/

#[derive(Debug)]
struct DefinitionTable<T> {
    definitions: HashMap<String, T>,
}

impl<T> DefinitionTable<T> {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
        }
    }

    pub fn define(&mut self, identifier: &str, new: T) {
        self.definitions.insert(identifier.to_string(), new);
    }

    pub fn undef(&mut self, identifier: &str) {
        self.definitions.remove(identifier);
    }

    pub fn ifdef(&self, identifier: &str) -> bool {
        self.definitions.contains_key(identifier)
    }

    pub fn ifndef(&self, identifier: &str) -> bool {
        !self.ifdef(identifier)
    }

    pub fn get(&self, identifier: &str) -> Option<&T> {
        self.definitions.get(identifier)
    }

    pub fn get_mut(&mut self, identifier: &str) -> Option<&mut T> {
        self.definitions.get_mut(identifier)
    }
}

/// Runs the preprocessor on the tokens provided, using the given include path if any .include
/// directives are encountered
pub fn preprocess(
    include_path: &str,
    tokens: Vec<Token>,
    source_files: &mut Vec<SourceFile>,
    errors: &mut ErrorManager,
) -> Option<Vec<Token>> {
    let mut preprocessor = Preprocessor::new(include_path);

    Some(preprocessor.process(tokens, source_files, errors)?)
}

/// The preprocessor that evaluates expressions and executes directives
pub struct Preprocessor {
    include_path: String,
    definitions: DefinitionTable<Definition>,
    macros: DefinitionTable<Macro>,
}

impl Preprocessor {
    pub fn new(include_path: &str) -> Self {
        Self {
            include_path: include_path.to_string(),
            definitions: DefinitionTable::new(),
            macros: DefinitionTable::new(),
        }
    }

    pub fn process(
        &mut self,
        tokens: Vec<Token>,
        source_files: &mut Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> Option<Vec<Token>> {
        // Pre-allocate for as many tokens as we had before, it should be a good estimate
        let mut preprocessed_tokens = Vec::with_capacity(tokens.len());

        // Create a new TokenIter over the tokens
        let mut token_iter = TokenIter::new(tokens);

        // While we have more tokens
        while token_iter.peek().is_some() {
            // Preprocess!
            self.process_part(
                &mut token_iter,
                source_files,
                errors,
                &mut preprocessed_tokens,
            )?;
        }

        println!("Definitions: {:#?}", self.definitions);
        println!("Macros: {:#?}", self.macros);

        Some(preprocessed_tokens)
    }

    // This function preprocesses a "part"
    // Basically, if we have just begun preprocessing, this is called. If we have just finished
    // preprocessing a macro definition, then we call this function. This is called in the main
    // loop until we are done with the entire file
    fn process_part(
        &mut self,
        token_iter: &mut TokenIter,
        source_files: &mut Vec<SourceFile>,
        errors: &mut ErrorManager,
        preprocessed_tokens: &mut Vec<Token>,
    ) -> Option<()> {
        // In the loop we checked if there was another token or not
        let token = token_iter.peek().unwrap();

        match token.kind {
            TokenKind::DirectiveDefine => {
                let (definition, identifier) = Definition::parse(token_iter, source_files, errors)?;

                println!("Defined {}", identifier);

                self.definitions.define(&identifier, definition);
            }
            TokenKind::DirectiveUndef => {
                let identifier = UnDefinition::parse(token_iter, source_files, errors)?;

                println!("Undefined {}", identifier);

                self.definitions.undef(&identifier);
            }
            TokenKind::DirectiveRepeat => {
                let repeat = Repeat::parse(token_iter, source_files, errors)?;

                repeat.invoke(preprocessed_tokens);

                todo!("Invoke definitions and macros within .rep");
            }
            TokenKind::DirectiveMacro => {
                let (our_macro, identifier) = Macro::parse(token_iter, source_files, errors)?;

                println!("Defined macro {}", identifier);

                self.macros.define(&identifier, our_macro);
            }
            TokenKind::DirectiveUnmacro => {
                let (identifier, args) = UnMacro::parse(token_iter, source_files, errors)?;

                // This awful mess of match statements basically just means if the macro arguments
                // are equal or not
                let matches = if let Some(our_macro) = self.macros.get(&identifier) {
                    match our_macro.args() {
                        Some(args_1) => match args {
                            Some(args_2) => match args_1 {
                                MacroArgs::Fixed(args_1_num) => match args_2 {
                                    MacroArgs::Fixed(args_2_num) => *args_1_num == args_2_num,
                                    _ => false,
                                },
                                MacroArgs::Range(args_1_min, args_1_max, _) => match args_2 {
                                    MacroArgs::Range(args_2_min, args_2_max, _) => {
                                        *args_1_min == args_2_min && *args_1_max == args_2_max
                                    }
                                    _ => false,
                                },
                            },
                            None => false,
                        },
                        None => match args {
                            Some(_) => false,
                            None => true,
                        },
                    }
                } else {
                    false
                };

                // If the macro arguments are equal
                if matches {
                    self.macros.undef(&identifier);
                }
            }
            TokenKind::DirectiveInclude => {
                todo!("Implement .include");
            }
            TokenKind::DirectiveIf
            | TokenKind::DirectiveIfNot
            | TokenKind::DirectiveIfDef
            | TokenKind::DirectiveIfNotDef => {
                todo!("Implement .if");
            }
            TokenKind::Identifier => {
                todo!("Implement macro expansion");
            }
            // Preprocessor ignores
            TokenKind::KeywordSection
            | TokenKind::KeywordText
            | TokenKind::KeywordData
            | TokenKind::Label
            | TokenKind::InnerLabel
            | TokenKind::InnerLabelReference
            | TokenKind::LiteralInteger
            | TokenKind::LiteralFloat
            | TokenKind::LiteralHex
            | TokenKind::LiteralBinary
            | TokenKind::LiteralTrue
            | TokenKind::LiteralFalse
            | TokenKind::LiteralString
            | TokenKind::SymbolLeftParen
            | TokenKind::SymbolRightParen
            | TokenKind::SymbolComma
            | TokenKind::SymbolHash
            | TokenKind::SymbolAt
            | TokenKind::SymbolAnd
            | TokenKind::Newline
            | TokenKind::OperatorMinus
            | TokenKind::OperatorPlus
            | TokenKind::OperatorCompliment
            | TokenKind::OperatorMultiply
            | TokenKind::OperatorDivide
            | TokenKind::OperatorMod
            | TokenKind::OperatorAnd
            | TokenKind::OperatorOr
            | TokenKind::OperatorEquals
            | TokenKind::OperatorNotEquals
            | TokenKind::OperatorNegate
            | TokenKind::OperatorGreaterThan
            | TokenKind::OperatorLessThan
            | TokenKind::OperatorGreaterEquals
            | TokenKind::OperatorLessEquals
            | TokenKind::DirectiveGlobal
            | TokenKind::DirectiveExtern
            | TokenKind::DirectiveLocal
            | TokenKind::DirectiveLine
            | TokenKind::DirectiveValue
            | TokenKind::DirectiveFunc
            | TokenKind::DirectiveType => {
                // Just add it to the preprocessed tokens
                preprocessed_tokens.push(*token_iter.next().unwrap());
            }
            // Directives that are not allowed outside of their respective parsing scopes
            TokenKind::DirectiveEndmacro
            | TokenKind::DirectiveEndRepeat
            | TokenKind::DirectiveEndIf
            | TokenKind::DirectiveElse
            | TokenKind::DirectiveElseIf
            | TokenKind::DirectiveElseIfNot
            | TokenKind::DirectiveElseIfDef
            | TokenKind::DirectiveElseIfNotDef => {
                errors.add_assembly(AssemblyError::new(ErrorKind::DirectiveNotAllowed, *token));

                return None;
            }
            TokenKind::Backslash
            | TokenKind::Whitespace
            | TokenKind::Comment
            | TokenKind::Error
            | TokenKind::JunkFloatError => unreachable!(),
        }

        Some(())
    }
}

// This parses a symbol binding directive
//
// Examples:
//
// .extern func
// .global _start
//
// This returns an option of a tuple of the symbol binding that was specified, and a String of the
// symbol that is being specified
fn parse_binding(
    token_iter: &mut TokenIter,
    source_files: &mut Vec<SourceFile>,
    errors: &mut ErrorManager,
) -> Option<(SymBind, String)> {
    // This will always either be .extern, .global, or .local
    let bind_token = token_iter.next().unwrap();

    let bind = match bind_token.kind {
        TokenKind::DirectiveExtern => SymBind::Extern,
        TokenKind::DirectiveGlobal => SymBind::Global,
        TokenKind::DirectiveLocal => SymBind::Local,
        _ => unreachable!(),
    };

    let symbol_name_token = parse_token(
        token_iter,
        errors,
        TokenKind::Identifier,
        ErrorKind::ExpectedBindingIdentifier,
        ErrorKind::MissingBindingIdentifier,
    )?;

    // Get the actual symbol name
    let symbol_name = symbol_name_token.slice(source_files).unwrap().to_string();

    Some((bind, symbol_name))
}
