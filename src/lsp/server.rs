use std::collections::HashSet;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::sql_tools::tools::{get_tables, Table};
use crate::terminal_ui::models::Connection;
use crate::terminal_ui::repository::{FsTenguRepository, TenguRepository};

#[derive(Debug)]
struct Backend<R: TenguRepository> {
    client: Client,
    repo: R,
    active_connection: Option<Connection>,
}

static ALL_TABLES: Lazy<Arc<Mutex<HashSet<Table>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

#[tower_lsp::async_trait]
impl LanguageServer for Backend<FsTenguRepository> {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let active_connection = self.repo.get_active_connection();
        self.active_connection = active_connection;
        let tables = get_tables(self.repo).await.unwrap();
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
        let mut items = Vec::new();
        let completions = || -> Option<Vec<CompletionItem>> {
            for table in all_tables.iter() {
                items.push(CompletionItem {
                    label: table.name.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    insert_text: Some(table.name.clone()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..CompletionItem::default()
                });
            }
            Some(items)
        }();
        Ok(completions.map(CompletionResponse::Array))
    }
}

pub async fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let repo = FsTenguRepository::new();
    let active_connection = repo.get_active_connection();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        repo,
        active_connection,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
