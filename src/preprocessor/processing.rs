use std::{collections::HashMap, error::Error, slice::Iter, fs, iter::Peekable, path::Path};

use crate::{Token, Lexer, Instruction, InputFiles, TokenType, TokenData, Definition, Macro};

pub struct DefinitionTable {
    definitions: HashMap<String, Definition>,
}

pub enum SymbolType {
    LABEL,
    EMPTY,
    MACRO
}

pub enum SymbolInfo {
    GLOBAL,
    LOCAL,
    EXTERN
}

pub struct Symbol {
    id: String,

}

pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

pub struct MacroTable {
    macros: HashMap<String, Macro>,
}

pub struct PreprocessorSettings {
    pub expand_macros: bool,
    pub expand_definitions: bool,
}

pub struct Preprocessor {
    include_path: String,
    macro_invocation_map: HashMap<String, u32>
}

impl Preprocessor {

    pub fn new(include_path: String) -> Preprocessor {
        Preprocessor {
            include_path,
            macro_invocation_map: HashMap::new(),
        }
    }

    pub fn process(&mut self, settings: PreprocessorSettings, input: Vec<Token>, definition_table: &mut DefinitionTable, macro_table: &mut MacroTable, symbol_table: &mut SymbolTable, input_files: &mut InputFiles) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut new_tokens = Vec::with_capacity(input.len());

        let mut token_iter = input.iter().peekable();
        
        while token_iter.peek().is_some() {
            let token = token_iter.peek().unwrap();

            if token.tt() == TokenType::DIRECTIVE {
                let directive = DirectiveType::from_str(match token.data() { TokenData::STRING(s) => s, _ => unreachable!()})?;
                let directive_line = token.line();
                token_iter.next();

                match directive {
                    DirectiveType::DEFINE => {
                        let definition = Definition::parse_definition(&mut token_iter)?;

                        if macro_table.ifdef(&definition.id()) {
                            return Err(format!("Cannot create definiton {} with same name as macro. Line {}", definition.id(), directive_line).into());
                        }

                        definition_table.def(&definition.id(), definition);
                    },
                    DirectiveType::MACRO => {
                        let parsed_macro = Macro::parse_macro(directive_line, &mut token_iter)?;
                        let final_macro;
                        let final_contents;
                        let macro_settings = PreprocessorSettings { expand_definitions: false, expand_macros: true };

                        if definition_table.ifdef(&parsed_macro.id()) {
                            return Err(format!("Cannot create macro {} with same name as definiton. Line {}", parsed_macro.id(), directive_line).into());
                        }

                        final_contents = self.process(macro_settings, parsed_macro.contents_cloned(), definition_table, macro_table, symbol_table, input_files)?;

                        final_macro = Macro::new(&parsed_macro.id(), final_contents, parsed_macro.args_cloned(), parsed_macro.num_required_args());

                        macro_table.def(&parsed_macro.id(), final_macro);
                    }
                    _ => unimplemented!(),
                }
            } else {
                let mut append = self.process_line(&settings, &mut token_iter, definition_table, macro_table)?;

                new_tokens.append(&mut append);
            }
        }

