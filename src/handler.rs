use dashmap::DashMap;
use tower_lsp::{
    jsonrpc::Result as LspResult,
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        InitializeParams, InitializeResult, MessageType, SemanticTokens, SemanticTokensFullOptions,
        SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
        ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    Client, LanguageServer,
};

use crate::{instance::Instance, tokens};

pub struct Handler {
    client: Client,
    instance_map: DashMap<String, Option<Instance>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Handler {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "catastrophic-language-server".into(),
                version: Some("0.0.1".into()),
            }),

            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),

                semantic_tokens_provider: Some(
                    SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: tokens::token_types().to_owned(),
                            token_modifiers: vec![],
                        },

                        full: Some(SemanticTokensFullOptions::Bool(true)),

                        ..Default::default()
                    }
                    .into(),
                ),

                ..Default::default()
            },

            ..Default::default()
        })
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_instance(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.update_instance(params.text_document.uri, &params.content_changes[0].text)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .show_message(
                MessageType::INFO,
                format!("Closed document: {}", params.text_document.uri),
            )
            .await;

        self.instance_map
            .remove(&params.text_document.uri.to_string());
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        Ok(self
            .instance_map
            .get(params.text_document.uri.as_str())
            .and_then(|item| item.value().as_ref().map(Instance::semantic_tokens))
            .map(|tokens| {
                SemanticTokens {
                    result_id: None,
                    data: tokens,
                }
                .into()
            }))
    }
}

impl Handler {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            instance_map: DashMap::new(),
        }
    }

    async fn update_instance(&self, url: Url, source: &str) {
        let mut maybe_instance = self.instance_map.entry(url.to_string()).or_default();
        maybe_instance.take();

        let (new_instance, diagnostics) = Instance::from_source(url.clone(), source);
        *maybe_instance = new_instance;

        self.client
            .publish_diagnostics(url, diagnostics, None)
            .await;
    }
}
