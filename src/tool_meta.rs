//! Tool descriptors. Pure (no WIT imports), so host-testable. `describe.json`
//! carries the same names, schemas and capabilities; a test keeps them equal.
//!
//! Every tool works only on the knowledge bases bound to the calling worker;
//! the host decides that set. The descriptions say so, and say which tool to
//! reach for when, because a model picks tools from these strings alone.

pub const LIST_BASES_TOOL: &str = "knowledge_list_bases";
pub const SEARCH_TOOL: &str = "knowledge_search";
pub const LIST_TOOL: &str = "knowledge_list_documents";
pub const READ_TOOL: &str = "knowledge_read";
pub const CREATE_TOOL: &str = "knowledge_create";
pub const UPDATE_TOOL: &str = "knowledge_update";
pub const REMOVE_TOOL: &str = "knowledge_remove";

pub struct ToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema_json: &'static str,
    pub output_schema_json: &'static str,
    pub capabilities: Vec<String>,
    pub agentic_worker_metadata: &'static str,
}

pub const NO_INPUT: &str = r#"{"type":"object","properties":{}}"#;
pub const SEARCH_INPUT: &str = r#"{"type":"object","required":["knowledge_base","query"],"properties":{"knowledge_base":{"type":"string","description":"Exact name of one of this worker's knowledge bases (from knowledge_list_bases)"},"query":{"type":"string","description":"What to look for, in natural language or keywords"},"limit":{"type":"integer","minimum":1,"maximum":20,"description":"How many passages to return (default 5, at most 20)"}}}"#;
pub const LIST_INPUT: &str = r#"{"type":"object","required":["knowledge_base"],"properties":{"knowledge_base":{"type":"string","description":"Exact name of one of this worker's knowledge bases (from knowledge_list_bases)"}}}"#;
pub const DOC_INPUT: &str = r#"{"type":"object","required":["knowledge_base","document_id"],"properties":{"knowledge_base":{"type":"string","description":"Exact name of one of this worker's knowledge bases"},"document_id":{"type":"string","description":"Id from knowledge_list_documents, knowledge_search, knowledge_create or knowledge_update"}}}"#;
pub const CREATE_INPUT: &str = r#"{"type":"object","required":["knowledge_base","name","text"],"properties":{"knowledge_base":{"type":"string","description":"Exact name of one of this worker's knowledge bases"},"name":{"type":"string","description":"Display name of the new document"},"text":{"type":"string","description":"Document text: plain text, markdown, CSV or JSON"}}}"#;
pub const UPDATE_INPUT: &str = r#"{"type":"object","required":["knowledge_base","document_id","text"],"properties":{"knowledge_base":{"type":"string","description":"Exact name of one of this worker's knowledge bases"},"document_id":{"type":"string","description":"Id of the document to replace"},"text":{"type":"string","description":"The complete new text (it replaces the old text, it is not appended)"}}}"#;

