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
            "this worker may not use that knowledge base; use the name of a base bound to the worker",
        ),
        KnowledgeError::NotFound => failure(
            codes::NOT_FOUND,
            "no document with that id is in that knowledge base",
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
pub fn knowledge_list(host: &dyn Host, args_json: &str) -> Value {
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
