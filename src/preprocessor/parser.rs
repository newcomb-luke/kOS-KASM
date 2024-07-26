#![allow(clippy::result_unit_err)]

use std::{collections::hash_map::DefaultHasher, hash::Hasher, num::NonZeroU8};

use kerbalobjects::Opcode;

type PResult<T> = Result<T, ()>;

// Only used in the parsing of a number, but it is useful nonetheless
type NumPResult<'a> = Result<(Span, i32), (DiagnosticBuilder<'a>, Option<(String, Token)>)>;

use crate::{
    errors::{DiagnosticBuilder, Span},
    lexer::{Directive, Literal, PreprocessorDirective, Token, TokenKind},
    session::Session,
};

use super::past::{
    Ident, IfStatement, MLMacroDef, SLMacroDef, IfClause, IfClauseBegin, IfCondition, IfDefCondition, IfExpCondition, Include, IncludePath, InertTokens, MLMacroArgs, MLMacroDefDefaults, MLMacroUndef, MacroInvocation, MacroInvocationArg, MacroInvocationArgs, PASTNode, Repeat, RepeatNumber, SLMacroDefArgs, SLMacroDefContents, SLMacroUndef, SLMacroUndefArgs
};

/// The parser for the preprocessor, which turns tokenized source code into preprocessable PASTNodes
pub struct Parser<'a> {
    tokens: Vec<Token>,
    token_cursor: usize,
    session: &'a Session,
    last_token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, session: &'a Session) -> Self {
        let first_token = tokens.get(0).copied();

        Self {
            tokens,
            token_cursor: 0,
            session,
            last_token: first_token,
        }
    }

    pub fn parse(mut self) -> PResult<Vec<PASTNode>> {
        let mut nodes = Vec::new();

        while self.peek_next().is_some() {
            // Parse only one for now
            let node = self.parse_node()?;
            nodes.push(node);
        }

        Ok(nodes)
    }

    // This usually parses a line, but in the case of any multi-line construct, this parses more
    // than that
    //
    // This assumes that there is a next token to parse, so keep that in mind
    //
    fn parse_node(&mut self) -> PResult<PASTNode> {
        let next = *self.peek_next().unwrap();

        match next.kind {
            TokenKind::Directive(Directive::Preprocessor(PreprocessorDirective::Include)) => self.parse_include(),
            t => {
                panic!("Reached token that cannot yet be parsed: {t:#?}");
            }
            // TokenKind::DirectiveDefine => self.parse_sl_macro_def(),
            // TokenKind::DirectiveMacro => self.parse_ml_macro_def(),
            // TokenKind::DirectiveUndef => self.parse_sl_macro_undef(),
            // TokenKind::DirectiveUnmacro => self.parse_ml_macro_undef(),
            // TokenKind::DirectiveRepeat => self.parse_repeat(),
            // TokenKind::DirectiveInclude => self.parse_include(),
            // TokenKind::DirectiveIf
            // | TokenKind::DirectiveIfNot
            // | TokenKind::DirectiveIfDef
            // | TokenKind::DirectiveIfNotDef => self.parse_if_statement(next, true, true),
            // TokenKind::DirectiveElseIf
            // | TokenKind::DirectiveElse
            // | TokenKind::DirectiveElseIfDef
            // | TokenKind::DirectiveElseIfNot
            // | TokenKind::DirectiveElseIfNotDef
            // | TokenKind::DirectiveEndIf => {
            //     self.session
            //         .struct_span_error(
            //             next.as_span(),
            //             "If directive with no previous .if".to_string(),
            //         )
            //         .emit();

            //     Err(())
            // }
            // TokenKind::Identifier => {
            //     let snippet = self.session.span_to_snippet(&next.as_span());
            //     let ident_str = snippet.as_slice();
            //     self.consume_next();

            //     // Tests if this is an instruction or not
            //     if Opcode::from(ident_str) != Opcode::Bogus {
            //         // If it is, we parse it as such
            //         self.parse_benign_tokens(next)
            //     } else {
            //         // If it isn't, it is going to be parsed as a macro invocation
            //         let macro_invocation = self.parse_macro_invocation(next.as_span(), ident_str)?;

            //         // If we have captured any tokens before this
            //         // Update this just in case it is the last part of the contents
            //         Ok(PASTNode::MacroInvocation(macro_invocation))
            //     }
            // }
            // _ => {
            //     self.consume_next();
            //     self.parse_benign_tokens(next)
            // }
        }
    }

    // Parse an include directive. See the Include grammar
    fn parse_include(&mut self) -> PResult<PASTNode> {
        // Consume the .include
        let mut span = self.assert_next_preprocessor(PreprocessorDirective::Include)?;

        // Skip any whitespace
        self.skip_whitespace();

        // We now require the actual include path
        if let Some(&token) = self.peek_next() {
            let path_span = token.as_span();
            span.end = path_span.end;

            let path = match token.kind {
                TokenKind::Literal(Literal::String) => {
                    self.assert_next(TokenKind::Literal(Literal::String))?;

                    let path_string = self.string_literal_to_string(token)?;
                    IncludePath::Literal(path_span, path_string)
                },
                TokenKind::Identifier => {
                    self.assert_next(TokenKind::Identifier)?;

                    let macro_invocation = self.parse_macro_invocation()?;
                    IncludePath::MacroInvocation(macro_invocation)
                },
                _ => {
                    self.struct_err_expected_found(token.as_span(), "include path (string literal or macro invocation)").emit();
                    return Err(());
                }
            };

            // We need a newline after our path
            self.skip_whitespace();
            self.expect_newline_or_eof(".include path")?;

            Ok(PASTNode::Include(Include::new(span, path)))
        } else {
            self.session
                .struct_span_error(span, ".include requires a path".to_string())
                .emit();

            Err(())
        }
    }

    // Parses a macro invocation. See the MacroInvocation grammar
    fn parse_macro_invocation(&mut self) -> PResult<MacroInvocation> {
        unimplemented!()
    }

    // // Parses an if statement directive
    // //
    // // See the IfStatement grammar
    // //
    // // The consume_first flag determines if this function should consume the next token or not as
    // // the beginning directive.
    // //
    // // The allow_preprocessor flag determines if preprocessor directives will be allowed or not
    // //
    // fn parse_if_statement(
    //     &mut self,
    //     mut token: Token,
    //     consume_first: bool,
    //     allow_preprocessor: bool,
    // ) -> PResult<PASTNode> {
    //     if consume_first {
    //         // Consume the .if*
    //         token = *self.consume_next().unwrap();
    //     }

    //     let mut clauses = Vec::new();
    //     let mut else_encountered = false;

    //     let (first_clause, end_kind) = self.parse_if_clause(token, allow_preprocessor)?;
    //     clauses.push(first_clause);

    //     if end_kind != TokenKind::DirectiveEndIf {
    //         if end_kind == TokenKind::DirectiveElse {
    //             else_encountered = true;
    //         }

    //         token = *self.consume_next().unwrap();

    //         loop {
    //             // Parse the clause
    //             let (if_clause, end_kind) = self.parse_if_clause(token, allow_preprocessor)?;

    //             if else_encountered && !matches!(if_clause.condition, IfCondition::Else) {
    //                 self.session
    //                     .struct_span_error(
    //                         if_clause.begin.span,
    //                         ".endif expected after .else clause".to_string(),
    //                     )
    //                     .emit();

    //                 return Err(());
    //             }

    //             // Add it
    //             clauses.push(if_clause);

    //             // If it isn't the end, set the next token
    //             if end_kind != TokenKind::DirectiveEndIf {
    //                 if end_kind == TokenKind::DirectiveElse {
    //                     else_encountered = true;
    //                 }

    //                 token = *self.consume_next().unwrap();
    //             } else {
    //                 break;
    //             }
    //         }
    //     }

    //     // Consume the .endif
    //     self.assert_next(TokenKind::DirectiveEndIf)?;

    //     Ok(PASTNode::IfStatement(IfStatement::from_vec(clauses)))
    // }

    // // Parses 0 or more "benign tokens" from the token stream
    // //
    // // "Benign tokens" are tokens that are non-preprocessor, things like normal identifiers,
    // // integer literals, etc.
    // //
    // fn parse_benign_tokens(&mut self, start: Token) -> PResult<PASTNode> {
    //     let mut tokens = vec![start];

    //     while let Some(&next) = self.peek_next() {
    //         match next.kind {
    //             TokenKind::DirectiveDefine
    //             | TokenKind::DirectiveUndef
    //             | TokenKind::DirectiveMacro
    //             | TokenKind::DirectiveEndmacro
    //             | TokenKind::DirectiveUnmacro
    //             | TokenKind::DirectiveRepeat
    //             | TokenKind::DirectiveEndRepeat
    //             | TokenKind::DirectiveInclude
    //             | TokenKind::DirectiveIf
    //             | TokenKind::DirectiveIfDef
    //             | TokenKind::DirectiveIfNot
    //             | TokenKind::DirectiveIfNotDef
    //             | TokenKind::DirectiveElse
    //             | TokenKind::DirectiveElseIf
    //             | TokenKind::DirectiveElseIfDef
    //             | TokenKind::DirectiveElseIfNot
    //             | TokenKind::DirectiveElseIfNotDef
    //             | TokenKind::DirectiveEndIf => {
    //                 break;
    //             }
    //             TokenKind::Identifier => {
    //                 let snippet = self.session.span_to_snippet(&next.as_span());
    //                 let ident_str = snippet.as_slice();

    //                 // Tests if this is an instruction or not
    //                 if Opcode::from(ident_str) != Opcode::Bogus {
    //                     // It is, which is "benign"
    //                     tokens.push(next);

    //                     self.consume_next();
    //                 } else {
    //                     break;
    //                 }
    //             }
    //             _ => {
    //                 tokens.push(next);
    //                 self.consume_next();
    //             }
    //         }
    //     }

    //     Ok(PASTNode::BenignTokens(BenignTokens::from_vec(tokens)))
    // }

    // // Parses an if statement clause
    // //
    // // See the IfClause grammar
    // //
    // // The if_token is passed in to parse the if clause type and condition
    // //
    // // The allow_preprocessor flag determines if preprocessor directives will be allowed or not
    // //
    // fn parse_if_clause(
    //     &mut self,
    //     if_token: Token,
    //     allow_preprocessor: bool,
    // ) -> PResult<(IfClause, TokenKind)> {
    //     let mut span = Span::new(0, 0, 0);
    //     let begin = self.parse_if_clause_begin(if_token)?;
    //     let condition = self.parse_if_condition(if_token)?;
    //     let mut contents = Vec::new();
    //     let mut end_kind = TokenKind::Error;

    //     span.start = begin.span.start;
    //     span.file = begin.span.file;

    //     // Two different loops is pretty bad, but it avoids checking the allow_preprocessor flag
    //     // every loop
    //     if allow_preprocessor {
    //         while let Some(&next) = self.peek_next() {
    //             let node = match next.kind {
    //                 TokenKind::DirectiveDefine => self.parse_sl_macro_def(),
    //                 TokenKind::DirectiveMacro => self.parse_ml_macro_def(),
    //                 TokenKind::DirectiveUndef => self.parse_sl_macro_undef(),
    //                 TokenKind::DirectiveUnmacro => self.parse_ml_macro_undef(),
    //                 TokenKind::DirectiveRepeat => self.parse_repeat(),
    //                 TokenKind::DirectiveInclude => self.parse_include(),
    //                 TokenKind::DirectiveIf
    //                 | TokenKind::DirectiveIfNot
    //                 | TokenKind::DirectiveIfDef
    //                 | TokenKind::DirectiveIfNotDef => self.parse_if_statement(next, true, true),
    //                 TokenKind::DirectiveEndIf
    //                 | TokenKind::DirectiveElse
    //                 | TokenKind::DirectiveElseIf
    //                 | TokenKind::DirectiveElseIfDef
    //                 | TokenKind::DirectiveElseIfNot
    //                 | TokenKind::DirectiveElseIfNotDef => {
    //                     end_kind = next.kind;
    //                     break;
    //                 }
    //                 TokenKind::Identifier => {
    //                     let snippet = self.session.span_to_snippet(&next.as_span());
    //                     let ident_str = snippet.as_slice();
    //                     self.consume_next();

    //                     // Tests if this is an instruction or not
    //                     if Opcode::from(ident_str) != Opcode::Bogus {
    //                         // If it is, we parse it as such
    //                         self.parse_benign_tokens(next)
    //                     } else {
    //                         // If it isn't, it is going to be parsed as a macro invocation
    //                         let macro_invocation = self.parse_macro_invocation(next.as_span(), ident_str)?;

    //                         // If we have captured any tokens before this
    //                         // Update this just in case it is the last part of the contents
    //                         Ok(PASTNode::MacroInvocation(macro_invocation))
    //                     }
    //                 }
    //                 _ => {
    //                     self.consume_next();
    //                     self.parse_benign_tokens(next)
    //                 }
    //             }?;

    //             span.end = node.span_end();

    //             contents.push(node);
    //         }
    //     } else {
    //         while let Some(&next) = self.peek_next() {
    //             let node = match next.kind {
    //                 TokenKind::DirectiveDefine
    //                 | TokenKind::DirectiveMacro
    //                 | TokenKind::DirectiveEndmacro
    //                 | TokenKind::DirectiveUndef
    //                 | TokenKind::DirectiveUnmacro
    //                 | TokenKind::DirectiveRepeat
    //                 | TokenKind::DirectiveEndRepeat
    //                 | TokenKind::DirectiveInclude => {
    //                     self.session
    //                         .struct_span_error(
    //                             next.as_span(),
    //                             "preprocessor directives not allowed here".to_string(),
    //                         )
    //                         .emit();

    //                     return Err(());
    //                 }
    //                 TokenKind::DirectiveIf
    //                 | TokenKind::DirectiveIfNot
    //                 | TokenKind::DirectiveIfDef
    //                 | TokenKind::DirectiveIfNotDef => self.parse_if_statement(next, true, true),
    //                 TokenKind::DirectiveEndIf
    //                 | TokenKind::DirectiveElse
    //                 | TokenKind::DirectiveElseIf
    //                 | TokenKind::DirectiveElseIfDef
    //                 | TokenKind::DirectiveElseIfNot
    //                 | TokenKind::DirectiveElseIfNotDef => {
    //                     end_kind = next.kind;
    //                     break;
    //                 }
    //                 TokenKind::Identifier => {
    //                     let snippet = self.session.span_to_snippet(&next.as_span());
    //                     let ident_str = snippet.as_slice();

    //                     self.consume_next();

    //                     // Tests if this is an instruction or not
    //                     if Opcode::from(ident_str) != Opcode::Bogus {
    //                         // If it is, we parse it as such
    //                         self.parse_benign_tokens(next)
    //                     } else {
    //                         // If it isn't, it is going to be parsed as a macro invocation
    //                         let macro_invocation = self.parse_macro_invocation(next.as_span(), ident_str)?;

    //                         // If we have captured any tokens before this
    //                         // Update this just in case it is the last part of the contents
    //                         Ok(PASTNode::MacroInvocation(macro_invocation))
    //                     }
    //                 }
    //                 _ => {
    //                     self.consume_next();
    //                     self.parse_benign_tokens(next)
    //                 }
    //             }?;

    //             span.end = node.span_end();

    //             contents.push(node);
    //         }
    //     }

    //     // If we have ended by running out of tokens, but the last token isn't an endif
    //     if self.peek_next().is_none() && end_kind != TokenKind::DirectiveEndIf {
    //         // Error
    //         self.session
    //             .struct_error("if clause has no .endif".to_string())
    //             .span_label(if_token.as_span(), "this clause".to_string())
    //             .span_label(
    //                 self.last_token.unwrap().as_span(),
    //                 "file ended unexpectedly".to_string(),
    //             )
    //             .emit();

    //         return Err(());
    //     }

    //     Ok((IfClause::new(span, begin, condition, contents), end_kind))
    // }

    // fn parse_if_clause_begin(&mut self, if_token: Token) -> PResult<IfClauseBegin> {
    //     let inverse = !matches!(
    //         if_token.kind,
    //         TokenKind::DirectiveIf
    //             | TokenKind::DirectiveIfDef
    //             | TokenKind::DirectiveElseIf
    //             | TokenKind::DirectiveElseIfDef
    //             | TokenKind::DirectiveElse
    //     );

    //     let span = if_token.as_span();

    //     Ok(IfClauseBegin::new(span, inverse))
    // }

    // fn parse_if_condition(&mut self, if_token: Token) -> PResult<IfCondition> {
    //     Ok(match if_token.kind {
    //         TokenKind::DirectiveIfDef
    //         | TokenKind::DirectiveIfNotDef
    //         | TokenKind::DirectiveElseIfDef
    //         | TokenKind::DirectiveElseIfNotDef => IfCondition::Def(self.parse_if_def_condition()?),
    //         TokenKind::DirectiveElse => {
    //             self.parse_if_else_condition()?;
    //             IfCondition::Else
    //         }
    //         _ => IfCondition::Exp(self.parse_if_exp_condition(if_token)?),
    //     })
    // }

    // fn parse_if_else_condition(&mut self) -> PResult<()> {
    //     self.skip_whitespace();

    //     if let Some(&next) = self.consume_next() {
    //         if next.kind != TokenKind::Newline {
    //             self.session
    //                 .struct_span_error(next.as_span(), "Unexpected token".to_string())
    //                 .emit();

    //             return Err(());
    //         }
    //     }

    //     Ok(())
    // }

    // fn parse_if_def_condition(&mut self) -> PResult<IfDefCondition> {
    //     let mut span = Span::new(0, 0, 0);

    //     self.skip_whitespace();

    //     // We need an identifier, that is the entire idea of an .ifdef
    //     let identifier = self.parse_ident()?;

    //     // A number of arguments is optional though
    //     let args = self.parse_ml_macro_args()?;

    //     span.start = identifier.span.start;
    //     span.file = identifier.span.file;

    //     if let Some(args) = &args {
    //         span.end = args.span.end;
    //     } else {
    //         span.end = identifier.span.end;
    //     }

    //     Ok(IfDefCondition::new(span, identifier, args))
    // }

    // fn parse_if_exp_condition(&mut self, if_token: Token) -> PResult<IfExpCondition> {
    //     let mut span = Span::new(0, 0, 0);

    //     let mut expression = Vec::new();

    //     let mut benign_tokens = Vec::new();

    //     let mut ended = false;

    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     while let Some(&token) = self.consume_next() {
    //         match token.kind {
    //             TokenKind::Newline => {
    //                 ended = true;
    //                 break;
    //             }
    //             TokenKind::DirectiveDefine
    //             | TokenKind::DirectiveUndef
    //             | TokenKind::DirectiveMacro
    //             | TokenKind::DirectiveEndmacro
    //             | TokenKind::DirectiveUnmacro
    //             | TokenKind::DirectiveRepeat
    //             | TokenKind::DirectiveEndRepeat
    //             | TokenKind::DirectiveInclude
    //             | TokenKind::DirectiveIf
    //             | TokenKind::DirectiveIfDef
    //             | TokenKind::DirectiveIfNot
    //             | TokenKind::DirectiveIfNotDef
    //             | TokenKind::DirectiveElse
    //             | TokenKind::DirectiveElseIf
    //             | TokenKind::DirectiveElseIfDef
    //             | TokenKind::DirectiveElseIfNot
    //             | TokenKind::DirectiveElseIfNotDef
    //             | TokenKind::DirectiveEndIf => {
    //                 self.session
    //                     .struct_span_error(token.as_span(), "Expected condition".to_string())
    //                     .emit();

    //                 return Err(());
    //             }
    //             TokenKind::Identifier => {
    //                 let snippet = self.session.span_to_snippet(&token.as_span());
    //                 let ident_str = snippet.as_slice();

    //                 // Tests if this is an instruction or not
    //                 if Opcode::from(ident_str) != Opcode::Bogus {
    //                     // If it is
    //                     // Just push it
    //                     benign_tokens.push(token);

    //                     span.end = token.as_span().end;
    //                 } else {
    //                     // If it isn't, it is going to be parsed as a macro invocation
    //                     let macro_invocation = self.parse_macro_invocation(token.as_span(), ident_str)?;

    //                     // If we have captured any tokens before this
    //                     if !benign_tokens.is_empty() {
    //                         let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
    //                         expression.push(PASTNode::BenignTokens(benign_tokens_node));

    //                         benign_tokens = Vec::new();
    //                     }

    //                     // Update this just in case it is the last part of the contents
    //                     span.end = macro_invocation.span.end;

    //                     expression.push(PASTNode::MacroInvocation(macro_invocation));
    //                 }
    //             }
    //             _ => {
    //                 // Just push this, it is allowed and not special
    //                 benign_tokens.push(token);

    //                 // Just in case this is the last one
    //                 span.end = token.as_span().end;
    //             }
    //         }
    //     }

    //     // Check if benign_tokens didn't end empty
    //     if !benign_tokens.is_empty() {
    //         expression.push(PASTNode::BenignTokens(BenignTokens::from_vec(
    //             benign_tokens,
    //         )));
    //     }

    //     // If our expression is completely empty
    //     if expression.is_empty() {
    //         // Emit an error
    //         self.session
    //             .struct_span_error(if_token.as_span(), "expected expression".to_string())
    //             .emit();

    //         return Err(());
    //     }

    //     // We didn't end with a newline, we ended with an EOF
    //     if !ended {
    //         // At this point due to the upper check, we are guaranteed to have a valid span
    //         self.session
    //             .struct_span_error(span, "expected newline after expression".to_string())
    //             .emit();

    //         return Err(());
    //     }

    //     Ok(IfExpCondition::new(span, expression))
    // }

    // // Parses a macro directive
    // //
    // // See the MLMacroDef grammar
    // //
    // fn parse_ml_macro_def(&mut self) -> PResult<PASTNode> {
    //     let mut span = Span::new(0, 0, 0);

    //     // Consume the .macro
    //     let macro_span = self.assert_next(TokenKind::DirectiveMacro)?;

    //     // Copy the span values
    //     span.start = macro_span.start;
    //     span.file = macro_span.file;

    //     // Skip whitespace
    //     self.skip_whitespace();

    //     let identifier = self.parse_ident()?;

    //     // Capture the macro arguments
    //     let args = self.parse_ml_macro_args()?;

    //     let defaults = if let Some(args) = &args {
    //         // Update this to be passed in
    //         span.end = args.span.end;

    //         if let Some(maximum) = args.maximum {
    //             let num_required_defaults = maximum.get() - args.required;

    //             let defaults = self.parse_ml_macro_defaults(span, num_required_defaults)?;

    //             span.end = defaults.span.end;

    //             Some(defaults)
    //         } else {
    //             None
    //         }
    //     } else {
    //         None
    //     };

    //     // Now we parse the actual contents
    //     let contents = self.parse_ml_macro_contents(macro_span)?;

    //     Ok(PASTNode::MLMacroDef(MLMacroDef::new(
    //         span, identifier, args, defaults, contents,
    //     )))
    // }

    // // Parse a multi line macro's contents
    // fn parse_ml_macro_contents(&mut self, macro_span: Span) -> PResult<Vec<PASTNode>> {
    //     let mut contents = Vec::new();
    //     let mut benign_tokens = Vec::new();
    //     let mut span = Span::new(0, 0, 0);
    //     let mut found_end = false;

    //     // Parse the first token. We will allow this to immediately be an .endmacro
    //     if let Some(&token) = self.consume_next() {
    //         if token.kind == TokenKind::DirectiveEndmacro {
    //             found_end = true;
    //         } else {
    //             span.start = token.as_span().start;
    //             span.file = token.as_span().file;

    //             benign_tokens.push(token);
    //         }
    //     } else {
    //         self.session
    //             .struct_span_error(macro_span, "Missing accompanying `.endmacro`".to_string())
    //             .emit();

    //         return Err(());
    //     }

    //     if !found_end {
    //         while let Some(&token) = self.consume_next() {
    //             match token.kind {
    //                 TokenKind::DirectiveDefine
    //                 | TokenKind::DirectiveMacro
    //                 | TokenKind::DirectiveRepeat
    //                 | TokenKind::DirectiveEndRepeat
    //                 | TokenKind::DirectiveInclude
    //                 | TokenKind::DirectiveUndef
    //                 | TokenKind::DirectiveElseIf
    //                 | TokenKind::DirectiveElseIfNot
    //                 | TokenKind::DirectiveElseIfDef
    //                 | TokenKind::DirectiveElseIfNotDef
    //                 | TokenKind::DirectiveElse
    //                 | TokenKind::DirectiveEndIf
    //                 | TokenKind::DirectiveUnmacro => {
    //                     self.session
    //                         .struct_span_error(
    //                             token.as_span(),
    //                             "Not allowed within .macro block".to_string(),
    //                         )
    //                         .span_label(macro_span, "in macro".to_string())
    //                         .emit();

    //                     return Err(());
    //                 }
    //                 TokenKind::DirectiveIf
    //                 | TokenKind::DirectiveIfNot
    //                 | TokenKind::DirectiveIfDef
    //                 | TokenKind::DirectiveIfNotDef => {
    //                     let if_statement = match self.parse_if_statement(token, false, false)? {
    //                         PASTNode::IfStatement(statement) => statement,
    //                         _ => unreachable!(),
    //                     };

    //                     // If we have captured any tokens before this
    //                     if !benign_tokens.is_empty() {
    //                         let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
    //                         contents.push(PASTNode::BenignTokens(benign_tokens_node));

    //                         benign_tokens = Vec::new();
    //                     }

    //                     // Update this just in case it is the last part of the contents
    //                     span.end = if_statement.span.end;

    //                     contents.push(PASTNode::IfStatement(if_statement));
    //                 }
    //                 TokenKind::DirectiveEndmacro => {
    //                     found_end = true;
    //                     break;
    //                 }
    //                 TokenKind::Identifier => {
    //                     let snippet = self.session.span_to_snippet(&token.as_span());
    //                     let ident_str = snippet.as_slice();

    //                     // Tests if this is an instruction or not
    //                     if Opcode::from(ident_str) != Opcode::Bogus {
    //                         // If it is
    //                         // Just push it
    //                         benign_tokens.push(token);

    //                         span.end = token.as_span().end;
    //                     } else {
    //                         // If it isn't, it is going to be parsed as a macro invocation
    //                         let macro_invocation = self.parse_macro_invocation(token.as_span(), ident_str)?;

    //                         // If we have captured any tokens before this
    //                         if !benign_tokens.is_empty() {
    //                             let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
    //                             contents.push(PASTNode::BenignTokens(benign_tokens_node));

    //                             benign_tokens = Vec::new();
    //                         }

    //                         // Update this just in case it is the last part of the contents
    //                         span.end = macro_invocation.span.end;

    //                         contents.push(PASTNode::MacroInvocation(macro_invocation));
    //                     }
    //                 }
    //                 TokenKind::SymbolAnd => {
    //                     benign_tokens.push(token);

    //                     // We expect an integer literal after this
    //                     if let Some(&hopefully_num) = self.consume_next() {
    //                         // If there is anything at all
    //                         if hopefully_num.kind != TokenKind::LiteralInteger {
    //                             if hopefully_num.kind != TokenKind::Newline {
    //                                 self.session
    //                                     .struct_span_error(
    //                                         hopefully_num.as_span(),
    //                                         "Expected argument number".to_string(),
    //                                     )
    //                                     .emit();
    //                             } else {
    //                                 self.session
    //                                     .struct_span_error(
    //                                         token.as_span(),
    //                                         "Expected argument number after `&`".to_string(),
    //                                     )
    //                                     .emit();
    //                             }

    //                             return Err(());
    //                         } else {
    //                             benign_tokens.push(hopefully_num);
    //                         }
    //                     }
    //                 }
    //                 _ => {
    //                     // Just push this, it is allowed and not special
    //                     benign_tokens.push(token);

    //                     // Just in case this is the last one
    //                     span.end = token.as_span().end;
    //                 }
    //             }
    //         }
    //     }

    //     // If we ended because we ran out of tokens that is bad, so check the flag
    //     if !found_end {
    //         self.session
    //             .struct_span_error(macro_span, "Missing accompanying `.endmacro`".to_string())
    //             .span_label(self.last_token.unwrap().as_span(), "Got end of file while parsing macro definition".to_string())
    //             .emit();

    //         return Err(());
    //     }

    //     // Check if benign_tokens didn't end empty
    //     if !benign_tokens.is_empty() {
    //         contents.push(PASTNode::BenignTokens(BenignTokens::from_vec(
    //             benign_tokens,
    //         )));
    //     }

    //     Ok(contents)
    // }

    // // Parse a multi line macro argument reference (ex. &2)
    // fn parse_ml_macro_arg_ref(&self, token: &Token, maximum: usize) -> PResult<usize> {
    //     if maximum == 0 {
    //         self.session
    //             .struct_span_error(
    //                 token.as_span(),
    //                 "Macro takes no arguments, yet found argument index"
    //                     .to_string(),
    //             )
    //             .emit();
    //         return Err(());
    //     }

    //     let arg_ref_snippet = self.session.span_to_snippet(&token.as_span());
    //     let arg_ref_str = arg_ref_snippet.as_slice();
    //     let arg_ref = match parse_integer_literal(arg_ref_str) {
    //         Ok(num) => num as usize,
    //         Err(_) => {
    //             self.session
    //                 .struct_span_error(
    //                     token.as_span(),
    //                     "Macro argument value out of bounds for signed 32 bit"
    //                         .to_string(),
    //                 )
    //                 .emit();
    //             return Err(());
    //         }
    //     };

    //     if arg_ref == 0 {
    //         self.session
    //             .struct_span_error(
    //                 token.as_span(),
    //                 "Macro argument indexes start at 1".to_string(),
    //             )
    //             .emit();
    //         return Err(());
    //     }

    //     if arg_ref > maximum {
    //         self.session
    //             .struct_span_error(
    //                 token.as_span(),
    //                 "Macro argument index out of bounds".to_string(),
    //             )
    //             .emit();
    //         return Err(());
    //     }

    //     return Ok(arg_ref);
    // }

    // // Parse a multi line macro argument defaults
    // //
    // // See the MLMacroDefDefaults grammar
    // //
    // fn parse_ml_macro_defaults(
    //     &mut self,
    //     err_span: Span,
    //     number: u8,
    // ) -> PResult<MLMacroDefDefaults> {
    //     let mut defaults = Vec::new();

    //     // Collect as many as we can
    //     while let (Some(default), end) = self.parse_ml_macro_default()? {
    //         defaults.push(default);

    //         if end {
    //             break;
    //         }
    //     }

    //     let defaults = MLMacroDefDefaults::from_vec(defaults);

    //     // Check if it is how many we need
    //     if defaults.values.len() != number as usize {
    //         if !defaults.values.is_empty() {
    //             self.session
    //                 .struct_error(format!("expected {} arguments", number))
    //                 .span_label(defaults.span, format!("found {}", defaults.values.len()))
    //                 .emit();
    //         } else {
    //             self.session
    //                 .struct_error(format!("expected {} arguments", number))
    //                 .span_label(err_span, "found none".to_string())
    //                 .emit();
    //         }

    //         Err(())
    //     } else {
    //         Ok(defaults)
    //     }
    // }

    // // Parse a single multi line macro argument default
    // //
    // // Only "benign" tokens are allowed. No macro invocations or preprocessor directives
    // //
    // // Returns a tuple of an Option<BenignTokens> that represents if there was a default to parse
    // //
    // fn parse_ml_macro_default(&mut self) -> PResult<(Option<BenignTokens>, bool)> {
    //     // Skip whitespace
    //     self.skip_whitespace();

    //     let mut tokens = Vec::new();
    //     let mut comma_span = None;
    //     let mut end = false;

    //     while let Some(&token) = self.consume_next() {
    //         if token.kind == TokenKind::SymbolComma {
    //             comma_span = Some(token.as_span());
    //             break;
    //         } else if token.kind == TokenKind::Newline {
    //             end = true;
    //             break;
    //         } else if token.kind == TokenKind::Identifier {
    //             let ident_snippet = self.session.span_to_snippet(&token.as_span());
    //             let ident_str = ident_snippet.as_slice();

    //             // If this isn't an instruction
    //             if Opcode::from(ident_str) == Opcode::Bogus {
    //                 self.session
    //                     .struct_span_error(
    //                         token.as_span(),
    //                         "Macro invocations are not allowed as defaults".to_string(),
    //                     )
    //                     .help("maybe you misspelled an instruction mnemonic?".to_string())
    //                     .emit();

    //                 return Err(());
    //             }
    //         }

    //         tokens.push(token);
    //     }

    //     if tokens.is_empty() {
    //         if let Some(comma_span) = comma_span {
    //             self.session
    //                 .struct_span_error(
    //                     comma_span,
    //                     "expected argument default before `,`".to_string(),
    //                 )
    //                 .emit();

    //             Err(())
    //         } else {
    //             Ok((None, end))
    //         }
    //     } else {
    //         Ok((Some(BenignTokens::from_vec(tokens)), end))
    //     }
    // }

    // // Parse a repeat directive
    // //
    // // See the Repeat grammar
    // //
    // fn parse_repeat(&mut self) -> PResult<PASTNode> {
    //     let mut span = Span::new(0, 0, 0);

    //     // Consume the .rep
    //     let rep_span = self.assert_next(TokenKind::DirectiveRepeat)?;

    //     // Copy the span values
    //     span.start = rep_span.start;
    //     span.file = rep_span.file;

    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // As per the grammar, the next tokens must not contain preprocessor directives
    //     let number = self.parse_repeat_number(rep_span)?;

    //     let contents = self.parse_repeat_contents(rep_span)?;

    //     Ok(PASTNode::Repeat(Repeat::new(span, number, contents)))
    // }

    // // Parses a repeat preprocessor directive's contents.
    // //
    // // See the Repeat grammar
    // //
    // fn parse_repeat_contents(&mut self, rep_span: Span) -> PResult<Vec<PASTNode>> {
    //     let mut contents = Vec::new();
    //     let mut inert_tokens = Vec::new();
    //     let mut span = Span::new(0, 0, 0);
    //     let mut found_end = false;

    //     // Parse the first token. We will allow this to immediately be an .endrep
    //     if let Some(&token) = self.consume_next() {
    //         if token.kind == TokenKind::DirectiveEndRepeat {
    //             found_end = true;
    //         } else {
    //             span.start = token.as_span().start;
    //             span.file = token.as_span().file;

    //             inert_tokens.push(token);
    //         }
    //     } else {
    //         self.session
    //             .struct_span_error(rep_span, "Missing accompanying `.endrep`".to_string())
    //             .emit();

    //         return Err(());
    //     }

    //     if !found_end {
    //         while let Some(&token) = self.consume_next() {
    //             match token.kind {
    //                 TokenKind::DirectiveDefine
    //                 | TokenKind::DirectiveMacro
    //                 | TokenKind::DirectiveEndmacro
    //                 | TokenKind::DirectiveRepeat
    //                 | TokenKind::DirectiveInclude
    //                 | TokenKind::DirectiveUndef
    //                 | TokenKind::DirectiveUnmacro
    //                 | TokenKind::DirectiveIf
    //                 | TokenKind::DirectiveIfNot
    //                 | TokenKind::DirectiveIfDef
    //                 | TokenKind::DirectiveIfNotDef
    //                 | TokenKind::DirectiveElseIf
    //                 | TokenKind::DirectiveElseIfNot
    //                 | TokenKind::DirectiveElseIfDef
    //                 | TokenKind::DirectiveElseIfNotDef
    //                 | TokenKind::DirectiveElse
    //                 | TokenKind::DirectiveEndIf => {
    //                     self.session
    //                         .struct_span_error(
    //                             token.as_span(),
    //                             "Not allowed within .rep block".to_string(),
    //                         )
    //                         .emit();

    //                     return Err(());
    //                 }
    //                 TokenKind::DirectiveEndRepeat => {
    //                     found_end = true;
    //                     break;
    //                 }
    //                 TokenKind::Identifier => {
    //                     let snippet = self.session.span_to_snippet(&token.as_span());
    //                     let ident_str = snippet.as_slice();

    //                     // Tests if this is an instruction or not
    //                     if Opcode::from(ident_str) != Opcode::Bogus {
    //                         // If it is
    //                         // Just push it
    //                         inert_tokens.push(token);

    //                         span.end = token.as_span().end;
    //                     } else {
    //                         // If it isn't, it is going to be parsed as a macro invocation
    //                         let macro_invocation = self.parse_macro_invocation(token.as_span(), ident_str)?;

    //                         // If we have captured any tokens before this
    //                         if !inert_tokens.is_empty() {
    //                             let inert_tokens_node = InertTokens::from_vec(inert_tokens);
    //                             contents.push(PASTNode::InertTokens(inert_tokens_node));

    //                             inert_tokens = Vec::new();
    //                         }

    //                         // Update this just in case it is the last part of the contents
    //                         span.end = macro_invocation.span.end;

    //                         contents.push(PASTNode::MacroInvocation(macro_invocation));
    //                     }
    //                 }
    //                 _ => {
    //                     // Just push this, it is allowed and not special
    //                     inert_tokens.push(token);

    //                     // Just in case this is the last one
    //                     span.end = token.as_span().end;
    //                 }
    //             }
    //         }
    //     }

    //     // If we ended because we ran out of tokens that is bad, so check the flag
    //     if !found_end {
    //         self.struct_err_expected_eof(self.last_token.unwrap().as_span(), ".endrep")
    //             .emit();
    //     }

    //     // Check if inert_tokens didn't end empty
    //     if !inert_tokens.is_empty() {
    //         contents.push(PASTNode::InertTokens(InertTokens::from_vec(
    //             inert_tokens,
    //         )));
    //     }

    //     Ok(contents)
    // }

    // // Parses a repeat directive number of repetitions, which can be an expression
    // fn parse_repeat_number(&mut self, directive_span: Span) -> PResult<RepeatNumber> {
    //     let expression = self.parse_non_preprocessor(&[])?;

    //     if let Some((span, expression)) = expression {
    //         Ok(RepeatNumber::new(span, expression))
    //     } else {
    //         self.session
    //             .struct_span_error(
    //                 directive_span,
    //                 ".rep requires a number of repetitions".to_string(),
    //             )
    //             .emit();

    //         Err(())
    //     }
    // }

    // // Parse a multi line macro undefinition
    // //
    // // See the MLMacroUndef grammar
    // //
    // fn parse_ml_macro_undef(&mut self) -> PResult<PASTNode> {
    //     let mut span = Span::new(0, 0, 0);

    //     // Consume the .unmacro
    //     let unmacro_span = self.assert_next(TokenKind::DirectiveUnmacro)?;

    //     // Copy the span values
    //     span.start = unmacro_span.start;
    //     span.file = unmacro_span.file;

    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // As per the grammar, the next token MUST be an identifier
    //     let identifier = self.parse_ident()?;

    //     // Now parse the optional arguments/range
    //     let args = match self.parse_ml_macro_args()? {
    //         Some(args) => args,
    //         None => MLMacroArgs::new(identifier.span, 0, None),
    //     };

    //     // Adjust the span
    //     span.end = args.span.end;

    //     Ok(PASTNode::MLMacroUndef(MLMacroUndef::new(
    //         span, identifier, args,
    //     )))
    // }

    // // Parse a multi line macro's number of arguments
    // fn parse_ml_macro_args(&mut self) -> PResult<Option<MLMacroArgs>> {
    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // If we have any tokens at all
    //     let required = if let Some(&next) = self.peek_next() {
    //         // If it is not a newline, then it _should_ be the number of arguments
    //         if next.kind == TokenKind::Newline {
    //             self.assert_next(TokenKind::Newline)?;

    //             // No number of arguments
    //             None
    //         } else {
    //             // Parse the number
    //             Some(self.parse_num_arguments()?)
    //         }
    //     } else {
    //         None
    //     };

    //     if let Some((required_span, required_num)) = required {
    //         // If we had the first number, there might be `-` and then another.
    //         let maximum = if let Some(&next) = self.peek_next() {
    //             // First test for a newline
    //             if next.kind == TokenKind::Newline {
    //                 self.assert_next(TokenKind::Newline)?;

    //                 None
    //             }
    //             // If it isn't a `-`
    //             else if next.kind != TokenKind::OperatorMinus {
    //                 // This is an error. With only the number of required arguments specified, we don't
    //                 // have any default arguments to take in, so there should be nothing there
    //                 self.struct_err_expected_found(next.as_span(), "`-` or a newline")
    //                     .emit();

    //                 return Err(());
    //             } else {
    //                 // We have a `-`
    //                 self.assert_next(TokenKind::OperatorMinus)?;

    //                 // Now parse the next number then
    //                 let (span, num) = self.parse_num_arguments()?;

    //                 // This has the additional bound that it must be > # of required arguments
    //                 if num <= required_num {
    //                     self.session
    //                         .struct_span_error(
    //                             span,
    //                             format!(
    //                                 "Maximum must be greater than number of required ({})",
    //                                 required_num
    //                             ),
    //                         )
    //                         .emit();

    //                     return Err(());
    //                 }

    //                 // SAFETY: We just checked if this was greater the number of required
    //                 // arguments, which has to be at least 0, so this has to be >= 1
    //                 Some((span, unsafe { NonZeroU8::new_unchecked(num) }))
    //             }
    //         } else {
    //             None
    //         };

    //         let mut span = required_span;

    //         if let Some((max_span, max_num)) = maximum {
    //             span.end = max_span.end;

    //             Ok(Some(MLMacroArgs::new(span, required_num, Some(max_num))))
    //         } else {
    //             Ok(Some(MLMacroArgs::new(span, required_num, None)))
    //         }
    //     }
    //     // If we didn't get a number of arguments at all, give the default of 0
    //     else {
    //         Ok(Some(MLMacroArgs::new(
    //             self.last_token.unwrap().as_span(),
    //             0,
    //             None,
    //         )))
    //     }
    // }

    // // Parse a single line macro undefinition
    // //
    // // See the SLMacroUndef grammar
    // //
    // fn parse_sl_macro_undef(&mut self) -> PResult<PASTNode> {
    //     let mut span = Span::new(0, 0, 0);

    //     // Consume the .undef
    //     let undef_span = self.assert_next(TokenKind::DirectiveUndef)?;

    //     // Copy the span values
    //     span.start = undef_span.start;
    //     span.file = undef_span.file;

    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // As per the grammar, the next token MUST be an identifier
    //     let identifier = self.parse_ident()?;

    //     // Now parse the optional number of arguments
    //     let args = match self.parse_sl_macro_undef_args()? {
    //         Some(args) => args,
    //         None => SLMacroUndefArgs::new(identifier.span, 0),
    //     };

    //     // Adjust the span
    //     span.end = args.span.end;

    //     Ok(PASTNode::SLMacroUndef(SLMacroUndef::new(
    //         span, identifier, args,
    //     )))
    // }

    // // Parses the number of arguments in a single line macro undefinition
    // fn parse_sl_macro_undef_args(&mut self) -> PResult<Option<SLMacroUndefArgs>> {
    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // If we have any tokens at all
    //     if let Some(&next) = self.peek_next() {
    //         // If it is not a newline, then it _should_ be the number of arguments
    //         if next.kind == TokenKind::Newline {
    //             self.assert_next(TokenKind::Newline)?;

    //             // No number of arguments
    //             Ok(None)
    //         } else {
    //             // Parse the number
    //             let (span, num) = self.parse_num_arguments()?;

    //             Ok(Some(SLMacroUndefArgs::new(span, num)))
    //         }
    //     } else {
    //         Ok(None)
    //     }
    // }

    // // Parses a number of arguments, which is basically just a number that has the additional
    // // requirement of being less than 255. This will also emit a special diagnostic that says that
    // // macro expansions are not allowed in this place.
    // fn parse_num_arguments(&mut self) -> PResult<(Span, u8)> {
    //     let (span, num) = match self.parse_number() {
    //         Ok(data) => Ok(data),
    //         Err((mut db, data)) => {
    //             if let Some((string, token)) = data {
    //                 // If we actually got an identifier
    //                 if token.kind == TokenKind::Identifier {
    //                     // If it isn't an instruction
    //                     if Opcode::from(string.as_str()) == Opcode::Bogus {
    //                         db.help("macro expansions are not allowed here".to_string());
    //                     }
    //                 }
    //             }

    //             db.emit();

    //             Err(())
    //         }
    //     }?;

    //     if num > u8::MAX as i32 {
    //         self.session
    //             .struct_span_error(
    //                 span,
    //                 "number greater than the maximum number of arguments (255)".to_string(),
    //             )
    //             .emit();

    //         Err(())
    //     } else {
    //         Ok((span, num as u8))
    //     }
    // }

    // // Parses a number from the source. This could be a hexadecimal, binary, or decimal number
    // //
    // // i32 is the return type because that is the maximum value that any kOS value can have, and it
    // // works for our purposes as well
    // //
    // fn parse_number(&mut self) -> NumPResult {
    //     if let Some(&token) = self.consume_next() {
    //         let span = token.as_span();
    //         let snippet = self.session.span_to_snippet(&span);
    //         let string = snippet.as_slice().to_string();

    //         match token.kind {
    //             TokenKind::LiteralInteger => {
    //                 if let Ok(num) = parse_integer_literal(&string) {
    //                     Ok((span, num))
    //                 } else {
    //                     Err((
    //                         self.session.struct_span_error(
    //                             span,
    //                             format!("number too large to be stored {}", string),
    //                         ),
    //                         Some((string, token)),
    //                     ))
    //                 }
    //             }
    //             TokenKind::LiteralHex => {
    //                 if let Ok(num) = parse_hexadecimal_literal(&string) {
    //                     Ok((span, num))
    //                 } else {
    //                     Err((
    //                         self.session.struct_span_error(
    //                             span,
    //                             format!("Number too large to be stored {}", string),
    //                         ),
    //                         Some((string, token)),
    //                     ))
    //                 }
    //             }
    //             TokenKind::LiteralBinary => {
    //                 if let Ok(num) = parse_binary_literal(&string) {
    //                     Ok((span, num))
    //                 } else {
    //                     Err((
    //                         self.session.struct_span_error(
    //                             span,
    //                             format!("Number too large to be stored {}", string),
    //                         ),
    //                         Some((string, token)),
    //                     ))
    //                 }
    //             }
    //             _ => Err((
    //                 self.struct_err_expected_found(token.as_span(), "number"),
    //                 Some((string, token)),
    //             )),
    //         }
    //     } else {
    //         Err((
    //             self.struct_err_expected_eof(self.last_token.unwrap().as_span(), "number"),
    //             None,
    //         ))
    //     }
    // }

    // // Parse a single line macro definition
    // //
    // // See the SLMacroDef grammar
    // //
    // fn parse_sl_macro_def(&mut self) -> PResult<PASTNode> {
    //     let mut span = Span::new(0, 0, 0);
    //     // Consume the .define
    //     let define_span = self.assert_next(TokenKind::DirectiveDefine)?;

    //     // Copy the span values
    //     span.start = define_span.start;
    //     span.file = define_span.file;

    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // As per the grammar, the next token MUST be an identifier
    //     let identifier = self.parse_ident()?;

    //     // Now we parse the optional arguments
    //     let args = self.parse_sl_macro_def_args()?;

    //     let not_macros: &[Ident] = match &args {
    //         Some(def_args) => &def_args.args,
    //         None => &[],
    //     };

    //     // Then the optional contents
    //     let contents = self.parse_sl_macro_def_contents(not_macros)?;

    //     // Adjust this SLMacroDef's span
    //     if let Some(contents) = &contents {
    //         span.end = contents.span.end;
    //     } else if let Some(args) = &args {
    //         // This means we have arguments, but no tokens to expand
    //         // This is valid, but we should emit a warning

    //         self.session
    //             .struct_span_warn(args.span, "Macro arguments but no expansion".to_string())
    //             .emit();

    //         span.end = args.span.end;
    //     } else {
    //         span.end = identifier.span.end;
    //     }

    //     Ok(PASTNode::SLMacroDef(SLMacroDef::new(
    //         span, identifier, args, contents,
    //     )))
    // }

    // // Parse a single line macro definition arguments
    // fn parse_sl_macro_def_args(&mut self) -> PResult<Option<SLMacroDefArgs>> {
    //     if let Some(&token) = self.peek_next() {
    //         // ( is required
    //         if token.kind != TokenKind::SymbolLeftParen {
    //             // No (, no args
    //             Ok(None)
    //         } else {
    //             let mut arguments = Vec::new();
    //             let mut span = Span::new(0, 0, 0);

    //             // Consume the (
    //             let paren_span = self.assert_next(TokenKind::SymbolLeftParen)?;

    //             span.start = paren_span.start;
    //             span.file = paren_span.file;

    //             // We have (
    //             // Now we parse until we reach a )
    //             while let Some(&token) = self.peek_next() {
    //                 // Check if it is a )
    //                 if token.kind == TokenKind::SymbolRightParen {
    //                     break;
    //                 }

    //                 // We could have whitespace before this which shouldn't matter
    //                 self.skip_whitespace();

    //                 // It should now be an identifier
    //                 let ident = self.parse_ident()?;
    //                 arguments.push(ident);

    //                 // We could also have whitespace after it
    //                 self.skip_whitespace();

    //                 // Now we should check if it is a comma, or a ). Anything else is not allowed
    //                 if let Some(&next) = self.peek_next() {
    //                     if next.kind == TokenKind::SymbolComma {
    //                         self.assert_next(TokenKind::SymbolComma)?;
    //                     } else if next.kind == TokenKind::SymbolRightParen {
    //                         continue;
    //                     } else {
    //                         // Emit an error, it wasn't either of them
    //                         self.session
    //                             .struct_span_error(next.as_span(), "`,` or `)`".to_string())
    //                             .emit();

    //                         return Err(());
    //                     }
    //                 }
    //             }

    //             // We need to check if this ended because we ran out of tokens, which isn't okay
    //             if self.peek_next().is_none() {
    //                 // Emit an error
    //                 self.struct_err_expected_eof(self.last_token.unwrap().as_span(), ")")
    //                     .emit();

    //                 Err(())
    //             } else {
    //                 // Consume the ) that caused us to stop
    //                 let right_span = self.assert_next(TokenKind::SymbolRightParen)?;

    //                 span.end = right_span.end;

    //                 Ok(Some(SLMacroDefArgs::new(span, arguments)))
    //             }
    //         }
    //     } else {
    //         Ok(None)
    //     }
    // }
    // // Parse a single line macro definition contents
    // fn parse_sl_macro_def_contents(
    //     &mut self,
    //     not_macros: &[Ident],
    // ) -> PResult<Option<SLMacroDefContents>> {
    //     if let Some((span, contents)) = self.parse_non_preprocessor(not_macros)? {
    //         Ok(Some(SLMacroDefContents::new(span, contents)))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // // Parse a sequence of tokens ended by a newline or EOF that are "benign tokens" or macro
    // // expansions. This just means that preprocessor directives are not allowed. Macro invocations, expressions, etc, are
    // // all allowed.
    // fn parse_non_preprocessor(
    //     &mut self,
    //     not_macros: &[Ident],
    // ) -> PResult<Option<(Span, Vec<PASTNode>)>> {
    //     // Skip any whitespace
    //     self.skip_whitespace();

    //     // We basically let anything in except for preprocessor directives
    //     // We do let in:
    //     //  Macro invocations
    //     //  Other directives

    //     // We will tell if an identifier is an instruction or a macro invocation by checking it
    //     // against the names of instructions

    //     // If we do have any token
    //     if let Some(&next) = self.peek_next() {
    //         // If it is a newline, this doesn't count as contents
    //         if next.kind == TokenKind::Newline {
    //             self.assert_next(TokenKind::Newline)?;
    //             return Ok(None);
    //         }

    //         let mut span = Span::new(0, 0, 0);
    //         let first_span = next.as_span();
    //         span.file = first_span.file;
    //         span.start = first_span.start;

    //         let mut nodes = Vec::new();
    //         let mut benign_tokens = Vec::new();

    //         // Now that we know there is other stuff, loop until a newline/EOF
    //         while let Some(&next) = self.consume_next() {
    //             match next.kind {
    //                 TokenKind::DirectiveIf
    //                 | TokenKind::DirectiveIfNot
    //                 | TokenKind::DirectiveIfDef
    //                 | TokenKind::DirectiveIfNotDef
    //                 | TokenKind::DirectiveEndIf
    //                 | TokenKind::DirectiveElse
    //                 | TokenKind::DirectiveElseIf
    //                 | TokenKind::DirectiveElseIfNot
    //                 | TokenKind::DirectiveElseIfDef
    //                 | TokenKind::DirectiveElseIfNotDef
    //                 | TokenKind::DirectiveMacro
    //                 | TokenKind::DirectiveEndmacro
    //                 | TokenKind::DirectiveRepeat
    //                 | TokenKind::DirectiveEndRepeat
    //                 | TokenKind::DirectiveDefine
    //                 | TokenKind::DirectiveUndef
    //                 | TokenKind::DirectiveUnmacro
    //                 | TokenKind::DirectiveInclude => {
    //                     self.session
    //                         .struct_span_error(
    //                             next.as_span(),
    //                             "Preprocessor directives not allowed here".to_string(),
    //                         )
    //                         .emit();

    //                     return Err(());
    //                 }
    //                 TokenKind::Newline => break,
    //                 TokenKind::Identifier => {
    //                     let snippet = self.session.span_to_snippet(&next.as_span());
    //                     let ident_str = snippet.as_slice();

    //                     // Tests if this is an instruction or not
    //                     if Opcode::from(ident_str) != Opcode::Bogus {
    //                         // If it is
    //                         // Just push it
    //                         benign_tokens.push(next);

    //                         span.end = next.as_span().end;
    //                     } else {
    //                         let mut hasher = DefaultHasher::new();
    //                         hasher.write(ident_str.as_bytes());
    //                         let ident_hash = hasher.finish();

    //                         // Now check if it is actually the identifier representing an argument
    //                         // of this macro
    //                         if not_macros.iter().any(|ident| ident.hash == ident_hash) {
    //                             // Just add it to the benign tokens
    //                             benign_tokens.push(next);

    //                             // Just in case this is the last one
    //                             span.end = next.as_span().end;
    //                         } else {
    //                             // If it isn't, it is going to be parsed as a macro invocation
    //                             let macro_invocation =
    //                                 self.parse_macro_invocation(next.as_span(), ident_str)?;

    //                             // If we have captured any tokens before this
    //                             if !benign_tokens.is_empty() {
    //                                 let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
    //                                 nodes.push(PASTNode::BenignTokens(benign_tokens_node));

    //                                 benign_tokens = Vec::new();
    //                             }

    //                             // Update this just in case it is the last part of the contents
    //                             span.end = macro_invocation.span.end;

    //                             nodes.push(PASTNode::MacroInvocation(macro_invocation));
    //                         }
    //                     }
    //                 }
    //                 _ => {
    //                     // Just push this, it is allowed and not special
    //                     benign_tokens.push(next);

    //                     // Just in case this is the last one
    //                     span.end = next.as_span().end;
    //                 }
    //             }
    //         }

    //         // Check if benign_tokens didn't end empty
    //         if !benign_tokens.is_empty() {
    //             nodes.push(PASTNode::BenignTokens(BenignTokens::from_vec(
    //                 benign_tokens,
    //             )));
    //         }
    //         Ok(Some((span, nodes)))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // // Parses a macro invocation
    // fn parse_macro_invocation(&mut self, ident_span: Span, ident_str: &str) -> PResult<MacroInvocation> {
    //     let mut hasher = DefaultHasher::new();
    //     hasher.write(ident_str.as_bytes());
    //     let hash = hasher.finish();
    //     let mut span = Span::new(ident_span.start, 0, ident_span.file);

    //     let identifier = Ident::new(ident_span, hash);

    //     // After the identifier, there could be arguments, or not
    //     if let Some(&token) = self.peek_next() {
    //         if token.kind == TokenKind::SymbolLeftParen {
    //             self.assert_next(TokenKind::SymbolLeftParen)?;

    //             let args = self.parse_macro_invocation_args(token.as_span())?;

    //             if let Some(macro_args) = &args {
    //                 span.end = macro_args.span.end;
    //             } else {
    //                 span.end = identifier.span.end;
    //             }

    //             Ok(MacroInvocation::new(span, identifier, args))
    //         } else {
    //             span.end = identifier.span.end;

    //             Ok(MacroInvocation::new(span, identifier, None))
    //         }
    //     } else {
    //         span.end = identifier.span.end;

    //         Ok(MacroInvocation::new(span, identifier, None))
    //     }
    // }

    // // Parses a macro invocation's arguments
    // fn parse_macro_invocation_args(&mut self, paren_span: Span) -> PResult<Option<MacroInvocationArgs>> {
    //     if self.peek_next().is_none() {
    //         self.session
    //             .struct_bug(
    //                 "Found non-whitespace after macro invocation, but found no tokens".to_string(),
    //             )
    //             .emit();

    //         return Err(());
    //     }

    //     if let Some(token) = self.peek_next() {
    //         if token.kind == TokenKind::SymbolRightParen {
    //             self.assert_next(TokenKind::SymbolRightParen)?;
    //             return Ok(None);
    //         }
    //     }

    //     let mut args = Vec::new();

    //     loop {
    //         let (arg, is_last) = self.parse_macro_invocation_arg(paren_span)?;

    //         args.push(arg);

    //         if is_last {
    //             break;
    //         }
    //     }

    //     Ok(Some(MacroInvocationArgs::from_vec(args)))
    // }

    // // Parses a single macro invocation argument, ended by a comma, or a `)`
    // //
    // // No preprocessor directives are allowed as argument parts, but other macro invocations are
    // // allowed.
    // //
    // fn parse_macro_invocation_arg(&mut self, paren_span: Span) -> PResult<(MacroInvocationArg, bool)> {
    //     let mut span = Span::new(0, 0, 0);

    //     let mut contents = Vec::new();
    //     let mut benign_tokens = Vec::new();
    //     let mut comma_span = None;
    //     let mut close_paren_span = None;
    //     let mut is_last = false;

    //     if let Some(&token) = self.consume_next() {
    //         let token_span = token.as_span();

    //         match token.kind {
    //             TokenKind::SymbolComma => {
    //                 comma_span = Some(token_span);

    //                 span.start = token_span.start;
    //                 span.file = token_span.file;
    //             }
    //             TokenKind::SymbolRightParen => {
    //                 close_paren_span = Some(token.as_span());
    //                 is_last = true;

    //                 span.start = token_span.start;
    //                 span.file = token_span.file;
    //             }
    //             TokenKind::Newline => {
    //                 self.session
    //                     .struct_span_error(
    //                         paren_span,
    //                         "Macro invocation requires closing `)`".to_string(),
    //                     )
    //                     .emit();

    //                 return Err(());
    //             }
    //             TokenKind::DirectiveDefine
    //             | TokenKind::DirectiveMacro
    //             | TokenKind::DirectiveEndmacro
    //             | TokenKind::DirectiveUndef
    //             | TokenKind::DirectiveUnmacro
    //             | TokenKind::DirectiveRepeat
    //             | TokenKind::DirectiveEndRepeat
    //             | TokenKind::DirectiveInclude
    //             | TokenKind::DirectiveIf
    //             | TokenKind::DirectiveIfNot
    //             | TokenKind::DirectiveIfDef
    //             | TokenKind::DirectiveIfNotDef
    //             | TokenKind::DirectiveElseIf
    //             | TokenKind::DirectiveElseIfNot
    //             | TokenKind::DirectiveElseIfDef
    //             | TokenKind::DirectiveElseIfNotDef
    //             | TokenKind::DirectiveElse
    //             | TokenKind::DirectiveEndIf => {
    //                 self.session
    //                     .struct_error("Not allowed in macro arguments".to_string())
    //                     .span_label(token.as_span(), "found preprocessor directive".to_string())
    //                     .emit();

    //                 return Err(());
    //             }
    //             TokenKind::Identifier => {
    //                 let snippet = self.session.span_to_snippet(&token_span);
    //                 let ident_str = snippet.as_slice();

    //                 // Tests if this is an instruction or not
    //                 if Opcode::from(ident_str) != Opcode::Bogus {
    //                     // If it is
    //                     // Just push it
    //                     benign_tokens.push(token);

    //                     span.end = token.as_span().end;
    //                 } else {
    //                     // If it isn't, it is going to be parsed as a macro invocation
    //                     let macro_invocation = self.parse_macro_invocation(token.as_span(), ident_str)?;

    //                     // If we have captured any tokens before this
    //                     if !benign_tokens.is_empty() {
    //                         let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
    //                         contents.push(PASTNode::BenignTokens(benign_tokens_node));

    //                         benign_tokens = Vec::new();
    //                     }

    //                     // Update this just in case it is the last part of the contents
    //                     span.end = macro_invocation.span.end;

    //                     contents.push(PASTNode::MacroInvocation(macro_invocation));
    //                 }

    //                 span.start = token_span.start;
    //                 span.file = token_span.file;
    //             }
    //             _ => {
    //                 benign_tokens.push(token);

    //                 span.start = token_span.start;
    //                 span.file = token_span.file;
    //             }
    //         }
    //     }

    //     while let Some(&token) = self.consume_next() {
    //         match token.kind {
    //             TokenKind::SymbolComma => {
    //                 comma_span = Some(token.as_span());
    //                 break;
    //             }
    //             TokenKind::SymbolRightParen => {
    //                 close_paren_span = Some(token.as_span());
    //                 is_last = true;
    //                 break;
    //             }
    //             TokenKind::Newline => {
    //                 self.session
    //                     .struct_span_error(
    //                         paren_span,
    //                         "Macro invocation requires closing `)`".to_string(),
    //                     )
    //                     .emit();

    //                 return Err(());
    //             }
    //             TokenKind::DirectiveDefine
    //             | TokenKind::DirectiveMacro
    //             | TokenKind::DirectiveEndmacro
    //             | TokenKind::DirectiveUndef
    //             | TokenKind::DirectiveUnmacro
    //             | TokenKind::DirectiveRepeat
    //             | TokenKind::DirectiveEndRepeat
    //             | TokenKind::DirectiveInclude
    //             | TokenKind::DirectiveIf
    //             | TokenKind::DirectiveIfNot
    //             | TokenKind::DirectiveIfDef
    //             | TokenKind::DirectiveIfNotDef
    //             | TokenKind::DirectiveElseIf
    //             | TokenKind::DirectiveElseIfNot
    //             | TokenKind::DirectiveElseIfDef
    //             | TokenKind::DirectiveElseIfNotDef
    //             | TokenKind::DirectiveElse
    //             | TokenKind::DirectiveEndIf => {
    //                 self.session
    //                     .struct_error("Not allowed in macro arguments".to_string())
    //                     .span_label(token.as_span(), "found preprocessor directive".to_string())
    //                     .emit();

    //                 return Err(());
    //             }
    //             TokenKind::Identifier => {
    //                 let snippet = self.session.span_to_snippet(&token.as_span());
    //                 let ident_str = snippet.as_slice();

    //                 // Tests if this is an instruction or not
    //                 if Opcode::from(ident_str) != Opcode::Bogus {
    //                     // If it is
    //                     // Just push it
    //                     benign_tokens.push(token);

    //                     span.end = token.as_span().end;
    //                 } else {
    //                     // If it isn't, it is going to be parsed as a macro invocation
    //                     let macro_invocation = self.parse_macro_invocation(token.as_span(), ident_str)?;

    //                     // If we have captured any tokens before this
    //                     if !benign_tokens.is_empty() {
    //                         let benign_tokens_node = BenignTokens::from_vec(benign_tokens);
    //                         contents.push(PASTNode::BenignTokens(benign_tokens_node));

    //                         benign_tokens = Vec::new();
    //                     }

    //                     // Update this just in case it is the last part of the contents
    //                     span.end = macro_invocation.span.end;

    //                     contents.push(PASTNode::MacroInvocation(macro_invocation));
    //                 }
    //             }
    //             _ => {
    //                 benign_tokens.push(token);
    //             }
    //         }
    //     }

    //     // Check if benign_tokens didn't end empty
    //     if !benign_tokens.is_empty() {
    //         contents.push(PASTNode::BenignTokens(BenignTokens::from_vec(
    //             benign_tokens,
    //         )));
    //     }

    //     // If we just have nothing
    //     if contents.is_empty() {
    //         // This is always an error
    //         if let Some(comma_span) = comma_span {
    //             self.session
    //                 .struct_span_error(comma_span, "No arguments after comma".to_string())
    //                 .emit();
    //         } else if let Some(close_paren_span) = close_paren_span {
    //             self.session
    //                 .struct_span_error(close_paren_span, "Expected argument before `)`".to_string())
    //                 .emit();
    //         }

    //         return Err(());
    //     } else {
    //         span.end = self.last_token.unwrap().as_span().end;
    //     }

    //     Ok((MacroInvocationArg::new(span, contents), is_last))
    // }

    fn expect_newline_or_eof(&mut self, after: &str) -> PResult<()> {
        if let Some(&token) = self.consume_next() {
            if token.kind == TokenKind::Newline {
                Ok(())
            } else {
                self.struct_err_expected_found(token.as_span(), &format!("newline or end of file after {after}")).emit();
                Err(())
            }
        } else {
            Ok(())
        }
    }

    // Peeks the next token from the Parser's tokens
    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.token_cursor)
    }

    // Consumes the next token from the Parser's tokens
    fn consume_next(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.token_cursor)?;

        self.token_cursor += 1;
        self.last_token = Some(*token);

        Some(token)
    }

    // Emits a bug diagnostic and Err(()) if the token was not what we assumed it to be
    fn assert_next(&mut self, token_kind: TokenKind) -> PResult<Span> {
        if let Some(&token) = self.consume_next() {
            if token.kind == token_kind {
                Ok(token.as_span())
            } else {
                self.session
                    .struct_bug(format!(
                        "Token assertion failed: {:?}, found {:?}",
                        token_kind, token.kind
                    ))
                    .emit();
                Err(())
            }
        } else {
            // If it doesn't exist, that is also a problem
            self.session
                .struct_bug(format!(
                    "{:?} token assumed to exist, found None",
                    token_kind
                ))
                .emit();
            Err(())
        }
    }

    // A helper method for asserting that the next token is a preprocessor directive
    fn assert_next_preprocessor(&mut self, directive: PreprocessorDirective) -> PResult<Span> {
        self.assert_next(TokenKind::Directive(Directive::Preprocessor(directive)))
    }

    // Skips whitespace if there is any, not including newlines
    //
    // Returns true if there was any, false if not
    fn skip_whitespace(&mut self) -> bool {
        let mut was_whitespace = false;

        while self.peek_next().is_some() && self.peek_next().unwrap().kind == TokenKind::Whitespace
        {
            was_whitespace = true;

            // Increment the token cursor
            self.token_cursor += 1;
        }

        was_whitespace
    }

    // // Parses an identifier, or returns an Err(())
    // //
    // // This will emit a diagnostic if the next token is not an identifier
    // //
    // fn parse_ident(&mut self) -> PResult<Ident> {
    //     if let Some(&token) = self.consume_next() {
    //         if token.kind == TokenKind::Identifier {
    //             let span = token.as_span();
    //             let snippet = self.session.span_to_snippet(&span);

    //             let mut hasher = DefaultHasher::new();
    //             hasher.write(snippet.as_slice().as_bytes());
    //             let hash = hasher.finish();

    //             Ok(Ident { span, hash })
    //         } else {
    //             self.struct_err_expected_found(token.as_span(), "identifier")
    //                 .emit();

    //             Err(())
    //         }
    //     } else {
    //         self.struct_err_expected_eof(self.last_token.unwrap().as_span(), "identifier")
    //             .emit();

    //         Err(())
    //     }
    // }

    fn struct_err_expected_eof(&self, last: Span, expected: &str) -> DiagnosticBuilder<'_> {
        let message = format!("expected {}", expected);
        let mut db = self.session.struct_error(message);

        db.span_label(last, "found end of file".to_string());

        db
    }

    fn struct_err_expected_found(&self, found: Span, expected: &str) -> DiagnosticBuilder<'_> {
        let message = format!("expected {}", expected);
        let mut db = self.session.struct_error(message);

        db.span_label(
            found,
            format!("found `{}`", self.session.span_to_snippet(&found)),
        );

        db
    }

    fn string_literal_to_string(&self, token: Token) -> PResult<String> {
        if token.kind != TokenKind::Literal(Literal::String) {
            self.session.struct_bug("Called method expecting token to be string literal".to_string()).emit();
            return Err(());
        }

        Ok(self.session.span_to_snippet(&token.as_span()).as_slice().trim_matches('\"').to_string())
    }
}

