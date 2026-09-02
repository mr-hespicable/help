use crate::{
    ast::{ASTNode, ASTNodeKind, CmpType, EqType, ShiftType}, errors::GeneratorError,
};

use nanoid::nanoid;

pub struct Generator {
    ast: ASTNode,
    asm: String,
}

impl Generator {
    pub fn new(ast: ASTNode) -> Generator {
        Generator {
            ast,
            asm: String::new(),
        }
    }

    pub fn generate(&mut self) -> Result<String, GeneratorError> {
        if !matches!(self.ast.get_kind(), ASTNodeKind::Program) {
            return Err(self.fail("starting node is not Program node"));
        }

        let children: Vec<Box<ASTNode>> = self.ast.get_subtrees();

        for child in children {
            if matches!(child.get_kind(), ASTNodeKind::FunctionDeclaration(_, _)) {
                let func_asm = self.generate_function(&child)?;
                self.asm.push_str(&func_asm);
            }
        }
        Ok(self.asm.clone())
    }

    fn generate_function(&mut self, ast: &ASTNode) -> Result<String, GeneratorError> {
        let mut func_str = String::new();
        let ASTNodeKind::FunctionDeclaration(ref _datatype, ref id) = *ast.get_kind() else {
            return Err(self.fail("ASTNode passed to function was not of function type"));
        };
        func_str.push_str(&format!(".globl _{}\n_{}:\n", id, id));

        for child in ast.get_subtrees() {
            if matches!(child.get_kind(), ASTNodeKind::Statement) {
                let statement_asm = &self.generate_statement(&*child)?;
                func_str.push_str(statement_asm);
            }
        }

        Ok(func_str)
    }

    fn generate_statement(&mut self, ast: &ASTNode) -> Result<String, GeneratorError> {
        let mut statement_str = String::new();

        let ASTNodeKind::Return = ast.get_kind() else {
            return Err(self.fail("ASTNode passed to statement node was not of statement type"));
        };

        for child in ast.get_subtrees() {
            match child.get_kind() {
                ASTNodeKind::PrimaryExp => {
                    let expression_asm = self.generate_primary_with_register(&*child, "w0")?;
                    statement_str.push_str(&expression_asm);
                }

                ASTNodeKind::UnOp(_)
                | ASTNodeKind::MultDivOp(_)
                | ASTNodeKind::AddSubOp(_)
                | ASTNodeKind::ShiftOp(_)
                | ASTNodeKind::CmpOp(_)
                | ASTNodeKind::EqOp(_)
                | ASTNodeKind::BAndOp
                | ASTNodeKind::BXOrOp
                | ASTNodeKind::BOrOp
                | ASTNodeKind::LAndOp
                | ASTNodeKind::LOrOp => {
                    let expression_asm = self.generate_binary_op(&*child, "w0")?;
                    statement_str.push_str(&expression_asm);
                }
                _ => unimplemented!(),
            }
        }
        statement_str.push_str("ret");

        Ok(statement_str)
    }

