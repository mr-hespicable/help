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

        for child in self.ast.get_children() {
            if matches!(child.get_kind(), ASTNodeKind::FunctionDeclaration(_, _)) {
                let func_asm = &self.generate_function(*child)?;
                self.asm.push_str(func_asm);
            }
        }
        Ok(self.asm.clone())
    }

    fn generate_function(&mut self, mut ast: ASTNode) -> Result<String, GeneratorError> {
        let mut func_str = String::new();
        let ASTNodeKind::FunctionDeclaration(ref _datatype, ref id) = *ast.get_kind() else {
            return Err(self.fail("ASTNode passed to function was not of function type"));
        };
        func_str.push_str(&format!(".globl _{}\n_{}:\n", id, id));

        for child in ast.get_children() {
            if matches!(child.get_kind(), ASTNodeKind::Statement(_)) {
                let statement_asm = &self.generate_statement(*child)?;
                func_str.push_str(statement_asm);
            }
        }

        Ok(func_str)
    }

    fn generate_statement(&mut self, mut ast: ASTNode) -> Result<String, GeneratorError> {
        let mut statement_str = String::new();

        let ASTNodeKind::Statement(r) = ast.get_kind() else {
            return Err(self.fail("ASTNode passed to statement node was not of statement type"));
        };

        if r == "return" {
            for child in ast.get_children() {
                if matches!(child.get_kind(), ASTNodeKind::Expression) {
                    let expression_asm = self.generate_expression(*child)?;
                    statement_str.push_str(&expression_asm);
                }
            }
            statement_str.push_str("ret");
        }

        Ok(statement_str)
    }

    fn generate_expression(&mut self, mut ast: ASTNode) -> Result<String, GeneratorError> {
        // we can safely assume `ast` has ASTNodeKind::Expression because it's only called when
        // the `ast` node has this type.

        let mut expression_str = String::new();
        
        let children = ast.get_children();

        for child in children {
            match child.get_kind() {
                ASTNodeKind::Constant(v) => {
                    expression_str.push_str(&format!("mov w0, #{}\n", v));
                },
                ASTNodeKind::UnOp(op) => {
                    let op_asm = self.generate_unary(*op, *child)?;
                    expression_str.push_str(&op_asm);
                },
                _ => return Err(self.fail("token in expression children was not constant or unary operator"))
            }
        }


        Ok(expression_str)
    }

    fn generate_unary(&mut self, op: char, mut ast: ASTNode) -> Result<String, GeneratorError> {
        let mut unary_str = String::new();

        let children = ast.get_children();

        // we can do this since ASTNodeKind::UnOp should only  have one child
        // let's check, just in case! 
        if children.len() != 1 {
            return Err(self.fail("length of unary children array > 1"));
        }
         
        let inner_exp_asm = self.generate_expression(*children[0].clone())?;
        unary_str.push_str(&inner_exp_asm);
        
        match op {
            '-' => {
                unary_str.push_str("neg w0, w0\n");
            },
            '~' => {
                unary_str.push_str("mvn w0, w0\n");
            },
            '!' => {
                unary_str.push_str("cmp w0, #0\ncset w0, eq\n");
            },
            _ => return Err(self.fail("failed when generating unary"))
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