const LIST_BASES_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"bases":{"type":"array","items":{"type":"object","properties":{"name":{"type":"string"},"description":{"type":"string"},"document_count":{"type":"integer"}}}},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const SEARCH_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"hits":{"type":"array","items":{"type":"object","properties":{"document_id":{"type":"string"},"name":{"type":"string"},"passage":{"type":"string","description":"An excerpt of the matching text, shortened by the host"},"score":{"type":"number"}}}},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const LIST_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"documents":{"type":"array","items":{"type":"object","properties":{"document_id":{"type":"string"},"name":{"type":"string"},"size_chars":{"type":"integer"}}}},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const READ_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"name":{"type":"string"},"text":{"type":"string"},"truncated":{"type":"boolean","description":"true when only the start of the text was returned"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const CREATE_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"document_id":{"type":"string"},"created":{"type":"boolean","description":"false when an identical document already existed and was reused"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const UPDATE_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"document_id":{"type":"string","description":"The document's id after the update; use this one from now on, it may differ from the old id"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const REMOVE_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"removed":{"type":"boolean","description":"false when the document was not in that base"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;

const LIST_BASES_META: &str = r#"{"usage_hint":"Call this first when you do not know the exact name of a knowledge base. It lists only the bases bound to you; every other knowledge tool takes one of these names.","examples":[{"when":"before searching or saving, to learn which bases you have","input":{}}],"side_effects":"read","cost":"low","confirmation_required":false}"#;
const SEARCH_META: &str = r#"{"usage_hint":"Find passages relevant to a question in one of your knowledge bases. Prefer this over knowledge_read when you do not yet know which document holds the answer; then read a hit's document in full if the passage is not enough.","examples":[{"when":"the user asks something your documents may answer","input":{"knowledge_base":"Product docs","query":"refund policy for annual plans","limit":5}}],"side_effects":"read","cost":"low","confirmation_required":false}"#;
const LIST_META: &str = r#"{"usage_hint":"List the documents of one of your knowledge bases (id, name, size). Use it to find a document by name, or to get an id before reading, updating or removing.","examples":[{"when":"you need a document id","input":{"knowledge_base":"Team notes"}}],"side_effects":"read","cost":"low","confirmation_required":false}"#;
const READ_META: &str = r#"{"usage_hint":"Read the full text of one document by id. Long documents may come back truncated (truncated: true).","examples":[{"when":"a search hit or a listed document needs reading in full","input":{"knowledge_base":"Team notes","document_id":"<id from knowledge_list_documents>"}}],"side_effects":"read","cost":"low","confirmation_required":false}"#;
const CREATE_META: &str = r#"{"usage_hint":"Save new text as a document in one of your knowledge bases. An identical document is reused, never duplicated. To change an existing document use knowledge_update instead. New text becomes searchable after the base is next prepared or re-synced, not immediately.","examples":[{"when":"the user asks you to remember a meeting summary","input":{"knowledge_base":"Team notes","name":"Meeting 2026-11-12","text":"Decisions: ..."}}],"side_effects":"write","cost":"low","confirmation_required":false}"#;
const UPDATE_META: &str = r#"{"usage_hint":"Replace the whole text of an existing document. Read it first if you only want to change part of it. The document id may change; use the returned id afterwards.","examples":[{"when":"a saved note is out of date","input":{"knowledge_base":"Team notes","document_id":"<id>","text":"<the full corrected text>"}}],"side_effects":"write","cost":"low","confirmation_required":false}"#;
const REMOVE_META: &str = r#"{"usage_hint":"Remove a document from one of your knowledge bases. The library copy is kept; only its membership in the base is removed.","examples":[{"when":"the user says a saved note is wrong and should be dropped","input":{"knowledge_base":"Team notes","document_id":"<id from knowledge_list_documents>"}}],"side_effects":"write","cost":"low","confirmation_required":false}"#;

fn caps(op: &str) -> Vec<String> {
    vec![format!("cap://knowledge/{op}"), "agentic_worker".into()]
}

#[must_use]
pub fn all_tools() -> Vec<ToolMeta> {
    vec![
        ToolMeta {
            name: LIST_BASES_TOOL,
            description: "List the knowledge bases you may use (name, description, number of documents). Only the bases bound to you appear; use one of these names in every other knowledge tool.",
            input_schema_json: NO_INPUT,
            output_schema_json: LIST_BASES_OUTPUT,
            capabilities: caps("list-bases"),
            agentic_worker_metadata: LIST_BASES_META,
        },
        ToolMeta {
            name: SEARCH_TOOL,
            description: "Search one of your knowledge bases and return the most relevant passages (at most 20). Use this to answer a question from your documents when you do not know which document holds the answer.",
            input_schema_json: SEARCH_INPUT,
            output_schema_json: SEARCH_OUTPUT,
            capabilities: caps("search"),
            agentic_worker_metadata: SEARCH_META,
        },
        ToolMeta {
            name: LIST_TOOL,
            description: "List the documents of one of your knowledge bases (id, name, size), to find a document or get its id.",
            input_schema_json: LIST_INPUT,
            output_schema_json: LIST_OUTPUT,
            capabilities: caps("list"),
            agentic_worker_metadata: LIST_META,
        },
        ToolMeta {
            name: READ_TOOL,
            description: "Read the full text of one document in one of your knowledge bases, by its id.",
            input_schema_json: DOC_INPUT,
            output_schema_json: READ_OUTPUT,
            capabilities: caps("read"),
            agentic_worker_metadata: READ_META,
        },
        ToolMeta {
            name: CREATE_TOOL,
            description: "Save new text (plain text, markdown, CSV or JSON) as a document in one of your knowledge bases. To change an existing document, use knowledge_update.",
            input_schema_json: CREATE_INPUT,
            output_schema_json: CREATE_OUTPUT,
            capabilities: caps("create"),
            agentic_worker_metadata: CREATE_META,
        },
        ToolMeta {
            name: UPDATE_TOOL,
            description: "Replace the whole text of a document in one of your knowledge bases. Returns the document's id afterwards, which may differ from the old one.",
            input_schema_json: UPDATE_INPUT,
            output_schema_json: UPDATE_OUTPUT,
            capabilities: caps("update"),
            agentic_worker_metadata: UPDATE_META,
        },
        ToolMeta {
            name: REMOVE_TOOL,
            description: "Remove a document from one of your knowledge bases. The library copy is kept; only its membership in the base is removed.",
            input_schema_json: DOC_INPUT,
            output_schema_json: REMOVE_OUTPUT,
            capabilities: caps("remove"),
            agentic_worker_metadata: REMOVE_META,
        },
    ]
}
