//! KiCad file format handling utilities

/// KiCad S-expression token.
#[derive(Debug, Clone)]
pub struct SexprToken {
    pub kind: SexprKind,
    pub line: u32,
}

/// Kind of S-expression token.
#[derive(Debug, Clone)]
pub enum SexprKind {
    Open(String),
    Close,
    Atom(String),
}

/// Parse a simple KiCad S-expression string into a tree structure.
pub fn parse_sexpr(input: &str) -> Result<SexprNode, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    if !trimmed.starts_with('(') {
        return Err("Expected opening parenthesis".into());
    }
    parse_node(trimmed, &mut 0)
}

/// A node in the KiCad S-expression tree.
#[derive(Debug, Clone)]
pub struct SexprNode {
    pub tag: String,
    pub children: Vec<SexprChild>,
}

/// A child element in the S-expression tree.
#[derive(Debug, Clone)]
pub enum SexprChild {
    Node(SexprNode),
    Atom(String),
}

fn parse_node(input: &str, pos: &mut usize) -> Result<SexprNode, Box<dyn std::error::Error>> {
    // Skip whitespace and opening paren
    while *pos < input.len() && input.chars().nth(*pos) == Some(' ') {
        *pos += 1;
    }
    if input.chars().nth(*pos) != Some('(') {
        return Err("Expected '('".into());
    }
    *pos += 1;

    // Read tag
    let tag_start = *pos;
    while *pos < input.len() && !input.chars().nth(*pos).map(|c| c.is_whitespace() || c == ')').unwrap_or(false) {
        *pos += 1;
    }
    let tag = input[tag_start..*pos].to_string();
    let mut children = Vec::new();

    // Read children until closing paren
    loop {
        while *pos < input.len() && input.chars().nth(*pos).map(|c| c.is_whitespace()).unwrap_or(false) {
            *pos += 1;
        }
        if *pos >= input.len() {
            return Err("Unexpected end of input".into());
        }
        if input.chars().nth(*pos) == Some(')') {
            *pos += 1;
            break;
        }
        if input.chars().nth(*pos) == Some('(') {
            children.push(SexprChild::Node(parse_node(input, pos)?));
        } else {
            let atom_start = *pos;
            while *pos < input.len() && !input.chars().nth(*pos).map(|c| c.is_whitespace() || c == ')').unwrap_or(false) {
                *pos += 1;
            }
            children.push(SexprChild::Atom(input[atom_start..*pos].to_string()));
        }
    }

    Ok(SexprNode { tag, children })
}