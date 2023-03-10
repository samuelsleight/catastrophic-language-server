use catastrophic_ast::ast;
use ruinous_util::span::Span;

use tower_lsp::lsp_types::SemanticToken;

use crate::tokens;

pub struct TokenCollector {
    tokens: Vec<Span<u32>>,
}

impl TokenCollector {
    fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn collect_tokens(source: &ast::Block) -> Vec<SemanticToken> {
        let mut collector = Self::new();

        collector.process_block(source);
        collector.map_semantic_tokens()
    }

    fn map_semantic_tokens(mut self) -> Vec<SemanticToken> {
        self.tokens.sort_by_key(|span| span.start);

        let mut line = 0u32;
        let mut col = 0u32;

        self.tokens
            .into_iter()
            .map(|token| {
                let delta_line = token.start.line as u32 - line;

                if delta_line > 0 {
                    line += delta_line;
                    col = 0;
                }

                let delta_start = token.start.col as u32 - col;

                if delta_start > 0 {
                    col += delta_start;
                }

                SemanticToken {
                    delta_line,
                    delta_start,
                    length: (token.end.col - token.start.col) as u32,
                    token_type: token.data,
                    token_modifiers_bitset: 0,
                }
            })
            .collect()
    }

    fn process_block(&mut self, block: &ast::Block) {
        for comment in &block.comments {
            self.tokens.push(comment.swap(tokens::comment()))
        }

        for arg in &block.args {
            self.tokens.push(arg.swap(tokens::parameter()));
        }

        for symbol in block.symbols.values() {
            self.tokens.push(symbol.name_span.swap(tokens::variable()));
            let value_span = symbol.value.swap(());

            match &symbol.value.data {
                ast::SymbolValue::Number(_) | ast::SymbolValue::Builtin(_) => {
                    self.tokens.push(value_span.swap(tokens::number()));
                }

                ast::SymbolValue::Block(inner) => self.process_block(inner),
            }
        }

        for instr in &block.instrs {
            let instr_span = instr.swap(());

            match &instr.data {
                ast::Instruction::Command(_) => {
                    self.tokens.push(instr_span.swap(tokens::operator()));
                }

                ast::Instruction::Push(value) => match value {
                    ast::InstrValue::Number(_) | ast::InstrValue::Builtin(_) => {
                        self.tokens.push(instr_span.swap(tokens::number()));
                    }

                    ast::InstrValue::Ident(_) => {
                        self.tokens.push(instr_span.swap(tokens::variable()));
                    }

                    ast::InstrValue::Block(inner) => self.process_block(inner),
                },
            }
        }
    }
}