        Ok(new_tokens)
    }

    pub fn process_line(&mut self, settings: &PreprocessorSettings, token_iter: &mut Peekable<Iter<Token>>, definition_table: &mut DefinitionTable, macro_table: &mut MacroTable) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut new_tokens = Vec::new();

        // While we haven't reached the end of the line
        while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {

            let token = token_iter.next().unwrap().clone();

            if token.tt() == TokenType::IDENTIFIER {
                let id = match token.data() { TokenData::STRING(s) => s, _ => unreachable!() };
                let line =  token.line();

                // 0 means that it is not a valid mnemonic
                if Instruction::opcode_from_mnemonic(id) == 0 {
                    // If it is a defined macro
                    if macro_table.ifdef(id) {
                        // And if we want it to be expanded
                        if settings.expand_macros {
                            let mut expanded = match self.expand_macro(id, token_iter, definition_table, macro_table)  {
                                Ok(expanded) => expanded,
                                Err(e) => {
                                    return Err(format!("{}. Line {}", e, line).into());
                                }
                            };

                            // Append the expansion
                            new_tokens.append(&mut expanded);
                        }
                        // If we don't, just append it
                        else {
                            new_tokens.push(token);
                        }
                    }
                    // If it is a definition
                    else if definition_table.ifdef(id) {
                        // And if we want it to be expanded
                        if settings.expand_definitions {
                            let mut expanded = match self.expand_definition(id, token_iter, definition_table) {
                                Ok(expanded) => expanded,
                                Err(e) => {
                                    return Err(format!("{}. Line {}", e, line).into());
                                }
                            };
    
                            new_tokens.append(&mut expanded);
                        }
                        // If not, just append it
                        else {
                            new_tokens.push(token);
                        }

                    } else {
                        return Err(format!("Macro or definition used before declaration: {}, line {}.", id, line).into());
                    }
                }
                // If it is a valid mnemonic, then just append it
                else {
                    new_tokens.push(token);
                }
            } else {
                new_tokens.push(token);
            }

        }

        // If the line ended because of a new line
        if token_iter.peek().is_some() {
            // Add it to the end
            new_tokens.push(token_iter.next().unwrap().clone());
        }

        Ok(new_tokens)
    }

    pub fn expand_macro(&mut self, id: &str, token_iter: &mut Peekable<Iter<Token>>, definition_table: &DefinitionTable, macro_table: &MacroTable) -> Result<Vec<Token>, Box<dyn Error>> {
        let macro_ref = macro_table.get(id)?;
        let macro_args = macro_ref.args();
        let num_required_args = macro_ref.num_required_args();
        let mut content_iter = macro_ref.get_contents_iter();
        let mut without_placeholders = Vec::new();
        let mut without_placeholders_iter;
        let mut expanded_tokens = Vec::new();
        let mut args = Vec::new();
        let invocation_id;

        // In order to solve the problem of macro local labels, each macro invocation is given an id, this is appended to the macro's name followed by an underscore
        // Ex. if this was the second time the macro was expanded, and the macro's name was WRITE
        // __WRITE_2
        // This way, a local label ".loop" would be:
        // __WRITE_2.loop
        
        // If this is the first time
        if self.macro_invocation_map.get(id).is_none() {
            // Create the entry
            self.macro_invocation_map.insert(String::from(id), 1);
            // Set the id
            invocation_id = format!("__{}_{}", id, 0);
        } else {
            // Get the number of the invocation
            let num = *self.macro_invocation_map.get(id).unwrap();
            // Increment the value in the map
            self.macro_invocation_map.insert(String::from(id), num + 1);
            // SEt the id
            invocation_id = format!("__{}_{}", id, num);
        }

        // We need to collect all arguments before the newline
        while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {

            // We will stop the current argument if there is a comma
            while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::COMMA && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                let mut arg_tokens = Vec::new();

                // Get the token
                let token = token_iter.next().unwrap();

                // If this token is a definiition expansion
                if token.tt() == TokenType::IDENTIFIER && definition_table.ifdef(id) {
                    // Get the id
                    let inner_id = match token.data() { TokenData::STRING(s) => s, _ => unreachable!()};

                    // Expand it
                    let mut inner = self.expand_definition(inner_id, token_iter, definition_table)?;

                    // Append it to this argument's tokens
                    arg_tokens.append(&mut inner);
                }
                // If not
                else {
                    // Just append the token
                    arg_tokens.push(token.clone());
                }

                // Add this argument to the list
                args.push(arg_tokens);
            }

            // If it was a comma, consume it
            if token_iter.peek().unwrap().tt() == TokenType::COMMA {
                token_iter.next();
            }
        }

        // If it was a newline that ended it, consume it
        if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::NEWLINE {
            token_iter.next();
        }

        // Note: We are actually perfectly fine if this ended because there was EOF

        // Before we go on we need to test if we have the correct number of arguments
        if num_required_args > args.len() || args.len() > macro_args.len() {
            return Err(format!("Invalid number of arguments for macro {} expansion, invocation has {}, expected {}", id, args.len(),num_required_args).into());
        }

        // Now we need to fill in the rest of the arguments using their default values if they need them
        for i in args.len()..macro_args.len() {
            // The logic to decide if this argument is required is handled above
            // Now we just need to append the tokens contained in the default value if the macro arg to the args
            args.push(macro_args.get(i).unwrap().default_owned());
        }

        // Now we need to replace all of the placeholders with the argument values
        while content_iter.peek().is_some() {
            let token = content_iter.next().unwrap();

            // If the token is a placeholder
            if token.tt() == TokenType::PLACEHOLDER {
                let arg_index = match token.data() { TokenData::INT(i) => *i as usize, _ => unreachable!()};

                // Replace it with the tokens of that argument
                for token in args.get(arg_index - 1).unwrap() {
                    without_placeholders.push(token.clone());
                }
            }
            // If it is a local label, then we need to prepend the invocation id
            else if token.tt() == TokenType::INNERLABEL {
                // Extract the local label's name
                let label_name = match token.data() { TokenData::STRING(s) => s, _ => unreachable!() };
                // Create the combined name
                let new_name = format!("{}.{}", invocation_id, label_name);

                // So this acts correctly in global contexts, make it a regular label
                without_placeholders.push(Token::new(TokenType::LABEL, TokenData::STRING(new_name)));
            }
            // If it isn't either
            else {
                // Just append the token
                without_placeholders.push(token.clone());
            }
        }

        // Create an iterator
        without_placeholders_iter = without_placeholders.iter().peekable();

        // Loop through each token in the definiton's new contents
        while without_placeholders_iter.peek().is_some() {
            let token = without_placeholders_iter.next().unwrap();

            // If this token is a definition expansion
            if token.tt() == TokenType::IDENTIFIER {
                // Get the id
                let inner_id = match token.data() { TokenData::STRING(s) => s, _ => unreachable!()};

                if definition_table.ifdef(inner_id) {

                    // Expand it
                    let mut inner = self.expand_definition(inner_id, &mut without_placeholders_iter, definition_table)?;
    
                    // Append it to the expanded tokens list
                    expanded_tokens.append(&mut inner);
                } else {
                    // Just append the token
                    expanded_tokens.push(token.clone());
                }
            }
            // If it isn't
            else {
                // Just append the token
                expanded_tokens.push(token.clone());
            }
        }

        // Finally, return the expanded tokens

        Ok(expanded_tokens)
    }

    pub fn expand_definition(&self, id: &str, token_iter: &mut Peekable<Iter<Token>>, definition_table: &DefinitionTable) -> Result<Vec<Token>, Box<dyn Error>> {
        let def_ref = definition_table.get(id)?;
        let mut content_iter = def_ref.get_contents_iter();
        let mut without_placeholders = Vec::new();
        let mut without_placeholders_iter;
        let mut expanded_tokens = Vec::new();
        let mut args = Vec::new();

        // We can't really expand a definition if it is in fact, empty
        if def_ref.is_empty() {
            return Err(format!("Definition {} is empty, and cannot be expanded", id).into());
        }

        // If the next token is an open parenthesis, then we have some arguments
        if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::OPENPAREN {

            // This will help us collect all of the arguments, because all we need to look for is the closing parenthesis
            while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::CLOSEPAREN {
                let mut arg_tokens = Vec::new();

                // Consume either the opening parenthesis, or the preceeding comma
                token_iter.next();

                // We want to collect all tokens before there is a comma, or close parenthesis.
                while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::COMMA && token_iter.peek().unwrap().tt() != TokenType::CLOSEPAREN {
                    // Get the token
                    let token = token_iter.next().unwrap();

                    // If this token is another definiition expansion...
                    if token.tt() == TokenType::IDENTIFIER && definition_table.ifdef(id) {
                        // Get the id
                        let inner_id = match token.data() { TokenData::STRING(s) => s, _ => unreachable!()};

                        // Expand it
                        let mut inner = self.expand_definition(inner_id, token_iter, definition_table)?;

                        // Append it to this argument's tokens
                        arg_tokens.append(&mut inner);
                    }
                    // If not
                    else {
                        // Just append the token
                        arg_tokens.push(token.clone());
                    }
                }

                // Add this argument to the list
                args.push(arg_tokens);
            }

            // If the loop is over, that means there is a closing parenthesis, or no more tokens...
            // If the case is no more tokens, that is an error
            match token_iter.next() {
                Some(_token) => {},
                None => {
                    return Err("Error reading arguments, expected closing parenthesis. Found end of file.".into());
                }
            }
        }

        // Before we go on we need to test if we have the correct number of arguments
        if def_ref.num_args() != args.len() {
            return Err(format!("Invalid number of arguments for definition {} expansion, invocation has {}, expected {}", id, args.len(), def_ref.num_args()).into());
        }

        // Now that we are done with the arguments if there were any, we can begin expansion

        // First though, we need to replace all of the placeholders with the argument values
        while content_iter.peek().is_some() {
            let token = content_iter.next().unwrap();

            // If the token is a placeholder
            if token.tt() == TokenType::PLACEHOLDER {
                let arg_index = match token.data() { TokenData::INT(i) => *i as usize, _ => unreachable!()};

                // Replace it with the tokens of that argument
                for token in args.get(arg_index).unwrap() {
                    without_placeholders.push(token.clone());
                }
            }
            // If it isn't
            else {
                // Just append the token
                without_placeholders.push(token.clone());
            }
        }

        // Create an iterator
        without_placeholders_iter = without_placeholders.iter().peekable();

        // Loop through each token in the definiton's new contents
        while without_placeholders_iter.peek().is_some() {
            let token = without_placeholders_iter.next().unwrap();

            // If this token is another definition expansion...
            if token.tt() == TokenType::IDENTIFIER && definition_table.ifdef(id) {
                // Get the id
                let inner_id = match token.data() { TokenData::STRING(s) => s, _ => unreachable!()};

                // We don't want recursive expansion, that would be bad.
                if *id == *inner_id {
                    return Err(format!("Cannot have recursive definition expansion: {}", id).into());
                }

                // Expand it
                let mut inner = self.expand_definition(inner_id, &mut without_placeholders_iter, definition_table)?;

                // Append it to the expanded tokens list
                expanded_tokens.append(&mut inner);
            }
            // If it isn't
            else {
                // Just append the token
                expanded_tokens.push(token.clone());
            }
        }

        // Finally, return the expanded tokens

        Ok(expanded_tokens)
    }

    pub fn include_file(&self, file_path: &str, input_files: &mut InputFiles) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut file_path = Path::new(file_path);
        let file_name;
        let file_id;
        let path_buffer;
        let contents;
        let tokens;

        // Create a new lexer just for this file
        let mut lexer = Lexer::new();

        // If the file does not exist, error out here
        if !file_path.exists() {
            return Err(format!("Could not include {}, file does not exist.", file_path.to_str().unwrap()).into());
        }

        // If the file isn't a file, there is a problem
        if !file_path.is_file() {
            return Err(format!("Could not include {}, directories cannot be included.", file_path.to_str().unwrap()).into());
        }

        // If it is an absolute path, we don't need to do anything. If it is not, make it one by adding the include path
        if !file_path.is_absolute() {
            let include_path = Path::new(&self.include_path);
            path_buffer = include_path.join(file_path);
            file_path = path_buffer.as_path();
        }

        // Attempt to read the file as text
        contents = fs::read_to_string(file_path)?;

        // Add this file to the inputfiles
        file_name = String::from(file_path.file_name().unwrap().to_str().unwrap());
        file_id = input_files.add_file(&file_name);

        // Lex the contents
        tokens = lexer.lex(&contents, &file_name, file_id)?;
        
        Ok(tokens)

    }

}

