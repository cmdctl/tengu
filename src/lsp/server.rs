use std::collections::HashSet;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::sql_tools::keywords::KEYWORDS;
use crate::sql_tools::tools::{get_tables, Table};
use crate::terminal_ui::repository::FsTenguRepository;

#[derive(Debug)]
struct Backend {
    client: Client,
}

static ALL_TABLES: Lazy<Arc<Mutex<HashSet<Table>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let repo = FsTenguRepository::new();
        let tables = get_tables(repo).await.unwrap();
        let mut all_tables = ALL_TABLES.lock().await;
        for table in tables {
            all_tables.insert(table);
        }
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "tengu-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    ..CompletionOptions::default()
                }),
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let all_tables = ALL_TABLES.lock().await;
        self.client
            .log_message(MessageType::INFO, format!("all_tables: {:?}", all_tables))
            .await;
        let table_completions = || -> Option<Vec<CompletionItem>> {
            let mut table_items = Vec::new();
            for table in all_tables.iter() {
                table_items.push(CompletionItem {
                    label: table.name.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    insert_text: Some(table.name.clone()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..CompletionItem::default()
                });
            }
            Some(table_items)
        }();
        let keyword_completions = || -> Option<Vec<CompletionItem>> {
            let mut keyword_items = Vec::new();
            for keyword in KEYWORDS.iter() {
                keyword_items.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    insert_text: Some(keyword.to_string()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..CompletionItem::default()
                });
            }
            Some(keyword_items)
        }();

        let completions = concat_optional_vecs(table_completions, keyword_completions);
        Ok(completions.map(CompletionResponse::Array))
    }
}

pub async fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn concat_optional_vecs<T>(opt_vec1: Option<Vec<T>>, opt_vec2: Option<Vec<T>>) -> Option<Vec<T>> {
    match (opt_vec1, opt_vec2) {
        (Some(mut vec1), Some(vec2)) => {
            vec1.extend(vec2);
            Some(vec1)
        }
        (Some(vec1), None) => Some(vec1),
        (None, Some(vec2)) => Some(vec2),
        (None, None) => None,
    }
}
