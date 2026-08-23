use std::borrow::BorrowMut;

#[derive(Default, Debug, Clone)]
pub enum ASTNodeKind {
    #[default]
    Program,
    FunctionDeclaration(String, String), // data_type, identifier
    Statement(String),
    Expression,
    Constant(String),
    UnOp(char), // operator,
}

#[derive(Default, Debug, Clone)]
pub struct ASTNode {
    kind: ASTNodeKind,
    children: Option<Vec<Box<ASTNode>>>,
}

impl ASTNode {
    pub fn new(kind: ASTNodeKind, content: Option<Vec<ASTNode>>) -> Self {
        let children;
        match content {
            Some(c) => children = Some(c.iter().map(|x| Box::new(x.clone())).collect()),
            None => children = None,
        }

        ASTNode {
            kind,
            children,
        }
    }

    pub fn push_child(&mut self, node: ASTNode) {
        match self.children.borrow_mut() {
            Some(v) => v.push(Box::new(node)),
            None => self.children = Some(vec![Box::new(node)])
        }
    }

    pub fn get_kind(&self) -> &ASTNodeKind {
        &self.kind
    }
    
    pub fn get_children(&mut self) -> Vec<Box<ASTNode>>{
        return self.children.as_mut().unwrap().to_vec()
    }
}
