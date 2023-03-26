use catastrophic_ast::ast;
use catastrophic_parser::parser::{Error as ParseError, ParseOutput};

use ruinous_util::{
    error::{context::ErrorProvider, writer::ErrorWriter},
    span::Span,
};

use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, Location, Position, Range, Url,
};

pub struct DiagnosticCollector {
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

    pub fn process_diagnostics(
        uri: Url,
        result: Result<ParseOutput, ParseError>,
    ) -> (Option<ast::Block>, Vec<Diagnostic>) {
        let mut diagnostics = DiagnosticCollector::new(uri);
        let mut ast = None;

        match result {
            Ok(output) => {
                ast = Some(output.ast);

                for error in output.errors {
                    error.write_errors(&mut diagnostics);
                }
            }

            Err(errors) => {
                errors.write_errors(&mut diagnostics);
            }
        }

        (ast, diagnostics.diagnostics)
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
            span.map(convert_span).unwrap_or_default(),
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
                });
        }

        Ok(())
    }
}
