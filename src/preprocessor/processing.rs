use std::{collections::HashMap, fs, iter::Peekable, path::Path, slice::Iter};

use crate::{
    Definition, ExpressionEvaluator, ExpressionParser, InputFiles, Instruction, Label, LabelInfo,
    LabelManager, LabelType, LabelValue, Lexer, Macro, Token, TokenData, TokenType, ValueType,
};

use super::{
    DefinitionError, DefinitionExpansionResult, MacroError, MacroExpansionResult, PreprocessError,
    PreprocessResult,
};

pub struct DefinitionTable {
    definitions: HashMap<String, Definition>,
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
    macro_invocation_map: HashMap<String, u32>,
}

impl Preprocessor {
    pub fn new(include_path: String) -> Preprocessor {
        Preprocessor {
            include_path,
            macro_invocation_map: HashMap::new(),
        }
    }

    pub fn process(
        &mut self,
        settings: &PreprocessorSettings,
        input: Vec<Token>,
        definition_table: &mut DefinitionTable,
        macro_table: &mut MacroTable,
        label_manager: &mut LabelManager,
        input_files: &mut InputFiles,
    ) -> PreprocessResult<Vec<Token>> {
        let mut new_tokens = Vec::with_capacity(input.len());

        let mut token_iter = input.iter().peekable();

        while token_iter.peek().is_some() {
            let token = token_iter.peek().unwrap();

            if token.tt() == TokenType::DIRECTIVE {
                let directive = DirectiveType::from_str(match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                })?;
                let directive_line = token.line();
                token_iter.next();

                match directive {
                    DirectiveType::DEFINE => {
                        let definition = Definition::parse_definition(&mut token_iter)?;

                        if macro_table.ifdef(&definition.id()) {
                            return Err(PreprocessError::DefinitionNameCollision(
                                definition.id(),
                                directive_line,
                            )
                            .into());
                        }

                        definition_table.def(&definition.id(), definition);
                    }
                    DirectiveType::MACRO => {
                        let parsed_macro = Macro::parse_macro(directive_line, &mut token_iter)?;
                        let final_macro;
                        let final_contents;
                        let macro_settings = PreprocessorSettings {
                            expand_definitions: false,
                            expand_macros: true,
                        };

                        if definition_table.ifdef(&parsed_macro.id()) {
                            return Err(PreprocessError::MacroNameCollision(
                                parsed_macro.id(),
                                directive_line,
                            )
                            .into());
                        }

                        final_contents = self.process(
                            &macro_settings,
                            parsed_macro.contents_cloned(),
                            definition_table,
                            macro_table,
                            label_manager,
                            input_files,
                        )?;

                        final_macro = Macro::new(
                            &parsed_macro.id(),
                            final_contents,
                            parsed_macro.args_cloned(),
                            parsed_macro.num_required_args(),
                        );

                        macro_table.def(&parsed_macro.id(), final_macro);
                    }
                    DirectiveType::REP => {
                        let times;
                        let mut body = Vec::new();
                        let mut body_iter;
                        let mut processed_body = Vec::new();
                        let mut was_last_newline = false;

                        // Now we need to read the amount of times that this is to be expanded
                        if token_iter.peek().is_some() {
                            let times_token = token_iter.next().unwrap();

                            // At this time, only ints are allowed
                            if times_token.tt() != TokenType::INT {
                                return Err(PreprocessError::InvalidDirectiveTokenType(
                                    times_token.as_str(),
                                    String::from(".times"),
                                    directive_line,
                                )
                                .into());
                            }

                            // Set the number of times accordingly
                            times = match times_token.data() {
                                TokenData::INT(i) => *i,
                                _ => unreachable!(),
                            };
                        } else {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("number of times"),
                                String::from(".rep"),
                                directive_line,
                            )
                            .into());
                        }

                        // We will break out of this other ways
                        while token_iter.peek().is_some() {
                            let rep_token = token_iter.next().unwrap();

                            // .endrep directives not on a newline will not be recognized.
                            if rep_token.tt() == TokenType::DIRECTIVE
                                && was_last_newline
                                && "endrep"
                                    == match rep_token.data() {
                                        TokenData::STRING(s) => s,
                                        _ => unreachable!(),
                                    }
                            {
                                break;
                            } else {
                                body.push(rep_token.clone());
                            }

                            // If this was a newline, set the variable for next time
                            was_last_newline = rep_token.tt() == TokenType::NEWLINE;
                        }

                        // If this loop ended because we ran out of tokens, that is a problem
                        if token_iter.peek().is_none() {
                            return Err(PreprocessError::EndedWithoutClosing(
                                String::from("Rep"),
                                String::from(".endrep"),
                                directive_line,
                            )
                            .into());
                        }

                        // Now we preprocess each line before we expand it
                        body_iter = body.iter().peekable();

                        // While there are more tokens
                        while body_iter.peek().is_some() {
                            // Process the line
                            let mut processed_line = self.process_line(
                                &settings,
                                &mut body_iter,
                                definition_table,
                                macro_table,
                                label_manager,
                            )?;
                            // Add the line to processed_tokens
                            processed_body.append(&mut processed_line);
                        }

                        // Now we need to perform the repitition
                        for _ in 0..times {
                            for p_token in processed_body.iter() {
                                new_tokens.push(p_token.clone());
                            }
                        }
                    }
                    DirectiveType::INCLUDE => {
                        let include_file;
                        let included;
                        let mut included_preprocessed;

                        // After .include there must be a string.
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::STRING
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("string"),
                                String::from(".include"),
                                directive_line,
                            )
                            .into());
                        }

                        // Get the file name or path
                        include_file = match token_iter.next().unwrap().data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // Also make sure there is nothing else
                        if token_iter.peek().is_some() {
                            if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                                return Err(PreprocessError::ExtraTokensAfterDirective(
                                    String::from(".include"),
                                    directive_line,
                                )
                                .into());
                            }

                            // Consume the newline
                            token_iter.next();
                        }

                        // Read the file and lex it
                        included = self.include_file(include_file, input_files)?;

                        // Now preprocess it like everything else
                        included_preprocessed = self.process(
                            &settings,
                            included,
                            definition_table,
                            macro_table,
                            label_manager,
                            input_files,
                        )?;

                        // Add it to the preprocessed tokens
                        new_tokens.append(&mut included_preprocessed);
                    }
                    DirectiveType::EXTERN => {
                        let id;

                        // There must be something after this, and it must be an identifier
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::IDENTIFIER
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("identifer"),
                                String::from(".extern"),
                                directive_line,
                            )
                            .into());
                        }

                        // Read in the identifier
                        id = match token_iter.next().unwrap().data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // Also make sure there is nothing else
                        if token_iter.peek().is_some() {
                            if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                                return Err(PreprocessError::ExtraTokensAfterDirective(
                                    String::from(".extern"),
                                    directive_line,
                                )
                                .into());
                            }

                            // Consume the newline
                            token_iter.next();
                        }

                        // Test if it is already in the Label table
                        if label_manager.ifdef(id) {
                            return Err(PreprocessError::DuplicateLabel(
                                id.to_owned(),
                                directive_line,
                            )
                            .into());
                        }

                        // Then define it
                        label_manager.def(
                            id,
                            Label::new(id, LabelType::UNDEF, LabelInfo::EXTERN, LabelValue::NONE),
                        );
                    }
                    DirectiveType::GLOBAL => {
                        let id;

                        // There must be something after this, and it must be an identifier
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::IDENTIFIER
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("identifier"),
                                String::from(".extern"),
                                directive_line,
                            )
                            .into());
                        }

                        // Read in the identifier
                        id = match token_iter.next().unwrap().data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // Also make sure there is nothing else
                        if token_iter.peek().is_some() {
                            if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                                return Err(PreprocessError::ExtraTokensAfterDirective(
                                    String::from(".global"),
                                    directive_line,
                                )
                                .into());
                            }

                            // Consume the newline
                            token_iter.next();
                        }

                        // Test if it is already in the Label table
                        if label_manager.ifdef(id) {
                            let found_label = match label_manager.get(id) {
                                Some(label) => label,
                                None => unreachable!(),
                            };
                            let found_label_type = found_label.label_type();
                            let found_label_value = found_label.label_value().clone();
                            // If it is, we need to test if it is external or global, which would make no sense
                            if found_label.label_info() == LabelInfo::EXTERN
                                || found_label.label_info() == LabelInfo::GLOBAL
                            {
                                return Err(PreprocessError::DuplicateLabel(
                                    id.to_owned(),
                                    directive_line,
                                )
                                .into());
                            }

                            // All this needs to do then is to modify the Label to make it global
                            label_manager.def(
                                id,
                                Label::new(
                                    id,
                                    found_label_type,
                                    LabelInfo::GLOBAL,
                                    found_label_value,
                                ),
                            );
                        }
                        // If it isn't
                        else {
                            // Then define it
                            label_manager.def(
                                id,
                                Label::new(
                                    id,
                                    LabelType::UNDEF,
                                    LabelInfo::GLOBAL,
                                    LabelValue::NONE,
                                ),
                            );
                        }
                    }
                    DirectiveType::LINE => {
                        return Err(PreprocessError::DirectiveCurrentlyUnsupported(
                            String::from(".line"),
                            directive_line,
                        )
                        .into());
                    }
                    DirectiveType::UNDEF => {
                        let id;

                        // There must be something after this, and it must be an identifier
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::IDENTIFIER
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("identifier"),
                                String::from(".undef"),
                                directive_line,
                            )
                            .into());
                        }

                        // Read in the identifier
                        id = match token_iter.next().unwrap().data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // Also make sure there is nothing else
                        if token_iter.peek().is_some() {
                            if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                                return Err(PreprocessError::ExtraTokensAfterDirective(
                                    String::from(".undef"),
                                    directive_line,
                                )
                                .into());
                            }

                            // Consume the newline
                            token_iter.next();
                        }

                        // Test if it is in the definition table
                        if definition_table.ifndef(id) {
                            return Err(PreprocessError::CannotUndefine(
                                id.to_owned(),
                                directive_line,
                            )
                            .into());
                        }

                        // If it is in there, undefine it
                        definition_table.undef(id);
                    }
                    DirectiveType::UNMACRO => {
                        let id;

                        // There must be something after this, and it must be an identifier
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::IDENTIFIER
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("identifier"),
                                String::from(".unmacro"),
                                directive_line,
                            )
                            .into());
                        }

                        // Read in the identifier
                        id = match token_iter.next().unwrap().data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // Also make sure there is nothing else
                        if token_iter.peek().is_some() {
                            if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                                return Err(PreprocessError::ExtraTokensAfterDirective(
                                    String::from(".unmacro"),
                                    directive_line,
                                )
                                .into());
                            }

                            // Consume the newline
                            token_iter.next();
                        }

                        // Test if it is in the macro table
                        if macro_table.ifndef(id) {
                            return Err(PreprocessError::CannotUnmacro(
                                id.to_owned(),
                                directive_line,
                            )
                            .into());
                        }

                        // If it is in there, undefine it
                        macro_table.undef(id);
                    }
                    DirectiveType::IF
                    | DirectiveType::IFDEF
                    | DirectiveType::IFN
                    | DirectiveType::IFNDEF => {
                        // Process the if(s)
                        let if_tokens = self.process_if(
                            settings,
                            &mut token_iter,
                            definition_table,
                            macro_table,
                            label_manager,
                            directive,
                            directive_line,
                        )?;

                        // Now we need to actually preprocess the input
                        let mut preprocessed_if = self.process(
                            settings,
                            if_tokens,
                            definition_table,
                            macro_table,
                            label_manager,
                            input_files,
                        )?;

                        // No matter what was returned, as long as it wasn't an error, append it
                        new_tokens.append(&mut preprocessed_if);
                    }
                    DirectiveType::FUNC => {
                        // This is meant to register the following label as a function
                        let func_label_id;
                        let label_token;
                        let func_label;
                        let label_info;

                        // There must be something after this, and it must be a newline
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::NEWLINE
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("newline"),
                                String::from(".func"),
                                directive_line,
                            )
                            .into());
                        }

                        // Consume the newline
                        token_iter.next();

                        // Now we need to find the label, which needs to be there
                        if token_iter.peek().is_none()
                            || token_iter.peek().unwrap().tt() != TokenType::LABEL
                        {
                            return Err(PreprocessError::ExpectedAfterDirective(
                                String::from("function label"),
                                String::from(".func"),
                                directive_line,
                            )
                            .into());
                        }

                        // Now store the label token which we will push later
                        label_token = token_iter.next().unwrap();

                        // Now we need to extract the function label
                        func_label_id = match label_token.data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // If this function was declared as global, it will already be in the Label table
                        // If it is in the Label table and it isn't global though, it is a duplicate
                        if label_manager.ifdef(func_label_id) {
                            // Retrieve it
                            let declared_label = label_manager.get(func_label_id).unwrap();

                            // Check if it isn't global
                            if declared_label.label_info() != LabelInfo::GLOBAL {
                                // Then it is a duplicate
                                return Err(PreprocessError::DuplicateLabel(
                                    func_label_id.to_owned(),
                                    directive_line,
                                )
                                .into());
                            }
                            // If it is global, then we need to make the new Label info global as well
                            else {
                                label_info = LabelInfo::GLOBAL;
                            }
                        }
                        // If it isn't already declared, then the new Label's info will be local
                        else {
                            label_info = LabelInfo::LOCAL;
                        }

                        // Now we need to make a Label for it
                        func_label = Label::new(
                            func_label_id,
                            LabelType::UNDEFFUNC,
                            label_info,
                            LabelValue::NONE,
                        );

                        // Now register it in the Label table
                        label_manager.def(func_label_id, func_label);

                        // Now we need to push a "function token" so that the parser knows that this is declaring a new function and not just referencing it
                        new_tokens.push(Token::new(TokenType::FUNCTION, TokenData::NONE));

                        // Finally, push back the label token as it is needed for parsing
                        new_tokens.push(label_token.clone());
                    }
                    _ => unreachable!(),
                }
            } else {
                let mut append = self.process_line(
                    &settings,
                    &mut token_iter,
                    definition_table,
                    macro_table,
                    label_manager,
                )?;

                new_tokens.append(&mut append);
            }
        }

        Ok(new_tokens)
    }

    pub fn process_if(
        &mut self,
        settings: &PreprocessorSettings,
        token_iter: &mut Peekable<Iter<Token>>,
        definition_table: &mut DefinitionTable,
        macro_table: &mut MacroTable,
        label_manager: &mut LabelManager,
        if_type: DirectiveType,
        directive_line: usize,
    ) -> PreprocessResult<Vec<Token>> {
        let mut is_true;
        let mut new_tokens: Vec<Token> = Vec::new();
        let mut if_ended = false;

        // This will be fun...
        // Here we must support expressions

        // Evaluate the if expression based on the directive type
        is_true = self.evaluate_if(
            settings,
            token_iter,
            definition_table,
            macro_table,
            label_manager,
            if_type,
            directive_line,
        )?;

        while !if_ended {
            // If any of the if's is found to be true
            if is_true {
                let mut endif = false;
                let mut scope_level = 0;
                // Keep looping until we reach another if directive
                while token_iter.peek().is_some() && !if_ended {
                    let if_token = token_iter.next().unwrap();

                    // If this token is a directive
                    if if_token.tt() == TokenType::DIRECTIVE {
                        // Find the directive type for easier classification
                        let inner_directive = DirectiveType::from_str(match if_token.data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        })?;

                        match inner_directive {
                            // If it is an if directive
                            DirectiveType::IF
                            | DirectiveType::IFDEF
                            | DirectiveType::IFN
                            | DirectiveType::IFNDEF => {
                                scope_level += 1;
                            }
                            // If it is an if else
                            DirectiveType::ELIF
                            | DirectiveType::ELIFDEF
                            | DirectiveType::ELIFN
                            | DirectiveType::ELIFNDEF
                            | DirectiveType::ELSE => {
                                // And on our scope
                                if scope_level == 0 {
                                    // If we haven't reached the end yet, find the next one
                                    while self.find_next_if(token_iter, directive_line)?
                                        != DirectiveType::ENDIF
                                    {
                                    }

                                    // Now that we are done, end this outer loop and set endif to true
                                    endif = true;
                                    if_ended = true;
                                }
                                // If it isn't on our scope, push it because it is internal
                                else {
                                    new_tokens.push(if_token.clone());
                                }
                            }
                            // If this is an endif
                            DirectiveType::ENDIF => {
                                // Check if the scope is zero
                                if scope_level == 0 {
                                    // If so, this is the end of the if block
                                    // Set endif to true
                                    endif = true;
                                    if_ended = true;
                                }
                                // If it isn't on our scope, push it because it is internal
                                else {
                                    // Also subtract 1 from scope
                                    scope_level -= 1;
                                    new_tokens.push(if_token.clone());
                                }
                            }
                            // If it is anything else, just push it
                            _ => {
                                new_tokens.push(if_token.clone());
                            }
                        }
                    }
                    // If this isn't a directive, just push it because it is part of the contents
                    else {
                        new_tokens.push(if_token.clone());
                    }
                }

                // If this ended because we ran out of tokens, that is an error
                if !endif {
                    return Err(PreprocessError::EndedWithoutClosing(
                        String::from(".if"),
                        String::from(".endif"),
                        directive_line,
                    )
                    .into());
                }
            }
            // If it is false
            else {
                // We just find the next if
                let next_if = self.find_next_if(token_iter, directive_line)?;

                // If we reach an endif, then just end
                if next_if == DirectiveType::ENDIF {
                    // Let this loop end
                    if_ended = true;
                }
                // If it is an else, well that is always true if the past ones are false
                else if next_if == DirectiveType::ELSE {
                    is_true = true;
                }
                // If it is neither, then it must be an if
                else {
                    is_true = self.evaluate_if(
                        settings,
                        token_iter,
                        definition_table,
                        macro_table,
                        label_manager,
                        next_if,
                        directive_line,
                    )?;
                }
            }
        }

        println!("\tNew tokens:\n");

        for token in new_tokens.iter() {
            println!("\t\t{}", token.as_str());
        }

        Ok(new_tokens)
    }

    pub fn find_next_if(
        &self,
        token_iter: &mut Peekable<Iter<Token>>,
        directive_line: usize,
    ) -> PreprocessResult<DirectiveType> {
        let mut scope = 0;

        // The only time this should end is if there is a return
        while token_iter.peek().is_some() {
            // If the token is a directive
            if token_iter.peek().unwrap().tt() == TokenType::DIRECTIVE {
                // Find the directive type for easier classification
                let inner_directive_line = token_iter.peek().unwrap().line();
                let inner_directive =
                    DirectiveType::from_str(match token_iter.next().unwrap().data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    })?;

                match inner_directive {
                    // If it is an "if*"
                    DirectiveType::IF
                    | DirectiveType::IFDEF
                    | DirectiveType::IFN
                    | DirectiveType::IFNDEF => {
                        // Check if the scope is 0
                        if scope == 0 {
                            // This is actually an error
                            return Err(
                                PreprocessError::InvalidStartOfIf(inner_directive_line).into()
                            );
                        } else {
                            // Add 1 to the scope
                            scope += 1;
                        }
                    }
                    // If this is any other if directive
                    DirectiveType::ELSE
                    | DirectiveType::ENDIF
                    | DirectiveType::ELIF
                    | DirectiveType::ELIFN
                    | DirectiveType::ELIFDEF
                    | DirectiveType::ELIFNDEF => {
                        // check if the scope is 0
                        if scope == 0 {
                            // If it is, return it!
                            return Ok(inner_directive);
                        } else {
                            // If not, check if it is an endif
                            if inner_directive == DirectiveType::ENDIF {
                                // If it is an endif, subtract one from the scope
                                scope -= 1;
                            }
                        }
                    }
                    // If it is any other directive, then just skip it because it would be part of the "body"
                    _ => {}
                }
            }
            // If this token isn't a directive, still consume it
            else {
                token_iter.next();
            }
        }

        // If we exited the loop that means that we ran out of tokens.
        Err(PreprocessError::EndedWithoutClosing(
            String::from(".if"),
            String::from(".endif"),
            directive_line,
        )
        .into())
    }

    pub fn evaluate_if(
        &mut self,
        settings: &PreprocessorSettings,
        token_iter: &mut Peekable<Iter<Token>>,
        definition_table: &mut DefinitionTable,
        macro_table: &mut MacroTable,
        label_manager: &mut LabelManager,
        if_type: DirectiveType,
        directive_line: usize,
    ) -> PreprocessResult<bool> {
        let is_true;

        // If it is an else, that is easy, return true.
        if if_type == DirectiveType::ELSE {
            is_true = true;
        } else if if_type == DirectiveType::IF
            || if_type == DirectiveType::ELIF
            || if_type == DirectiveType::IFN
            || if_type == DirectiveType::ELIFN
        {
            let mut processed_line;
            let mut processed_line_iter;
            let parsed_expression;
            let evaluated_expression;

            // Process the line
            processed_line = self.process_line(
                settings,
                token_iter,
                definition_table,
                macro_table,
                label_manager,
            )?;

            // If the line is empty or has only one token, there is a problem
            if processed_line.len() < 2 {
                return Err(PreprocessError::ExpectedAfterDirective(
                    String::from("expression and newline"),
                    String::from(".if"),
                    directive_line,
                )
                .into());
            }

            // Check if the last token is a newline
            if processed_line.last().unwrap().tt() != TokenType::NEWLINE {
                return Err(PreprocessError::ExpectedAfterDirective(
                    String::from("newline"),
                    String::from(".if"),
                    directive_line,
                )
                .into());
            }

            // Remove the newline
            processed_line.pop();

            // Make an iterator for this line
            processed_line_iter = processed_line.iter().peekable();

            // Parse the rest as an expression
            parsed_expression = match ExpressionParser::parse_expression(&mut processed_line_iter) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(PreprocessError::ExpressionParseError(e, directive_line).into());
                }
            };

            // Evaluate the expression to see what the result is
            evaluated_expression = match ExpressionEvaluator::evaluate(&parsed_expression) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(
                        PreprocessError::ExpressionEvaluationError(e, directive_line).into(),
                    );
                }
            };

            // Check if it is a boolean, because we are enforcing that the expression results in a boolean value
            if evaluated_expression.valtype() != ValueType::BOOL {
                return Err(PreprocessError::InvalidExpressionResultType(
                    String::from("if directive"),
                    String::from("boolean"),
                    directive_line,
                )
                .into());
            }

            // Actually determine if this if is true, or not
            if if_type == DirectiveType::IF || if_type == DirectiveType::ELIF {
                is_true = evaluated_expression.to_bool();
            } else {
                is_true = !evaluated_expression.to_bool();
            }
        }
        // If this was an ifdef or ifndef
        else {
            // There should only be one token before a newline and it should be an identifier that is a macro or definition
            let id;

            // Check if the last token is an identifier
            if token_iter.peek().unwrap().tt() != TokenType::IDENTIFIER {
                return Err(PreprocessError::ExpectedAfterDirective(
                    String::from("identifier"),
                    String::from(".ifdef"),
                    directive_line,
                )
                .into());
            }

            // Collect the string
            id = match token_iter.next().unwrap().data() {
                TokenData::STRING(s) => s,
                _ => unreachable!(),
            };

            // Check if the last token is a newline
            if token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
                return Err(PreprocessError::ExpectedAfterDirective(
                    String::from("newline"),
                    String::from(".ifdef"),
                    directive_line,
                )
                .into());
            }

            // Consume the newline
            token_iter.next();

            // If it was an ifdef, everything is fine. But if it is an ifndef, flip the output
            if if_type == DirectiveType::IFDEF || if_type == DirectiveType::ELIFDEF {
                // Now just see if it is defined in either, if not, then it is not true
                is_true = definition_table.ifdef(id) || macro_table.ifdef(id);
            } else {
                is_true = !(definition_table.ifdef(id) || macro_table.ifdef(id));
            }
        }

        Ok(is_true)
    }

    pub fn process_line(
        &mut self,
        settings: &PreprocessorSettings,
        token_iter: &mut Peekable<Iter<Token>>,
        definition_table: &mut DefinitionTable,
        macro_table: &mut MacroTable,
        label_manager: &mut LabelManager,
    ) -> PreprocessResult<Vec<Token>> {
        let mut new_tokens = Vec::new();

        // While we haven't reached the end of the line
        while token_iter.peek().is_some() && token_iter.peek().unwrap().tt() != TokenType::NEWLINE {
            let token = token_iter.next().unwrap().clone();

            if token.tt() == TokenType::IDENTIFIER {
                let id = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };
                let line = token.line();

                // 0 means that it is not a valid mnemonic
                if Instruction::opcode_from_mnemonic(id) == 0 {
                    // If it is a defined macro
                    if macro_table.ifdef(id) {
                        // And if we want it to be expanded
                        if settings.expand_macros {
                            let mut expanded = match self.expand_macro(
                                id,
                                line,
                                token_iter,
                                definition_table,
                                macro_table,
                            ) {
                                Ok(expanded) => expanded,
                                Err(e) => {
                                    return Err(PreprocessError::MacroExpansionError(
                                        id.to_owned(),
                                        line,
                                        e,
                                    )
                                    .into());
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
                            let mut expanded =
                                match self.expand_definition(id, token_iter, definition_table) {
                                    Ok(expanded) => expanded,
                                    Err(e) => {
                                        return Err(PreprocessError::DefinitionExpansionError(
                                            id.to_owned(),
                                            line,
                                            e,
                                        )
                                        .into());
                                    }
                                };

                            new_tokens.append(&mut expanded);
                        }
                        // If not, just append it
                        else {
                            new_tokens.push(token);
                        }
                    }
                    // If it is a boolean value
                    else if id == "true" || id == "false" {
                        // Push it right along
                        new_tokens.push(token);
                    }
                    // We also need to check if it is an external Label
                    else if label_manager.ifdef(id) {
                        // If it is, it will be dealt with much later on down the line, so just push it
                        new_tokens.push(token);
                    }
                    // If it isn't a defined Label, then assume that it is a function label or something
                    else {
                        label_manager.def(
                            id,
                            Label::new(id, LabelType::UNDEF, LabelInfo::LOCAL, LabelValue::NONE),
                        );
                        // return Err(format!("Macro or definition used before declaration: {}, line {}.", id, line).into());

                        // If it is, this will also be dealt with much later on down the line, so just push it
                        new_tokens.push(token);
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

    pub fn expand_macro(
        &mut self,
        id: &str,
        line: usize,
        token_iter: &mut Peekable<Iter<Token>>,
        definition_table: &DefinitionTable,
        macro_table: &MacroTable,
    ) -> MacroExpansionResult<Vec<Token>> {
        let macro_ref = match macro_table.get(id) {
            Some(macro_ref) => macro_ref,
            None => {
                return Err(MacroError::MacroNotFound(id.to_owned()));
            }
        };
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
            while token_iter.peek().is_some()
                && token_iter.peek().unwrap().tt() != TokenType::COMMA
                && token_iter.peek().unwrap().tt() != TokenType::NEWLINE
            {
                let mut arg_tokens = Vec::new();

                // Get the token
                let token = token_iter.next().unwrap();

                // If this token is a definiition expansion
                if token.tt() == TokenType::IDENTIFIER && definition_table.ifdef(id) {
                    // Get the id
                    let inner_id = match token.data() {
                        TokenData::STRING(s) => s,
                        _ => unreachable!(),
                    };

                    // Expand it
                    let mut inner =
                        match self.expand_definition(inner_id, token_iter, definition_table) {
                            Ok(inner) => inner,
                            Err(e) => {
                                return Err(MacroError::InnerDefinitionExpansionError(
                                    inner_id.to_owned(),
                                    token.line(),
                                    e,
                                ));
                            }
                        };

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
            return Err(MacroError::InvalidNumberOfArgumentsProvided(
                args.len(),
                line,
                num_required_args,
            ));
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
                let arg_index = match token.data() {
                    TokenData::INT(i) => *i as usize,
                    _ => unreachable!(),
                };

                // Replace it with the tokens of that argument
                for token in args.get(arg_index - 1).unwrap() {
                    without_placeholders.push(token.clone());
                }
            }
            // If it is a local label, then we need to prepend the invocation id
            else if token.tt() == TokenType::INNERLABEL {
                // Extract the local label's name
                let label_name = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };
                // Create the combined name
                let new_name = format!("{}.{}", invocation_id, label_name);

                // So this acts correctly in global contexts, make it a regular label
                without_placeholders
                    .push(Token::new(TokenType::LABEL, TokenData::STRING(new_name)));
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
                let inner_id = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };
                // Get the line for an error
                let inner_line = token.line();

                if definition_table.ifdef(inner_id) {
                    // Expand it
                    let mut inner = match self.expand_definition(
                        inner_id,
                        &mut without_placeholders_iter,
                        definition_table,
                    ) {
                        Ok(inner) => inner,
                        Err(e) => {
                            return Err(MacroError::InnerDefinitionExpansionError(
                                inner_id.to_owned(),
                                inner_line,
                                e,
                            ));
                        }
                    };

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

    pub fn expand_definition(
        &self,
        id: &str,
        token_iter: &mut Peekable<Iter<Token>>,
        definition_table: &DefinitionTable,
    ) -> DefinitionExpansionResult<Vec<Token>> {
        let def_ref = match definition_table.get(id) {
            Some(r) => r,
            None => return Err(DefinitionError::DefinitionNotFound(id.to_owned())),
        };
        let mut content_iter = def_ref.get_contents_iter();
        let mut without_placeholders = Vec::new();
        let mut without_placeholders_iter;
        let mut expanded_tokens = Vec::new();
        let mut args = Vec::new();

        // We can't really expand a definition if it is in fact, empty
        if def_ref.is_empty() {
            return Err(DefinitionError::EmptyDefinition(id.to_owned()));
        }

        // If the next token is an open parenthesis, then we have some arguments
        if token_iter.peek().is_some() && token_iter.peek().unwrap().tt() == TokenType::OPENPAREN {
            // This will help us collect all of the arguments, because all we need to look for is the closing parenthesis
            while token_iter.peek().is_some()
                && token_iter.peek().unwrap().tt() != TokenType::CLOSEPAREN
            {
                let mut arg_tokens = Vec::new();

                // Consume either the opening parenthesis, or the preceeding comma
                token_iter.next();

                // We want to collect all tokens before there is a comma, or close parenthesis.
                while token_iter.peek().is_some()
                    && token_iter.peek().unwrap().tt() != TokenType::COMMA
                    && token_iter.peek().unwrap().tt() != TokenType::CLOSEPAREN
                {
                    // Get the token
                    let token = token_iter.next().unwrap();

                    // If this token is another definiition expansion...
                    if token.tt() == TokenType::IDENTIFIER && definition_table.ifdef(id) {
                        // Get the id
                        let inner_id = match token.data() {
                            TokenData::STRING(s) => s,
                            _ => unreachable!(),
                        };

                        // Expand it
                        let mut inner =
                            self.expand_definition(inner_id, token_iter, definition_table)?;

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
                Some(_token) => {}
                None => {
                    return Err(DefinitionError::EndedWithoutAllArgs);
                }
            }
        }

        // Before we go on we need to test if we have the correct number of arguments
        if def_ref.num_args() != args.len() {
            return Err(DefinitionError::InvalidNumberOfArgumentsProvided(
                args.len(),
                def_ref.num_args(),
            ));
        }

        // Now that we are done with the arguments if there were any, we can begin expansion

        // First though, we need to replace all of the placeholders with the argument values
        while content_iter.peek().is_some() {
            let token = content_iter.next().unwrap();

            // If the token is a placeholder
            if token.tt() == TokenType::PLACEHOLDER {
                let arg_index = match token.data() {
                    TokenData::INT(i) => *i as usize,
                    _ => unreachable!(),
                };

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
                let inner_id = match token.data() {
                    TokenData::STRING(s) => s,
                    _ => unreachable!(),
                };

                // We don't want recursive expansion, that would be bad.
                if *id == *inner_id {
                    return Err(DefinitionError::RecursiveExpansion);
                }

                // Expand it
                let mut inner = self.expand_definition(
                    inner_id,
                    &mut without_placeholders_iter,
                    definition_table,
                )?;

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

    pub fn include_file(
        &self,
        file_path: &str,
        input_files: &mut InputFiles,
    ) -> PreprocessResult<Vec<Token>> {
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
            return Err(PreprocessError::InvalidIncludeFile(
                file_path.to_str().unwrap().to_owned(),
            ));
        }

        // If the file isn't a file, there is a problem
        if !file_path.is_file() {
            return Err(PreprocessError::DirectoryIncludeError(
                file_path.to_str().unwrap().to_owned(),
            ));
        }

        // If it is an absolute path, we don't need to do anything. If it is not, make it one by adding the include path
        if !file_path.is_absolute() {
            let include_path = Path::new(&self.include_path);
            path_buffer = include_path.join(file_path);
            file_path = path_buffer.as_path();
        }

        // Attempt to read the file as text
        contents = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                return Err(PreprocessError::UnableToReadFile(
                    file_path.to_str().unwrap().to_owned(),
                    e.into(),
                ));
            }
        };

        // Add this file to the inputfiles
        file_name = String::from(file_path.file_name().unwrap().to_str().unwrap());
        file_id = input_files.add_file(&file_name);

        // Lex the contents
        tokens = match lexer.lex(&contents, &file_name, file_id) {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err(PreprocessError::IncludedLexError(file_name.to_owned()));
            }
        };

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

    pub fn get(&self, identifier: &str) -> Option<&Definition> {
        self.definitions.get(identifier)
    }
}

impl MacroTable {
    pub fn new() -> MacroTable {
        MacroTable {
            macros: HashMap::new(),
        }
    }

    pub fn def(&mut self, identifier: &str, new_macro: Macro) {
        // This already does what it needs to do. If it exists, the value is updated, if not, the value is created.
        self.macros.insert(String::from(identifier), new_macro);
    }

    pub fn ifdef(&self, identifier: &str) -> bool {
        self.macros.contains_key(identifier)
    }

    pub fn ifndef(&self, identifier: &str) -> bool {
        !self.macros.contains_key(identifier)
    }

    pub fn undef(&mut self, identifier: &str) {
        self.macros.remove(identifier);
    }

    pub fn get(&self, identifier: &str) -> Option<&Macro> {
        self.macros.get(identifier)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DirectiveType {
    DEFINE,
    UNDEF,
    MACRO,
    // ENDMACRO,
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
    // ENDREP,
    INCLUDE,
    LINE,
    EXTERN,
    GLOBAL,
    FUNC,
}

impl DirectiveType {
    pub fn from_str(s: &str) -> PreprocessResult<DirectiveType> {
        Ok(match s {
            "define" => DirectiveType::DEFINE,
            "undef" => DirectiveType::UNDEF,
            "macro" => DirectiveType::MACRO,
            "unmacro" => DirectiveType::UNMACRO,
            // "endmacro" => DirectiveType::ENDMACRO,
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
            // "endrep" => DirectiveType::ENDREP,
            "include" => DirectiveType::INCLUDE,
            "line" => DirectiveType::LINE,
            "extern" => DirectiveType::EXTERN,
            "global" => DirectiveType::GLOBAL,
            "func" => DirectiveType::FUNC,
            _ => return Err(PreprocessError::InvalidDirective(s.to_owned())),
        })
    }
}
