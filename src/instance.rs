use catastrophic_ast::ast;
use catastrophic_parser::parser::Parser;

use tower_lsp::lsp_types::{Diagnostic, SemanticToken, Url};

use crate::{diagnostics::DiagnosticCollector, tokens::TokenCollector};

pub struct Instance {
    source: ast::Block,
}

impl Instance {
    fn new(source: ast::Block) -> Self {
        Self { source }
    }

    pub fn from_source(uri: Url, source: &str) -> (Option<Self>, Vec<Diagnostic>) {
        let parse_result = Parser::with_str(source).parse();
        let (block, diagnostics) = DiagnosticCollector::process_diagnostics(uri, parse_result);
        (block.map(Instance::new), diagnostics)
    }

    pub fn semantic_tokens(&self) -> Vec<SemanticToken> {
        TokenCollector::collect_tokens(&self.source)
    }
}
