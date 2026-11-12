//! The only thing the tools need from the host: the knowledge port. Implemented
//! by the wasm adapter in `lib.rs` and by a mock in tests.
//!
//! The host decides the tenant, the team and which bases a call may touch. A
//! guest names a base only by the name an operator sees.

/// A document the host created (or found already present).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Created {
    pub document_id: String,
    /// `false` when an identical document already existed and was reused.
    pub created: bool,
}

/// One document read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub name: String,
    pub text: String,
    /// `true` when the host returned only a prefix of the text.
    pub truncated: bool,
}

/// One row of a base's listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentInfo {
    pub document_id: String,
    pub name: String,
    pub size_chars: u64,
}

/// One base this call may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseInfo {
    pub name: String,
    pub description: String,
    pub document_count: u64,
}

/// One search result: a host-truncated excerpt of a matching document.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub document_id: String,
    pub name: String,
    pub passage: String,
    pub score: f32,
}

/// The host's closed set of refusals. None carries document text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeError {
    /// Undeclared operation, no tenant, or a base this call may not use (an
    /// unknown base reads the same).
    NotAllowed,
    NotFound,
    TooLarge,
    InvalidMediaType,
    QuotaExceeded,
    /// No knowledge store on this host, or it failed.
    Unavailable,
}

pub trait Host {
    fn create(
        &self,
        knowledge_base: &str,
        name: &str,
        text: &str,
    ) -> Result<Created, KnowledgeError>;
    fn read(&self, knowledge_base: &str, document_id: &str) -> Result<Document, KnowledgeError>;
    fn list(&self, knowledge_base: &str) -> Result<Vec<DocumentInfo>, KnowledgeError>;
    fn remove(&self, knowledge_base: &str, document_id: &str) -> Result<bool, KnowledgeError>;
    fn list_bases(&self) -> Result<Vec<BaseInfo>, KnowledgeError>;
    fn search(
        &self,
        knowledge_base: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchHit>, KnowledgeError>;
    /// Replace a document's text. Returns the id afterwards (ids are content
    /// hashes, so it may differ from the one passed in).
    fn update(
        &self,
        knowledge_base: &str,
        document_id: &str,
        text: &str,
    ) -> Result<String, KnowledgeError>;
}
