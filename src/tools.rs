//! Tool logic over the `Host` trait. Pure apart from the host calls.
//!
//! Every failure is a value: `{"ok":false,"error":{"code","message"}}`. The
//! messages are fixed sentences; the host never returns internal detail, and
//! nothing here echoes document text into an error.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::host::{Host, KnowledgeError};

/// Stable error codes a tool result may carry in `error.code`.
pub mod codes {
    pub const INVALID_INPUT: &str = "invalid_input";
    pub const NOT_ALLOWED: &str = "not_allowed";
    pub const NOT_FOUND: &str = "not_found";
    pub const TOO_LARGE: &str = "too_large";
    pub const INVALID_MEDIA_TYPE: &str = "invalid_media_type";
    pub const QUOTA_EXCEEDED: &str = "quota_exceeded";
    pub const UNAVAILABLE: &str = "unavailable";
}

fn failure(code: &str, message: &str) -> Value {
    json!({ "ok": false, "error": { "code": code, "message": message } })
}

fn host_failure(e: KnowledgeError) -> Value {
    match e {
        KnowledgeError::NotAllowed => failure(
            codes::NOT_ALLOWED,
            "not allowed: use the exact name of one of this worker's knowledge bases (knowledge_list_bases lists them)",
        ),
        KnowledgeError::NotFound => failure(
            codes::NOT_FOUND,
            "no document with that id is in that knowledge base; call knowledge_list_documents for current ids",
        ),
        KnowledgeError::TooLarge => failure(
            codes::TOO_LARGE,
            "the document text or name is over the host's size limit",
        ),
        KnowledgeError::InvalidMediaType => failure(
            codes::INVALID_MEDIA_TYPE,
            "the host accepts only plain text, markdown, CSV or JSON",
        ),
        KnowledgeError::QuotaExceeded => failure(
            codes::QUOTA_EXCEEDED,
            "the workspace knowledge quota is exhausted",
        ),
        KnowledgeError::Unavailable => failure(
            codes::UNAVAILABLE,
            "knowledge storage is not available here; it works in Test chat on a designer that provides it",
        ),
    }
}

/// Parse `args_json` into `T`, mapping a decode fault to a tool failure.
fn parse<T: for<'de> Deserialize<'de>>(args_json: &str) -> Result<T, Value> {
    serde_json::from_str(args_json).map_err(|e| {
        failure(
            codes::INVALID_INPUT,
            &format!("the arguments do not match the tool's input schema: {e}"),
        )
    })
}

/// A required identifier: trimmed, never blank.
fn required<'a>(field: &str, value: &'a str) -> Result<&'a str, Value> {
    let v = value.trim();
    if v.is_empty() {
        return Err(failure(codes::INVALID_INPUT, &format!("{field} is blank")));
    }
    Ok(v)
}

#[derive(Deserialize)]
struct CreateArgs {
    knowledge_base: String,
    name: String,
    text: String,
}

#[derive(Deserialize)]
struct DocArgs {
    knowledge_base: String,
    document_id: String,
}

#[derive(Deserialize)]
struct ListArgs {
    knowledge_base: String,
}

#[derive(Deserialize)]
struct SearchArgs {
    knowledge_base: String,
    query: String,
    limit: Option<u32>,
}

#[derive(Deserialize)]
struct UpdateArgs {
    knowledge_base: String,
    document_id: String,
    text: String,
}

/// Search results returned when `limit` is omitted.
pub const DEFAULT_SEARCH_LIMIT: u32 = 5;
/// The host's own cap; a larger `limit` is refused rather than silently cut.
pub const MAX_SEARCH_LIMIT: u32 = 20;

fn run(f: impl FnOnce() -> Result<Value, Value>) -> Value {
    f().unwrap_or_else(|failure| failure)
}

#[must_use]
pub fn knowledge_create(host: &dyn Host, args_json: &str) -> Value {
    run(|| {
        let a: CreateArgs = parse(args_json)?;
        let base = required("knowledge_base", &a.knowledge_base)?;
        required("name", &a.name)?;
        if a.text.trim().is_empty() {
            return Err(failure(codes::INVALID_INPUT, "text is blank"));
        }
        let c = host.create(base, &a.name, &a.text).map_err(host_failure)?;
        Ok(json!({ "ok": true, "document_id": c.document_id, "created": c.created }))
    })
}

#[must_use]
pub fn knowledge_read(host: &dyn Host, args_json: &str) -> Value {
    run(|| {
        let a: DocArgs = parse(args_json)?;
        let base = required("knowledge_base", &a.knowledge_base)?;
        let id = required("document_id", &a.document_id)?;
        let d = host.read(base, id).map_err(host_failure)?;
        Ok(json!({ "ok": true, "name": d.name, "text": d.text, "truncated": d.truncated }))
    })
}

#[must_use]
pub fn knowledge_list_documents(host: &dyn Host, args_json: &str) -> Value {
    run(|| {
        let a: ListArgs = parse(args_json)?;
        let base = required("knowledge_base", &a.knowledge_base)?;
        let docs = host.list(base).map_err(host_failure)?;
        let documents: Vec<Value> = docs
            .into_iter()
            .map(|d| {
                json!({ "document_id": d.document_id, "name": d.name, "size_chars": d.size_chars })
            })
            .collect();
        Ok(json!({ "ok": true, "documents": documents }))
    })
}

#[must_use]
pub fn knowledge_remove(host: &dyn Host, args_json: &str) -> Value {
    run(|| {
        let a: DocArgs = parse(args_json)?;
        let base = required("knowledge_base", &a.knowledge_base)?;
        let id = required("document_id", &a.document_id)?;
        let removed = host.remove(base, id).map_err(host_failure)?;
        Ok(json!({ "ok": true, "removed": removed }))
    })
}

#[must_use]
pub fn knowledge_list_bases(host: &dyn Host, _args_json: &str) -> Value {
    run(|| {
        let bases = host.list_bases().map_err(host_failure)?;
        let bases: Vec<Value> = bases
            .into_iter()
            .map(|b| {
                json!({ "name": b.name, "description": b.description, "document_count": b.document_count })
            })
            .collect();
        Ok(json!({ "ok": true, "bases": bases }))
    })
}

#[must_use]
pub fn knowledge_search(host: &dyn Host, args_json: &str) -> Value {
    run(|| {
        let a: SearchArgs = parse(args_json)?;
        let base = required("knowledge_base", &a.knowledge_base)?;
        let query = required("query", &a.query)?;
        let limit = a.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
        if !(1..=MAX_SEARCH_LIMIT).contains(&limit) {
            return Err(failure(
                codes::INVALID_INPUT,
                &format!("limit must be between 1 and {MAX_SEARCH_LIMIT}"),
            ));
        }
        let hits = host.search(base, query, limit).map_err(host_failure)?;
        let hits: Vec<Value> = hits
            .into_iter()
            .map(|h| {
                json!({ "document_id": h.document_id, "name": h.name, "passage": h.passage, "score": h.score })
            })
            .collect();
        Ok(json!({ "ok": true, "hits": hits }))
    })
}

#[must_use]
pub fn knowledge_update(host: &dyn Host, args_json: &str) -> Value {
    run(|| {
        let a: UpdateArgs = parse(args_json)?;
        let base = required("knowledge_base", &a.knowledge_base)?;
        let id = required("document_id", &a.document_id)?;
        if a.text.trim().is_empty() {
            return Err(failure(codes::INVALID_INPUT, "text is blank"));
        }
        let new_id = host.update(base, id, &a.text).map_err(host_failure)?;
        Ok(json!({ "ok": true, "document_id": new_id }))
    })
}