impl DefinitionTable {
    pub fn new() -> DefinitionTable {
        DefinitionTable {
            definitions: HashMap::new(),
        }
    }

    pub fn def(&mut self, identifier: &str, new_definition: Definition) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.definitions
            .insert(String::from(identifier), new_definition);
    }

    pub fn ifdef(&self, identifier: &str) -> bool {
        self.definitions.contains_key(identifier)
    }

    pub fn ifndef(&self, identifier: &str) -> bool {
        !self.ifdef(identifier)
    }

    pub fn undef(&mut self, identifier: &str) {
        self.definitions.remove(identifier);
    }

    pub fn get(&self, identifier: &str) -> Result<&Definition, Box<dyn Error>> {
        if self.ifdef(identifier) {
            Ok(self.definitions.get(identifier).unwrap())
        } else {
            Err(format!("Constant {} referenced before definition", identifier).into())
        }
    }
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { symbols: HashMap::new() }
    }
}

impl MacroTable {

    pub fn new() -> MacroTable {
        MacroTable { macros: HashMap::new() }
    }

    pub fn def(&mut self, identifier: &str, new_macro: Macro) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.macros
            .insert(String::from(identifier), new_macro);
    }

    pub fn ifdef(&self, identifier: &str) -> bool {
        self.macros.contains_key(identifier)
    }

    pub fn get(&self, identifier: &str) -> Result<&Macro, Box<dyn Error>> {
        if self.ifdef(identifier) {
            Ok(self.macros.get(identifier).unwrap())
        } else {
            Err(format!("Macro {} referenced before definition", identifier).into())
        }
    }
}