    fn generate_binary_op(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        let mut arithmetic_string = String::new();

        if ast.get_subtrees().len() != 2 {
            return Err(self.fail("arithmetic operator ASTNode does not have two children"));
        }

        let children = ast.get_subtrees();

        let mut lhs_string = String::new();
        let mut rhs_string = String::new();

        for t in children.iter().enumerate() {
            let c = t.1;

            let subtree = *c.clone();

            if subtree.get_kind() != &ASTNodeKind::PrimaryExp {
                if t.0 == 0 {
                    lhs_string.push_str(&self.generate_binary_op(&subtree, &format!("w{}", t.0))?);
                    lhs_string.push('\n');
                } else {
                    rhs_string.push_str(&self.generate_binary_op(&subtree, &format!("w{}", t.0))?);
                    rhs_string.push('\n');
                }
            } else {
                let subsubtree = subtree.get_subtrees();

                if subsubtree.len() != 1 {
                    return Err(self.fail("PrimaryExp ASTNode should only have one child."));
                }

                if t.0 == 0 {
                    lhs_string.push_str(
                        &self.generate_primary_with_register(&subtree, &format!("w{}", t.0))?,
                    );
                } else {
                    rhs_string.push_str(
                        &self.generate_primary_with_register(&subtree, &format!("w{}", t.0))?,
                    );
                }
            }
        }

        // specific edge case to ensure w0 is not overwritten
        if children[0].get_kind() == &ASTNodeKind::PrimaryExp
            && children[1].get_kind() != &ASTNodeKind::PrimaryExp
        {
            arithmetic_string.push_str(&rhs_string);
            arithmetic_string.push_str(&lhs_string);
        } else {
            arithmetic_string.push_str(&lhs_string);
            arithmetic_string.push_str(&rhs_string);
        }

        // now lhs is in w0, and rhs is in w1. both lhs and rhs are consts.

        match ast.get_kind() {
            &ASTNodeKind::AddSubOp(_) | &ASTNodeKind::MultDivOp(_) => {
                arithmetic_string.push_str(&self.generate_arithmetic(ast, register)?)
            }

            &ASTNodeKind::LOrOp | &ASTNodeKind::LAndOp => {
                arithmetic_string.push_str(&self.generate_logical(ast, register)?)
            }

            &ASTNodeKind::BOrOp | &ASTNodeKind::BXOrOp | &ASTNodeKind::BAndOp => {
                arithmetic_string.push_str(&self.generate_bitwise(ast, register)?)
            }

            &ASTNodeKind::EqOp(_) => {
                arithmetic_string.push_str(&self.generate_equative(ast, register)?)
            }

            &ASTNodeKind::CmpOp(_) => {
                arithmetic_string.push_str(&self.generate_comparative(ast, register)?)
            }

            &ASTNodeKind::ShiftOp(_) => {
                arithmetic_string.push_str(&self.generate_shift(ast, register)?)
            }

            _ => unimplemented!(),
        }
        arithmetic_string.push_str("\n");

        Ok(arithmetic_string)
    }

    fn generate_arithmetic(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        let (&ASTNodeKind::AddSubOp(op) | &ASTNodeKind::MultDivOp(op)) = ast.get_kind() else {
            return Err(
                self.fail("generate_arithmetic() called with ast node which is not arithmetic")
            );
        };
        match op {
            '+' => Ok(format!("add {}, w0, w1", register)),
            '-' => Ok(format!("sub {}, w0, w1", register)),
            '*' => Ok(format!("mul {}, w0, w1", register)),
            '/' => Ok(format!("udiv {}, w0, w1", register)),
            _ => Err(self.fail(&format!("invalid arithmetic operator {}", op))),
        }
    }

    fn generate_logical(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        let start_label_name = nanoid!();
        let end_label_name = nanoid!();
        match ast.get_kind() {
            // w0 = lhs; w1 = rhs
            ASTNodeKind::LAndOp => {
                Ok([
                    "cmp w0, #0",
                    &format!("b.ne _{}", start_label_name), // lhs != 0?
                    &format!("mov {}, #0", register),       // if not, lhs && rhs = 0
                    &format!("jmp _{}", end_label_name),
                    &format!("_{}", start_label_name), // lhs != 0
                    "cmp w1, #0",
                    &format!("cset {}, ne", register), // rhs != 0? if so, lhs && rhs = 1.
                    &format!("_{}", end_label_name),   // otherwise, lhs && rhs = 0.
                ]
                .join("\n"))
            }
            ASTNodeKind::LOrOp => {
                Ok([
                    "cmp w0, #0",
                    &format!("b.eq _{}", start_label_name), // lhs == 0?
                    &format!("mov {}, #1", register),       // if not, lhs || rhs == 1
                    &format!("jmp _{}", end_label_name),
                    &format!("_{}", start_label_name), // lhs == 0
                    "cmp w1, #0",
                    &format!("cset {}, ne", register), // rhs != 0? if so, lhs || rhs == 1
                    &format!("_{}", end_label_name),   // otherwise, lhs || rhs == 0
                ]
                .join("\n"))
            }
            _ => Err(self.fail("generate_logical() called with ast node which is not logical")),
        }
    }

    fn generate_bitwise(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        match ast.get_kind() {
            ASTNodeKind::BAndOp => Ok(format!("and {}, w0, w1", register)),
            ASTNodeKind::BOrOp => Ok(format!("orr {}, w0, w1", register)),
            ASTNodeKind::BXOrOp => Ok(format!("eor {}, w0, w1", register)),
            _ => Err(self.fail("generate_bitwise() called with ast node which is not bitwise")),
        }
    }

