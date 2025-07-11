#![allow(dead_code)]
#![allow(unused_imports)]

use tower_lsp::lsp_types::*;
use dashmap::DashMap;
use ropey::Rope;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub rope: Rope,
    pub version: i32,
    pub language_id: String,
}

pub struct DocumentManager {
    documents: DashMap<Url, Document>,
}

impl DocumentManager {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
        }
    }

    pub async fn open_document(&self, uri: Url, text: String, version: i32) {
        let document = Document {
            uri: uri.clone(),
            rope: Rope::from_str(&text),
            version,
            language_id: "nagari".to_string(),
        };

        self.documents.insert(uri, document);
    }

    pub async fn update_document(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) {
        if let Some(mut document) = self.documents.get_mut(uri) {
            for change in changes {
                match change.range {
                    Some(range) => {
                        // Incremental change
                        let start_char = self.position_to_char(&document.rope, range.start);
                        let end_char = self.position_to_char(&document.rope, range.end);

                        document.rope.remove(start_char..end_char);
                        document.rope.insert(start_char, &change.text);
                    }
                    None => {
                        // Full document change
                        document.rope = Rope::from_str(&change.text);
                    }
                }
            }
            document.version = version;
        }
    }

    pub async fn close_document(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub async fn get_document(&self, uri: &Url) -> Option<Document> {
        self.documents.get(uri).map(|doc| doc.clone())
    }

    pub async fn get_document_text(&self, uri: &Url) -> Option<String> {
        self.documents.get(uri).map(|doc| doc.rope.to_string())
    }

    pub async fn get_text_at_position(&self, uri: &Url, position: Position) -> Option<String> {
        let document = self.documents.get(uri)?;
        let char_pos = self.position_to_char(&document.rope, position);

        // Get word at position
        let text = document.rope.to_string();
        let chars: Vec<char> = text.chars().collect();

        let mut start = char_pos;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        let mut end = char_pos;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        Some(chars[start..end].iter().collect())
    }

    pub async fn get_line_text(&self, uri: &Url, line: u32) -> Option<String> {
        let document = self.documents.get(uri)?;
        let line_slice = document.rope.line(line as usize);
        Some(line_slice.to_string())
    }

    pub fn position_to_char(&self, rope: &Rope, position: Position) -> usize {
        let line_start = rope.line_to_char(position.line as usize);
        line_start + position.character as usize
    }

    pub fn char_to_position(&self, rope: &Rope, char_pos: usize) -> Position {
        let line = rope.char_to_line(char_pos);
        let line_start = rope.line_to_char(line);
        let character = char_pos - line_start;

        Position::new(line as u32, character as u32)
    }

    pub async fn list_documents(&self) -> Vec<Url> {
        self.documents.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get a shared reference to a document using Arc
    pub async fn get_document_arc(&self, uri: &Url) -> Option<Arc<Document>> {
        self.documents.get(uri).map(|entry| Arc::new(entry.clone()))
    }

    /// Store a document with Arc sharing
    pub async fn store_document_arc(&self, document: Arc<Document>) {
        self.documents.insert(document.uri.clone(), (*document).clone());
    }

    /// Get multiple documents as Arc references
    pub async fn get_documents_arc(&self, uris: &[Url]) -> Vec<Arc<Document>> {
        uris.iter()
            .filter_map(|uri| self.documents.get(uri).map(|entry| Arc::new(entry.clone())))
            .collect()
    }

    /// Get a thread-safe mutable reference to a document using Mutex
    pub async fn get_document_mutex(&self, uri: &Url) -> Option<Arc<std::sync::Mutex<Document>>> {
        self.documents.get(uri).map(|entry| Arc::new(std::sync::Mutex::new(entry.clone())))
    }
}
