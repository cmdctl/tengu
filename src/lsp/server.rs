use std::collections::HashSet;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use super::cache::{ALL_COLUMNS, TABLES_IN_FILE};
use super::document::get_word_at_position;
use super::file_watch::async_watch;
use crate::db::service::{Service, TenguService};
use crate::lsp::cache::{reset_cache, ALL_TABLES};
use crate::prelude::*;
use crate::terminal_ui::repository::{FsTenguRepository, TenguRepository};
use crate::tokenizer::{intersection, tokenize, Token};

#[derive(Debug)]
struct Backend {
    client: Client,
    service: TenguService,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let tables = self.service.get_tables().await.unwrap_or(vec![]);
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: None,
                    completion_item: Some(CompletionOptionsCompletionItem {
                        label_details_support: Some(true),
                    }),
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let all_tables = ALL_TABLES.lock().await;
        let mut tables_in_file = TABLES_IN_FILE.lock().await;
        let mut all_columns = ALL_COLUMNS.lock().await;
        let mut completions = || -> Option<Vec<CompletionItem>> {
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
            for keyword in self.service.get_keywords().iter() {
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
        completions.concat(&keyword_completions);
        let sql_file_path = params
            .text_document_position
            .text_document
            .uri
            .to_file_path()
            .unwrap();
        let Ok(sql_file_content) = read_file_to_string(sql_file_path) else {
            return Ok(completions.map(CompletionResponse::Array));
        };

        let content_tokens: Vec<_> = tokenize(sql_file_content.clone())
            .iter()
            .filter(|&t| if let Token::Token(_) = t { true } else { false })
            .map(|t| match t {
                Token::Token(t) => t.clone(),
                _ => unreachable!(),
            })
            .collect();
        let tables = all_tables.iter().map(|t| t.clone().name).collect();
        let tables_to_query = intersection(tables, content_tokens);
        let tables_to_query = all_tables
            .iter()
            .filter(|&t| tables_to_query.contains(&t.name))
            .map(|t| t.clone())
            .collect();
        if tables_in_file.equals(&tables_to_query) {
            completions.concat(&Some(all_columns.clone()));
            return Ok(completions.map(CompletionResponse::Array));
        } else {
            all_columns.clear();
            tables_in_file.clear();
            for table in tables_to_query.iter() {
                tables_in_file.insert(table.clone());
            }
        }

        let Ok(columns) = self.service.get_table_columns(tables_to_query).await else {
            return Ok(completions.map(CompletionResponse::Array))
        };
        let column_completions = || -> Option<Vec<CompletionItem>> {
            let mut column_items = Vec::new();
            for column in columns.iter() {
                column_items.push(CompletionItem {
                    label: column.name.to_owned(),
                    label_details: Some(CompletionItemLabelDetails {
                        detail: Some(column.table.to_owned()),
                        ..CompletionItemLabelDetails::default()
                    }),
                    kind: Some(CompletionItemKind::PROPERTY),
                    insert_text: Some(column.name.to_owned()),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    ..CompletionItem::default()
                });
            }
            for column in column_items.iter() {
                all_columns.push(column.clone());
            }
            Some(column_items)
        }();
        completions.concat(&column_completions);
        Ok(completions.map(CompletionResponse::Array))
    }
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let all_tables = ALL_TABLES.lock().await;
        let Ok(file_path) = params
            .text_document_position_params
            .text_document
            .uri
            .to_file_path() else {
            return Ok(None);
        };

        let position = &params.text_document_position_params.position;
        let Some(word) = get_word_at_position(position.line, position.character, file_path) else {
            return Ok(None);
        };
        let Some(table) = all_tables.iter().find(|t| t.name == word) else {
            return Ok(None);
        };
        let table_set = HashSet::from_iter(vec![table.clone()]);
        let Ok(columns) = self.service.get_table_columns(table_set).await else {
            return Ok(None);
        };
        let mut contents = Vec::new();
        for column in columns.iter() {
            contents.push(MarkedString::from_markdown(format!(
                "**{}**: {} {}",
                column.name,
                column.data_type,
                if column.is_nullable.eq("YES"){
                    "NULL"
                } else {
                    "NOT NULL"
                }
            )));
        }
        Ok(Some(Hover {
            contents: HoverContents::Array(contents),
            range: None,
        }))
    }
}

pub async fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let repo = FsTenguRepository::new();
    let active_connection_path = repo.active_connection_path();
    let Some(active_connection) = repo.get_active_connection() else {
        eprintln!("No active connection found");
        return;
    };
    let service = TenguService::new(active_connection.engine, repo);

    tokio::spawn(async move {
        async_watch(active_connection_path, reset_cache)
            .await
            .map_err(|e| {
                eprintln!("Error watching active connection path: {}", e);
            })
    });

    let (service, socket) = LspService::new(|client| Backend { client, service });
    Server::new(stdin, stdout, socket).serve(service).await;
}
