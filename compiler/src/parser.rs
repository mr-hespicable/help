use crate::ast::{ASTNode, ASTNodeKind};
use crate::errors::ParserError;
use crate::lexer::Token;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        // pass in program tokens
        Parser {
            tokens: VecDeque::from(tokens),
        }
    }

    fn expect_token(&mut self, expected_token: Token) -> Result<(), ParserError> {
        if let Some(token) = self.tokens.pop_front()
            && token == expected_token
        {
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

        let Token::DataType(data_type) = self
            .tokens
            .pop_front()
            .ok_or_else(|| self.fail("failed while parsing function"))?
        else {
            return Err(self.fail("failed while parsing function"));
        };

        let Token::Identifier(id) = self
            .tokens
            .pop_front()
            .ok_or_else(|| self.fail("failed while parsing function"))?
        else {
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
        let Token::Keyword(k) =
            current_token.ok_or_else(|| self.fail("failed while parsing statement"))?
        else {
            return Err(self.fail("failed while parsing statement"));
        };

        if k != "return" {
            return Err(self.fail("failed while parsing statement"));
        }

        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::Statement(k), None);

        let expression = self.parse_additive()?;
        ast_node.push_child(expression);

        self.expect_token(Token::Semicolon)?;

        Ok(ast_node)
    }

    fn parse_additive(&mut self) -> Result<ASTNode, ParserError> {
        // additive ( + or - ) additive | multiplicative
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| *x == Token::Add || *x == Token::Minus;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_multiplicative()?;
        children[0] = Some(lhs);

        let mut next = self.tokens.pop_front_if(predicate);

        while next.is_some() {
            if ast_node.is_some() {
                // i.e. in > 2 iteration of loop
                for child in &children {
                    ast_node
                        .as_mut()
                        .unwrap()
                        .push_child(child.clone().unwrap()); // WHYYYY
                }
                children[0] = ast_node;
                children[1] = None;
            }
            let operator = next.clone(); // unavoidable since VecDeque 
            let rhs = self.parse_multiplicative()?;

            match operator {
                Some(Token::Add) => ast_node = Some(ASTNode::new(ASTNodeKind::AddSubOp('+'), None)),
                Some(Token::Minus) => ast_node = Some(ASTNode::new(ASTNodeKind::AddSubOp('-'), None)),
                _ => return Err(self.fail("SOMEHOW. Somehow the operator is none. this is literally impossible. if this triggers idk what to do.")),
            }

            next = self.tokens.pop_front_if(predicate);

            children[1] = Some(rhs);
        }

        for child in children {
            if ast_node.is_none() && child.is_some() {
                ast_node = Some(child.unwrap());
            } else if child.is_some() {
                ast_node.as_mut().unwrap().push_child(child.unwrap())
            }
        }

        match ast_node {
            Some(n) => Ok(n.clone()),
            None => Err(self.fail("failed because ast_node is None")),
        }
    }

    fn parse_multiplicative(&mut self) -> Result<ASTNode, ParserError> {
        // mult ( * or / ) mult | primary
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| *x == Token::Multiply || *x == Token::Divide;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_primary()?;
        children[0] = Some(lhs);

        let mut next = self.tokens.pop_front_if(predicate);

        while next.is_some() {
            if ast_node.is_some() {
                // i.e. in > 2 iteration of loop
                for child in &children {
                    ast_node
                        .as_mut()
                        .unwrap()
                        .push_child(child.clone().unwrap()); // WHYYYY
                }
                children[0] = ast_node;
                children[1] = None;
            }
            let operator = next.clone(); // unavoidable since VecDeque 
            let rhs = self.parse_primary()?;

            next = self.tokens.pop_front_if(predicate);

            children[1] = Some(rhs);

            match operator {
                Some(Token::Multiply) => ast_node = Some(ASTNode::new(ASTNodeKind::MultDivOp('*'), None)),
                Some(Token::Divide) => ast_node = Some(ASTNode::new(ASTNodeKind::MultDivOp('/'), None)),
                _ => return Err(self.fail("SOMEHOW. Somehow the operator is none. this is literally impossible. if this matches idk what to do.")),
            }
        }

        for child in children {
            if ast_node.is_none() && child.is_some() {
                ast_node = Some(child.unwrap());
            } else if child.is_some() {
                ast_node.as_mut().unwrap().push_child(child.unwrap())
            }
        }

        match ast_node {
            Some(n) => Ok(n.clone()),
            None => Err(self.fail("failed because ast_node is None")),
        }
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParserError> {
        // "(" additive ")" | UnOp primary | DecIntLit
        let current_token = self.tokens.pop_front();
        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::PrimaryExp, None);

        match current_token {
            Some(Token::DecimalIntegarLiteral(n)) => {
                ast_node.push_child(ASTNode::new(ASTNodeKind::Constant(n.to_string()), None))
            }
            Some(Token::LeftParen) => ast_node = self.parse_paren()?,
            Some(unary_token) => ast_node.push_child(self.parse_unary(unary_token)?), // should be
                                                                                      // unary.
            _ => return Err(self.fail("failed while parsing expression")),
        }

        Ok(ast_node)
    }

    fn parse_paren(&mut self) -> Result<ASTNode, ParserError> {
        let inner_exp = self.parse_additive()?;

        self.expect_token(Token::RightParen)?;

        Ok(inner_exp)
    }

    fn parse_unary(&mut self, current_token: Token) -> Result<ASTNode, ParserError> {
        let ast_node: ASTNode;
        match current_token {
            Token::Minus => {
                ast_node = ASTNode::new(ASTNodeKind::UnOp('-'), Some(vec![self.parse_primary()?]))
            }
            Token::BitwiseComplement => {
                ast_node = ASTNode::new(ASTNodeKind::UnOp('~'), Some(vec![self.parse_primary()?]))
            }
            Token::LogicalNegation => {
                ast_node = ASTNode::new(ASTNodeKind::UnOp('!'), Some(vec![self.parse_primary()?]))
            }
            _ => return Err(self.fail("failed while parsing unary")),
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
