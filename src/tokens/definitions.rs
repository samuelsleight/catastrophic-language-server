use tower_lsp::lsp_types::SemanticTokenType;

const TOKEN_ARRAY: &[SemanticTokenType] = &[
    SemanticTokenType::VARIABLE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::NUMBER,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::COMMENT,
];

pub const fn token_types() -> &'static [SemanticTokenType] {
    TOKEN_ARRAY
}

fn token_index(kind: &SemanticTokenType) -> u32 {
    token_types().iter().position(|item| item == kind).unwrap() as u32
}

pub fn variable() -> u32 {
    token_index(&SemanticTokenType::VARIABLE)
}

pub fn parameter() -> u32 {
    token_index(&SemanticTokenType::PARAMETER)
}

pub fn number() -> u32 {
    token_index(&SemanticTokenType::NUMBER)
}

pub fn operator() -> u32 {
    token_index(&SemanticTokenType::OPERATOR)
}

pub fn comment() -> u32 {
    token_index(&SemanticTokenType::COMMENT)
}
