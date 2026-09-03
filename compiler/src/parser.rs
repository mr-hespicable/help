use crate::ast::{ASTNode, ASTNodeKind, AssignType, CmpType, EqType, ShiftType};
use crate::errors::ParserError;

use crate::tokens::Token;

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
        match self.tokens.pop_front() {
            Some(ref token) => {
                if !(*token == expected_token) {
                    Err(self.fail(&format!(
                        "failed while expecting token {:?}; was {:?}",
                        expected_token, &token
                    )))
                } else {
                    Ok(())
                }
            }
            _ => Err(self.fail("no tokens left when trying to expect tokens")),
        }
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::Program, None);

        while !self.tokens.is_empty() {
            let function = self.parse_function()?;
            ast_node.push_child(function);
        }
        Ok(ast_node)
    }

    fn parse_function(&mut self) -> Result<ASTNode, ParserError> {
        let mut ast_node: ASTNode;

        let Some(Token::DataType(data_type)) = self.tokens.pop_front() else {
            return Err(self.fail("failed while parsing function"));
        };

        let Some(Token::Identifier(id)) = self.tokens.pop_front() else {
            return Err(self.fail("failed while parsing function"));
        };

        ast_node = ASTNode::new(ASTNodeKind::FunctionDeclaration(data_type, id), None);

        self.expect_token(Token::LeftParen)?;
        self.expect_token(Token::RightParen)?;
        self.expect_token(Token::LeftBrace)?;

        let p = |x: &mut Token| matches!(x, Token::RightBrace);

        // while the next token is not a right brace,
        while self.tokens.pop_front_if(p).is_none() {
            let statement = self.parse_statement()?;
            ast_node.push_child(statement);
        }

        Ok(ast_node)
    }

    fn parse_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("STATEMENT");
        // statement is either:
        //  - a return (with an expression)
        //  - a declaration (with an optional expression if the variable is initialized)
        //      i.e. (int a; int b = 2;)
        //  - an expression (e.g. 2 + 2)
        let mut ast_node: ASTNode;

        let current_token = self.tokens.pop_front();

        match current_token {
            Some(Token::Keyword(k)) => {
                println!("kw");
                if k == "return" {
                    ast_node = ASTNode::new(ASTNodeKind::Return, None);
                    let expression = self.parse_assignment()?;
                    ast_node.push_child(expression);
                    self.expect_token(Token::Semicolon)?;
                } else {
                    return Err(self.fail("failed while parsing statement: keyword not return"));
                }
            }

            Some(Token::Identifier(id)) => {
                println!("id");

                let next_token = self.tokens.pop_front();
                let mut og_node: ASTNode;

                match next_token {
                    Some(Token::SimpleAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Simple), None);
                    },
                    Some(Token::SumAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Sum), None);
                    },
                    Some(Token::DiffAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Diff), None);
                    },
                    Some(Token::ProdAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Prod), None);
                    },
                    Some(Token::QuotAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Diff), None);
                    },
                    Some(Token::RemAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Rem), None);
                    },
                    Some(Token::LShAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::LShift), None);
                    },
                    Some(Token::RShAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::RShift), None);
                    },
                    Some(Token::AndAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::BAnd), None);
                    },
                    Some(Token::XOrAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::BXOr), None);
                    },
                    Some(Token::OrAssign) => {
                        og_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::BOr), None);
                    },
                    _ => return Err(self.fail(&format!("next_token not = when assigning variable, {:?}", next_token))),
                }
                let id_node = ASTNode::new(ASTNodeKind::Variable(id), None);
                og_node.push_child(id_node);

                let inner = self.parse_assignment()?;
                og_node.push_child(inner);
                ast_node = og_node;
                self.expect_token(Token::Semicolon)?;
            }

            // declaration
            Some(Token::DataType(data_type)) => {
                ast_node = ASTNode::new(ASTNodeKind::Declaration(data_type), None);

                let Some(Token::Identifier(id)) = self.tokens.pop_front() else {
                    return Err(self.fail("failed while parsing statement: token mismatch"));
                };

                let next_token = self.tokens.pop_front();

                match next_token {
                    // can only be simple assign
                    Some(Token::SimpleAssign) => {
                        // int a = <TOKENS>
                        let mut og_node =
                            ASTNode::new(ASTNodeKind::Assignment(AssignType::Simple), None);
                        let id_node = ASTNode::new(ASTNodeKind::Variable(id), None);
                        og_node.push_child(id_node);

                        let inner = self.parse_assignment()?;
                        og_node.push_child(inner);
                        ast_node.push_child(og_node);
                        self.expect_token(Token::Semicolon)?;
                    }
                    Some(Token::Semicolon) => {
                        // int a;
                        ast_node.push_child(ASTNode::new(ASTNodeKind::Variable(id), None));
                    }
                    _ => return Err(self.fail("next_token not = or ;")),
                }
                println!("EXPECTING");
                dbg![&self.tokens];

                // TODO: add datatype checking. for now its whatever.
                // TODO: check whether variable is already declared. may use a second passthrough of
                // AST prior to codegen/during maybe.
            }

            _ => {
                return Err(self.fail(&format!(
                    "in parse_statement(): {:?} is not Keyword or Datatype!",
                    current_token
                )));
            }
        }

        Ok(ast_node)
    }

    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        let mut ast_node: ASTNode;
        println!("TOKS");
        dbg![&self.tokens];
        let current_token = self
            .tokens
            .pop_front_if(|x| matches!(x, Token::Identifier(_)));

        match current_token {
            Some(Token::Identifier(ref id)) => {
                let is_assign = |x: &mut Token| {
                    matches!(
                        x,
                        Token::SimpleAssign
                            | Token::SumAssign
                            | Token::DiffAssign
                            | Token::ProdAssign
                            | Token::QuotAssign
                            | Token::RemAssign
                            | Token::LShAssign
                            | Token::RShAssign
                            | Token::AndAssign
                            | Token::XOrAssign
                            | Token::OrAssign
                    )
                };
                let next = self.tokens.pop_front_if(is_assign);
                match next {
                    Some(Token::SimpleAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Simple), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::SumAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Sum), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::DiffAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Diff), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::ProdAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Prod), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::QuotAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Quot), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::RemAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Rem), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::LShAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::LShift), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::RShAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::RShift), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::AndAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::BAnd), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::XOrAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::BXOr), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    Some(Token::OrAssign) => {
                        ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::BOr), None);
                        ast_node.push_child(self.parse_variable(id.to_string())?);
                        ast_node.push_child(self.parse_assignment()?);
                    }
                    _ => {
                        // math expr
                        // ast_node = ASTNode::new(ASTNodeKind::Assignment(AssignType::Simple), None);
                        self.tokens
                            .push_front(current_token.expect("can't be none, son!"));
                        ast_node = self.parse_logical_or()?;
                    }
                }
            }
            _ => {
                ast_node = self.parse_logical_or()?;
            }
        }

        Ok(ast_node)
    }

    // this function is the entry for binary expression processing.
    fn parse_logical_or(&mut self) -> Result<ASTNode, ParserError> {
        // logical_or || logical_or | logical_and
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::LogicalOr;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_logical_and()?;
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
            let rhs = self.parse_logical_and()?;

            match operator {
                Some(Token::LogicalOr) => ast_node = Some(ASTNode::new(ASTNodeKind::LOrOp, None)),
                _ => {
                    return Err(
                        self.fail("LogicalOr operator is not logical or (regex issue most likely)")
                    );
                }
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

    fn parse_logical_and(&mut self) -> Result<ASTNode, ParserError> {
        // logical_and && logical_and | logical_and
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::LogicalAnd;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_bitwise_or()?;
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
            let rhs = self.parse_bitwise_or()?;

            match operator {
                Some(Token::LogicalAnd) => ast_node = Some(ASTNode::new(ASTNodeKind::LAndOp, None)),
                _ => {
                    return Err(self
                        .fail("LogicalAnd operator is not logical and (regex issue most likely)"));
                }
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

    fn parse_bitwise_or(&mut self) -> Result<ASTNode, ParserError> {
        // bitwise_or && bitwise_or | bitwise_xor
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::BitwiseOr;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_bitwise_xor()?;
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
            let rhs = self.parse_bitwise_xor()?;

            match operator {
                Some(Token::BitwiseOr) => ast_node = Some(ASTNode::new(ASTNodeKind::BOrOp, None)),
                _ => {
                    return Err(
                        self.fail("BitwiseOr operator is not bitwise or (regex issue most likely)")
                    );
                }
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

    fn parse_bitwise_xor(&mut self) -> Result<ASTNode, ParserError> {
        // bitwise_xor && bitwise_xor | bitwise_and
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::BitwiseExclusiveOr;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_bitwise_and()?;
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
            let rhs = self.parse_bitwise_and()?;

            match operator {
                Some(Token::BitwiseExclusiveOr) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::BXOrOp, None))
                }
                _ => {
                    return Err(self
                        .fail("BitwiseXOr operator is not bitwise xor (regex issue most likely)"));
                }
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

    fn parse_bitwise_and(&mut self) -> Result<ASTNode, ParserError> {
        // bitwise_and && bitwise_and | equative
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::BitwiseAnd;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_equative()?;
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
            let rhs = self.parse_equative()?;

            match operator {
                Some(Token::BitwiseAnd) => ast_node = Some(ASTNode::new(ASTNodeKind::BAndOp, None)),
                _ => {
                    return Err(self
                        .fail("BitwiseAnd operator is not bitwise and (regex issue most likely)"));
                }
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

    fn parse_equative(&mut self) -> Result<ASTNode, ParserError> {
        // equative (== / !=) equative | cmpative
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::Eq || x == &Token::Ne;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_comparative()?;
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
            let rhs = self.parse_comparative()?;

            match operator {
                Some(Token::Eq) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::EqOp(EqType::Eq), None))
                }
                Some(Token::Ne) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::EqOp(EqType::Ne), None))
                }
                _ => return Err(self.fail("Eq/Ne operator is not Eq/Ne (regex issue most likely)")),
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

    fn parse_comparative(&mut self) -> Result<ASTNode, ParserError> {
        // cmpative ( <= | >= | < | > ) cmpative | shift
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| {
            x == &Token::Ge || x == &Token::Le || x == &Token::Gt || x == &Token::Lt
        };

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_shift()?;
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
            let rhs = self.parse_shift()?;

            match operator {
                Some(Token::Ge) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::CmpOp(CmpType::Ge), None))
                }
                Some(Token::Le) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::CmpOp(CmpType::Le), None))
                }
                Some(Token::Gt) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::CmpOp(CmpType::Gt), None))
                }
                Some(Token::Lt) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::CmpOp(CmpType::Lt), None))
                }
                _ => {
                    return Err(
                        self.fail("cmpative operator is not cmpative (regex issue most likely)")
                    );
                }
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

    fn parse_shift(&mut self) -> Result<ASTNode, ParserError> {
        // shift  ( << | >> ) shift | additive
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::LeftShift || x == &Token::RightShift;

        let mut children: [Option<ASTNode>; 2] = [None, None];

        let lhs = self.parse_additive()?;
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
            let rhs = self.parse_additive()?;

            match operator {
                Some(Token::LeftShift) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::ShiftOp(ShiftType::Left), None))
                }
                Some(Token::RightShift) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::ShiftOp(ShiftType::Right), None))
                }
                _ => return Err(self.fail("shift operator is not shift (regex issue most likely)")),
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

    fn parse_additive(&mut self) -> Result<ASTNode, ParserError> {
        // additive ( + or - ) additive | multiplicative
        let mut ast_node: Option<ASTNode> = None;

        let predicate = |x: &mut Token| x == &Token::Plus || x == &Token::Minus;

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
                Some(Token::Plus) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::AddSubOp('+'), None))
                }
                Some(Token::Minus) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::AddSubOp('-'), None))
                }
                _ => return Err(self.fail(&format!("{:?} operator is not additive", operator))),
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

        let predicate = |x: &mut Token| x == &Token::Times || x == &Token::Divide;

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
                Some(Token::Times) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::MultDivOp('*'), None))
                }
                Some(Token::Divide) => {
                    ast_node = Some(ASTNode::new(ASTNodeKind::MultDivOp('/'), None))
                }
                _ => {
                    return Err(
                        self.fail(&format!("{:?} operator is not multiplicative", operator))
                    );
                }
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
        // "(" additive ")" | UnOp primary | DecIntLit | Variable
        let current_token = self.tokens.pop_front();
        let mut ast_node: ASTNode = ASTNode::new(ASTNodeKind::PrimaryExp, None);

        match current_token {
            Some(Token::DecimalIntegerLiteral(n)) => {
                ast_node.push_child(ASTNode::new(ASTNodeKind::Constant(n.to_string()), None))
            }
            Some(Token::LeftParen) => ast_node = self.parse_paren()?,
            Some(Token::Identifier(id)) => ast_node = self.parse_variable(id)?,
            Some(unary_token) => ast_node.push_child(self.parse_unary(unary_token)?),
            _ => {
                return Err(self.fail(&format!(
                    "failed while parsing primary: {:?} is not valid token",
                    current_token
                )));
            }
        }

        Ok(ast_node)
    }

    fn parse_variable(&mut self, id: String) -> Result<ASTNode, ParserError> {
        Ok(ASTNode::new(ASTNodeKind::Variable(id), None))
    }

    fn parse_paren(&mut self) -> Result<ASTNode, ParserError> {
        let inner_exp = self.parse_logical_or()?;

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
            _ => {
                return Err(self.fail(&format!(
                    "failed while parsing unary: {:?} is not valid token\n{:?}",
                    current_token, &self.tokens,
                )));
            }
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
