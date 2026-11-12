//! Tool descriptors. Pure (no WIT imports), so host-testable. `describe.json`
//! carries the same names, schemas and capabilities; a test keeps them equal.

pub const CREATE_TOOL: &str = "knowledge_create";
pub const READ_TOOL: &str = "knowledge_read";
pub const LIST_TOOL: &str = "knowledge_list";
pub const REMOVE_TOOL: &str = "knowledge_remove";

pub struct ToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema_json: &'static str,
    pub output_schema_json: &'static str,
    pub capabilities: Vec<String>,
    pub agentic_worker_metadata: &'static str,
}

pub const CREATE_INPUT: &str = r#"{"type":"object","required":["knowledge_base","name","text"],"properties":{"knowledge_base":{"type":"string","description":"Name of a knowledge base bound to this worker, exactly as shown on the Knowledge page"},"name":{"type":"string","description":"Display name of the new document"},"text":{"type":"string","description":"Document text: plain text, markdown, CSV or JSON"}}}"#;
pub const DOC_INPUT: &str = r#"{"type":"object","required":["knowledge_base","document_id"],"properties":{"knowledge_base":{"type":"string","description":"Name of a knowledge base bound to this worker"},"document_id":{"type":"string","description":"Id from knowledge_list or knowledge_create"}}}"#;
pub const LIST_INPUT: &str = r#"{"type":"object","required":["knowledge_base"],"properties":{"knowledge_base":{"type":"string","description":"Name of a knowledge base bound to this worker"}}}"#;

const CREATE_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"document_id":{"type":"string"},"created":{"type":"boolean","description":"false when an identical document already existed and was reused"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const READ_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"name":{"type":"string"},"text":{"type":"string"},"truncated":{"type":"boolean"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const LIST_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"documents":{"type":"array","items":{"type":"object","properties":{"document_id":{"type":"string"},"name":{"type":"string"},"size_chars":{"type":"integer"}}}},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;
const REMOVE_OUTPUT: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"removed":{"type":"boolean","description":"false when the document was not in that base"},"error":{"type":"object","properties":{"code":{"type":"string"},"message":{"type":"string"}}}}}"#;

const CREATE_META: &str = r#"{"usage_hint":"Save text as a new document in one of your knowledge bases so it can be found later. An identical document is reused, never duplicated. It becomes searchable after the base is next prepared or re-synced.","examples":[{"when":"the user asks you to remember a meeting summary","input":{"knowledge_base":"Team notes","name":"Meeting 2026-11-12","text":"Decisions: ..."}}],"side_effects":"write","cost":"low","confirmation_required":false}"#;
const READ_META: &str = r#"{"usage_hint":"Read one document of a knowledge base by its id (from knowledge_list).","examples":[{"when":"you need the full text of a listed document","input":{"knowledge_base":"Team notes","document_id":"<id from knowledge_list>"}}],"side_effects":"read","cost":"low","confirmation_required":false}"#;
const LIST_META: &str = r#"{"usage_hint":"List the documents of a knowledge base: id, name and size.","examples":[{"when":"you need to find a document before reading or removing it","input":{"knowledge_base":"Team notes"}}],"side_effects":"read","cost":"low","confirmation_required":false}"#;
const REMOVE_META: &str = r#"{"usage_hint":"Remove a document from a knowledge base. The library copy is kept; only its membership in the base is removed.","examples":[{"when":"the user says a saved note is wrong and should be dropped","input":{"knowledge_base":"Team notes","document_id":"<id from knowledge_list>"}}],"side_effects":"write","cost":"low","confirmation_required":false}"#;

fn caps(op: &str) -> Vec<String> {
    vec![format!("cap://knowledge/{op}"), "agentic_worker".into()]
}

#[must_use]
pub fn all_tools() -> Vec<ToolMeta> {
    vec![
        ToolMeta {
            name: CREATE_TOOL,
            description: "Create a text document in one of this worker's knowledge bases.",
            input_schema_json: CREATE_INPUT,
            output_schema_json: CREATE_OUTPUT,
            capabilities: caps("create"),
            agentic_worker_metadata: CREATE_META,
        },
        ToolMeta {
            name: READ_TOOL,
            description: "Read one document of one of this worker's knowledge bases.",
            input_schema_json: DOC_INPUT,
            output_schema_json: READ_OUTPUT,
            capabilities: caps("read"),
            agentic_worker_metadata: READ_META,
        },
        ToolMeta {
            name: LIST_TOOL,
            description: "List the documents of one of this worker's knowledge bases.",
            input_schema_json: LIST_INPUT,
            output_schema_json: LIST_OUTPUT,
            capabilities: caps("list"),
            agentic_worker_metadata: LIST_META,
        },
        ToolMeta {
            name: REMOVE_TOOL,
            description: "Remove a document from one of this worker's knowledge bases (the library copy is kept).",
            input_schema_json: DOC_INPUT,
            output_schema_json: REMOVE_OUTPUT,
            capabilities: caps("remove"),
            agentic_worker_metadata: REMOVE_META,
        },
    ]
}