/// Parses an integer literal from the given &str
///
/// This differs from the normal &str::parse() because it supports random `_` characters in the
/// integer. They allow for more easily readable constants
///
pub fn parse_integer_literal(string: &str) -> Result<i32, ()> {
    // This makes sure we only have to allocate once
    let mut no_separators = String::with_capacity(string.len());

    for c in string.chars() {
        if c.is_digit(10) {
            no_separators.push(c);
        } else if c != '_' {
            return Err(());
        }
    }

    Ok(no_separators.parse().unwrap())
}

/// Parses a hexadecimal literal from the given &str
pub fn parse_hexadecimal_literal(string: &str) -> Result<i32, ()> {
    let string = &string[2..];
    let mut no_separators = String::with_capacity(string.len());

    for c in string.chars() {
        if c.is_digit(16) {
            no_separators.push(c);
        } else if c != '_' {
            return Err(());
        }
    }

    Ok(i32::from_str_radix(&no_separators, 16).unwrap())
}

/// Parses a binary literal from the given &str
pub fn parse_binary_literal(string: &str) -> Result<i32, ()> {
    let string = &string[2..];
    let mut no_separators = String::with_capacity(string.len());

    for c in string.chars() {
        if c == '0' || c == '1' {
            no_separators.push(c);
        } else if c != '_' {
            return Err(());
        }
    }

    Ok(i32::from_str_radix(&no_separators, 2).unwrap())
}

/// Parses a float literal from the given &str
pub fn parse_float_literal(string: &str) -> Result<f64, ()> {
    Ok(string.parse().unwrap())
}