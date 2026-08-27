use crate::{
    ast::{ASTNode, ASTNodeKind},
    errors::GeneratorError,
};

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
        // ".globl _" + function_name
        // "_" + function_name
        // "mov w0, #" + ret val
        // "ret"

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
            if matches!(child.get_kind(), ASTNodeKind::Statement(_)) {
                let statement_asm = &self.generate_statement(*child)?;
                func_str.push_str(statement_asm);
            }
        }

        Ok(func_str)
    }

    fn generate_statement(&mut self, ast: ASTNode) -> Result<String, GeneratorError> {
        let mut statement_str = String::new();

        let ASTNodeKind::Statement(r) = ast.get_kind() else {
            return Err(self.fail("ASTNode passed to statement node was not of statement type"));
        };

        if r == "return" {
            for child in ast.get_subtrees() {
                match child.get_kind() {
                    ASTNodeKind::PrimaryExp => {
                        let expression_asm = self.generate_primary(&*child)?;
                        statement_str.push_str(&expression_asm);
                    }
                    ASTNodeKind::AddSubOp(_) | ASTNodeKind::MultDivOp(_) => {
                        let expression_asm = self.generate_arithmetic(&*child, 0)?;
                        statement_str.push_str(&expression_asm);
                    }
                    _ => unimplemented!(),
                }
            }
            statement_str.push_str("ret");
        }

        Ok(statement_str)
    }

    fn generate_arithmetic(&mut self, ast: &ASTNode, rhs: usize) -> Result<String, GeneratorError> {
        let mut arithmetic_string = String::new();

        let (&ASTNodeKind::AddSubOp(op) | &ASTNodeKind::MultDivOp(op)) = ast.get_kind() else {
            return Err(
                self.fail("ASTNode passed to generate_arithmetic was not an arithmetic operator")
            );
        };

        let children = ast.get_subtrees();

        if children.len() != 2 {
            return Err(self.fail("arithmetic operator ASTNode does not have two children"));
        }

        let mut lhs_string = String::new();
        let mut rhs_string = String::new();

        for t in children.iter().enumerate() {
            let c = t.1;

            let subtree = *c.clone();

            if subtree.get_kind() != &ASTNodeKind::PrimaryExp {
                    if t.0 == 0 {
                        lhs_string.push_str(&self.generate_arithmetic(&subtree, t.0)?);
                        lhs_string.push('\n');
                    } else {
                        rhs_string.push_str(&self.generate_arithmetic(&subtree, t.0)?);
                        rhs_string.push('\n');
                    }
            } else {
                let subsubtree = subtree.get_subtrees();

                if subsubtree.len() != 1 {
                    return Err(self.fail("PrimaryExp ASTNode should only have one child."));
                }

                if t.0 == 0 {
                    lhs_string.push_str(&self.generate_primary_with_register(&subtree, &format!("w{}", t.0))?);
                } else {
                    rhs_string.push_str(&self.generate_primary_with_register(&subtree, &format!("w{}", t.0))?);
                }
            }
        }

        if children[0].get_kind() == &ASTNodeKind::PrimaryExp && children[1].get_kind() != &ASTNodeKind::PrimaryExp {
            arithmetic_string.push_str(&rhs_string);
            arithmetic_string.push_str(&lhs_string);
        } else {
            arithmetic_string.push_str(&lhs_string);
            arithmetic_string.push_str(&rhs_string);
        }

        match op {
            '+' => arithmetic_string.push_str(&format!("add w{}, w0, w1\n", rhs)),
            '-' => arithmetic_string.push_str(&format!("sub w{}, w0, w1\n", rhs)),
            '*' => arithmetic_string.push_str(&format!("mul w{}, w0, w1\n", rhs)),
            '/' => arithmetic_string.push_str(&format!("udiv w{}, w0, w1\n", rhs)),
            _ => return Err(self.fail("arithmetic operator was invalid")),
        }

        Ok(arithmetic_string)
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

    fn generate_primary(&mut self, ast: &ASTNode) -> Result<String, GeneratorError> {
        self.generate_primary_with_register(ast, "w0")
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
                let inner_exp_asm = self.generate_primary_with_register(&*children[0].clone(), register)?;
                unary_str.push_str(&inner_exp_asm);
            },
            _ => {
                let exp_asm = self.generate_arithmetic(&child, 0)?;
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
