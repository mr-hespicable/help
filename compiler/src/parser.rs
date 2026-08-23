use crate::ast::{ASTNode, ASTNodeKind};
use crate::lexer::Token;
use crate::errors::ParserError;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<Token>,
}


// program(function_decl(function(string, statement(return exp(constant(int)))))

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        // pass in program tokens
        Parser {
            tokens: VecDeque::from(tokens),
        }
    }

    fn expect_token(&mut self, expected_token: Token) -> Result<(), ParserError> {
        if let Some(token) = self.tokens.pop_front() && token == expected_token {
            Ok(())
        } else {
            Err(self.fail("failed while expecting token"))
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::Program, None);

        let function = self.parse_function()?;
        ast_node.push_child(function);
        Ok(ast_node)
    }

    fn parse_function(&mut self) -> Result<ASTNode, ParserError> {
        let mut ast_node: ASTNode;
        
        dbg![&self.tokens];
        let Token::DataType(data_type) = self.tokens.pop_front().ok_or_else(|| self.fail("failed while parsing function"))? else {
            return Err(self.fail("failed while parsing function"));
        };

        let Token::Identifier(id) = self.tokens.pop_front().ok_or_else(|| self.fail("failed while parsing function"))? else {
            return Err(self.fail("failed while parsing function"));
        };
        ast_node = ASTNode::new(ASTNodeKind::FunctionDeclaration(data_type, id), None);

        self.expect_token(Token::LeftParen)?;
        self.expect_token(Token::RightParen)?;
        self.expect_token(Token::LeftBrace)?;

        let statement = self.parse_statement()?;
        ast_node.push_child(statement);

        self.expect_token(Token::RightBrace)?;

        Ok(ast_node)
    }

    fn parse_statement(&mut self) -> Result<ASTNode, ParserError> {
        let current_token = self.tokens.pop_front();
        let Token::Keyword(k) = current_token.ok_or_else(|| self.fail("failed while parsing statement"))? else {
            return Err(self.fail("failed while parsing statement"));
        };

        if k != "return" {
            return Err(self.fail("failed while parsing statement"));
        }

        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::Statement(k), None);

        let expression = self.parse_expression()?;
        ast_node.push_child(expression); 

        self.expect_token(Token::Semicolon)?;

        Ok(ast_node)
    }

    fn parse_expression(&mut self) -> Result<ASTNode, ParserError> {
        let current_token = self.tokens.pop_front();
        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::Expression, None);

        match current_token {
            Some(Token::DecimalIntegarLiteral(n)) => ast_node.push_child(ASTNode::new(ASTNodeKind::Constant(n.to_string()), None)), // "2" or "5"
            Some(other_token) => ast_node.push_child(self.parse_unary(other_token)?), // "-2" or "-5"
            _ => return Err(self.fail("failed while parsing expression"))
        }

        Ok(ast_node)
    }

    fn parse_unary(&mut self, current_token: Token) -> Result<ASTNode, ParserError> {
        let ast_node: ASTNode;

        dbg![&current_token];

        match current_token {
            Token::Negation => ast_node = ASTNode::new(ASTNodeKind::UnOp('-'), Some(vec![self.parse_expression()?])),
            Token::BitwiseComplement => ast_node = ASTNode::new(ASTNodeKind::UnOp('~'), Some(vec![self.parse_expression()?])),
            Token::LogicalNegation => ast_node = ASTNode::new(ASTNodeKind::UnOp('!'), Some(vec![self.parse_expression()?])),
            _ => return Err(self.fail("failed while parsing unary"))
        }

        Ok(ast_node)
    }

    #[track_caller]
    fn fail(&mut self, message: &str) -> ParserError {
        let caller_location = std::panic::Location::caller();
        let err_string = message.to_string();
        ParserError(format!(
            "{}; in {} @ {}:{}",
            err_string,
            caller_location.file(),
            caller_location.line(),
            caller_location.column()
        ))
        
    }
}
