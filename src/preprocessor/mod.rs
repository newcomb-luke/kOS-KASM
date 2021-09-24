use std::collections::HashMap;

use crate::{
    errors::{ErrorManager, KASMResult, SourceFile},
    lexer::{
        token::{Token, TokenKind},
        TokenIter,
    },
    preprocessor::definitions::UnDefinition,
};

use self::{definitions::Definition, macros::Macro};

pub mod phase0;

pub mod expressions;

pub mod definitions;
pub mod macros;

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
}

// The mode of the preprocessor
enum PreprocessMode {
    Text,
    Data,
}

/// Runs the preprocessor on the tokens provided, using the given include path if any .include
/// directives are encountered
pub fn preprocess(
    include_path: &str,
    tokens: Vec<Token>,
    source_files: &mut Vec<SourceFile>,
    errors: &mut ErrorManager,
) -> KASMResult<Vec<Token>> {
    let mut preprocessor = Preprocessor::new(include_path);

    Ok(preprocessor.process(tokens, source_files, errors)?)
}

/// The preprocessor that evaluates expressions and executes directives
pub struct Preprocessor {
    include_path: String,
    mode: PreprocessMode,
    definitions: DefinitionTable<Definition>,
    macros: DefinitionTable<Macro>,
}

impl Preprocessor {
    pub fn new(include_path: &str) -> Self {
        Self {
            include_path: include_path.to_string(),
            mode: PreprocessMode::Text,
            definitions: DefinitionTable::new(),
            macros: DefinitionTable::new(),
        }
    }

    pub fn process(
        &mut self,
        tokens: Vec<Token>,
        source_files: &mut Vec<SourceFile>,
        errors: &mut ErrorManager,
    ) -> KASMResult<Vec<Token>> {
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

        Ok(preprocessed_tokens)
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
    ) -> KASMResult<()> {
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
            TokenKind::Newline => {
                token_iter.next();
            }
            _ => unimplemented!("Not implemented yet"),
        }

        Ok(())
    }
}