    fn generate_equative(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        let ASTNodeKind::EqOp(eq_type) = ast.get_kind() else {
            return Err(self.fail("generate_equative called with ast node which is not equative"));
        };
        match eq_type {
            EqType::Eq => Ok(["cmp w0, w1", &format!("cset {}, eq", register)].join("\n")),
            EqType::Ne => Ok(["cmp w0, w1", &format!("cset {}, ne", register)].join("\n")),
        }
    }

    fn generate_comparative(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        let ASTNodeKind::CmpOp(cmp_type) = ast.get_kind() else {
            return Err(
                self.fail("generate_comparative called with ast node which is not comparative")
            );
        };
        match cmp_type {
            CmpType::Ge => Ok(["cmp w0, w1", &format!("cset {}, ge", register)].join("\n")),
            CmpType::Le => Ok(["cmp w0, w1", &format!("cset {}, le", register)].join("\n")),
            CmpType::Gt => Ok(["cmp w0, w1", &format!("cset {}, gt", register)].join("\n")),
            CmpType::Lt => Ok(["cmp w0, w1", &format!("cset {}, lt", register)].join("\n")),
        }
    }

    fn generate_shift(&mut self, ast: &ASTNode, register: &str) -> Result<String, GeneratorError> {
        let ASTNodeKind::ShiftOp(shift_type) = ast.get_kind() else {
            return Err(
                self.fail("generate_shift called with ast node which is not a shift operation"));
        };            
        match shift_type {
            ShiftType::Right => Ok(format!("lsr {}, w0, w1", register)),
            ShiftType::Left => Ok(format!("lsl {}, w0, w1", register)),
        }
    }


    fn generate_primary_with_register(
        &mut self,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        // we can safely assume `ast` has ASTNodeKind:: because it's only called when
        // the `ast` node has this type.

        let mut expression_str = String::new();

        let children = ast.get_subtrees();

        for child in children {
            match child.get_kind() {
                ASTNodeKind::Constant(v) => {
                    expression_str.push_str(&format!("mov {}, #{}\n", register, v));
                }
                ASTNodeKind::UnOp(op) => {
                    let op_asm = self.generate_unary(&op, &child, register)?;
                    expression_str.push_str(&op_asm);
                }
                _ => {
                    return Err(
                        self.fail("token in primary children was not constant or unary operator")
                    );
                }
            }
        }

        Ok(expression_str)
    }

    fn generate_unary(
        &mut self,
        op: &char,
        ast: &ASTNode,
        register: &str,
    ) -> Result<String, GeneratorError> {
        let mut unary_str = String::new();

        let children = ast.get_subtrees();
        let child = &*children[0].clone();

        // we can do this since ASTNodeKind::UnOp should only  have one child
        // let's check, just in case!
        if children.len() != 1 {
            return Err(self.fail("length of unary children array > 1"));
        }

        match child.get_kind() {
            ASTNodeKind::PrimaryExp => {
                let inner_exp_asm =
                    self.generate_primary_with_register(&*children[0].clone(), register)?;
                unary_str.push_str(&inner_exp_asm);
            }
            _ => {
                let exp_asm = self.generate_binary_op(&child, register)?;
                unary_str.push_str(&exp_asm);
            }
        }

        match op {
            '-' => {
                unary_str.push_str(&format!("neg {}, {}\n", register, register));
            }
            '~' => {
                unary_str.push_str(&format!("mvn {}, {}\n", register, register));
            }
            '!' => {
                unary_str.push_str(&format!("cmp {}, #0\ncset {}, eq\n", register, register));
            }
            _ => return Err(self.fail("failed when generating unary")),
        }

        Ok(unary_str)
    }

    #[track_caller]
    fn fail(&self, msg: &str) -> GeneratorError {
        let caller_location = std::panic::Location::caller();
        let err_string = msg.to_string();
        GeneratorError(format!(
            "{}; in {} at {}:{}",
            err_string,
            caller_location.file(),
            caller_location.line(),
            caller_location.column()
        ))
    }
}