pub enum DirectiveType {
    DEFINE,
    UNDEF,
    MACRO,
    ENDMACRO,
    UNMACRO,
    IF,
    IFN,
    ELIF,
    ELIFN,
    ELIFDEF,
    ELIFNDEF,
    ELSE,
    ENDIF,
    IFDEF,
    IFNDEF,
    REP,
    ENDREP,
    INCLUDE,
    LINE,
    EXTERN,
    GLOBAL
}

impl DirectiveType {
    pub fn from_str(s: &str) -> Result<DirectiveType, Box<dyn Error>> {
        Ok(match s {
            "define" => DirectiveType::DEFINE,
            "undef" => DirectiveType::UNDEF,
            "macro" => DirectiveType::MACRO,
            "unmacro" => DirectiveType::UNMACRO,
            "endmacro" => DirectiveType::ENDMACRO,
            "if" => DirectiveType::IF,
            "ifn" => DirectiveType::IFN,
            "elif" => DirectiveType::ELIF,
            "elifn" => DirectiveType::ELIFN,
            "else" => DirectiveType::ELSE,
            "endif" => DirectiveType::ENDIF,
            "elifdef" => DirectiveType::ELIFDEF,
            "elifndef" => DirectiveType::ELIFNDEF,
            "ifdef" => DirectiveType::IFDEF,
            "ifndef" => DirectiveType::IFNDEF,
            "rep" => DirectiveType::REP,
            "endrep" => DirectiveType::ENDREP,
            "include" => DirectiveType::INCLUDE,
            "line" => DirectiveType::LINE,
            "extern" => DirectiveType::EXTERN,
            "global" => DirectiveType::GLOBAL,
            _ => {
                return Err(format!("Invalid directive found: {}", s).into());
            }
        })
    }
}