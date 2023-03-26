use std::sync::{Arc, RwLock};

use catastrophic_ast::ast;
use catastrophic_parser::parser::Parser;

use tokio::task::JoinHandle;
use tower_lsp::lsp_types::{Diagnostic, SemanticToken, Url};

use crate::{diagnostics::DiagnosticCollector, tokens::TokenCollector};

enum TaskResult<T> {
    Task(Option<JoinHandle<T>>),
    Value(T),
}

pub struct Instance {
    source: Arc<RwLock<ast::Block>>,

    tokens_task: TaskResult<Vec<SemanticToken>>,
}

impl Instance {
    fn new(source: ast::Block) -> Self {
        let source = Arc::new(RwLock::new(source));

        let tokens_task = {
            let source = source.clone();
            tokio::task::spawn_blocking(move || {
                let source = source.as_ref();
                let lock = source.read().unwrap();
                TokenCollector::collect_tokens(&lock)
            })
        };

        Self {
            source,
            tokens_task: TaskResult::Task(Some(tokens_task)),
        }
    }

    pub async fn from_source(uri: Url, source: &str) -> (Option<Self>, Vec<Diagnostic>) {
        let source = source.to_owned();
        let parse_result =
            tokio::task::spawn_blocking(move || Parser::with_str(&source).permissive(true).parse())
                .await
                .unwrap();

        let (block, diagnostics) = DiagnosticCollector::process_diagnostics(uri, parse_result);
        (block.map(Instance::new), diagnostics)
    }

    pub async fn semantic_tokens(&mut self) -> Vec<SemanticToken> {
        match &mut self.tokens_task {
            TaskResult::Task(handle) => {
                let handle = handle.take();
                let value = handle.unwrap().await.unwrap();
                self.tokens_task = TaskResult::Value(value.clone());
                value
            }

            TaskResult::Value(value) => value.clone(),
        }
    }
}
