use catastrophic_ast::ast;
use catastrophic_parser::parser::Parser;
use ruinous_util::{
    error::{context::ErrorProvider, writer::ErrorWriter},
    span::{Location as Loc, Span},
};

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, Location, Position, Range, Url,
};

pub struct Instance {
    source: ast::Block,
}

impl Instance {
    pub fn from_source(url: Url, source: String) -> (Option<Self>, Vec<Diagnostic>) {
        let parse_result = Parser::with_str(&source).parse();

        let mut diagnostics = DiagnosticCollector::new(url);
        let mut result = None;

        match parse_result {
            Ok(ast) => result = Some(Instance { source: ast }),
            Err(errors) => {
                errors.write_errors(&mut diagnostics);
            }
        }

        (result, diagnostics.diagnostics)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {}
}

struct DiagnosticCollector {
    uri: Url,
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    fn new(uri: Url) -> Self {
        Self {
            uri,
            diagnostics: Vec::new(),
        }
    }
}

fn convert_span(span: Span<()>) -> Range {
    Range::new(
        Position::new(span.start.line as u32, span.start.col as u32),
        Position::new(span.end.line as u32, span.end.col as u32),
    )
}

impl ErrorWriter for DiagnosticCollector {
    fn error(&mut self, span: Option<Span<()>>, message: &str) -> std::fmt::Result {
        self.diagnostics.push(Diagnostic::new_simple(
            convert_span(span.unwrap_or(Span::new(Loc::new(0, 0), Loc::new(0, 0), ()))),
            message.into(),
        ));

        Ok(())
    }

    fn note(&mut self, span: Span<()>, message: &str) -> std::fmt::Result {
        if let Some(diagnostic) = self.diagnostics.last_mut() {
            diagnostic
                .related_information
                .get_or_insert_with(Vec::new)
                .push(DiagnosticRelatedInformation {
                    message: message.into(),
                    location: Location::new(self.uri.clone(), convert_span(span)),
                })
        }

        Ok(())
    }
}
