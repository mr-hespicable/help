use std::borrow::BorrowMut;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ASTNodeKind {
    #[default]
    Program,
    FunctionDeclaration(String, String), // data_type, identifier
    Statement(String),
    PrimaryExp,
    Constant(String),
    UnOp(char),      // operator,
    MultDivOp(char), // * or /
    AddSubOp(char),  // + or -
}

#[derive(Default, Clone)]
pub struct ASTNode {
    kind: ASTNodeKind,
    children: Option<Vec<Box<ASTNode>>>,
}

impl std::fmt::Debug for ASTNode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fn pprint_inner(spaces: u8, child: ASTNode) -> String {
            let mut pstring = String::new();

            for _ in 0..spaces {
                pstring.push(' ');
            }

            pstring.push_str(&format!("{:?}", child.get_kind()));
            pstring.push('\n');

            for c in child.get_subtrees() {
                pstring.push_str(&pprint_inner(spaces + 4, *c.clone()));
                pstring.push('\n')
            }

            pstring
        }
        return write!(f, "\n{}", &pprint_inner(0, self.clone()));
    }
}

impl ASTNode {
    pub fn new(kind: ASTNodeKind, content: Option<Vec<ASTNode>>) -> Self {
        let children;
        match content {
            Some(c) => children = Some(c.iter().map(|x| Box::new(x.clone())).collect()),
            None => children = None,
        }

        ASTNode { kind, children }
    }

    pub fn push_child(&mut self, node: ASTNode) {
        match self.children.borrow_mut() {
            Some(v) => v.push(Box::new(node)),
            None => self.children = Some(vec![Box::new(node)]),
        }
    }

    pub fn get_kind(&self) -> &ASTNodeKind {
        &self.kind
    }

    pub fn get_subtrees(&self) -> Vec<Box<ASTNode>> {
        return self.children.clone().unwrap_or(Vec::new());
    }
}
