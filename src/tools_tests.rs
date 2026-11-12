//! Tool behaviour against a recording mock host, and `describe.json` kept in
//! step with the descriptors the component reports.

use std::cell::RefCell;

use serde_json::{Value, json};

use crate::host::{BaseInfo, Created, Document, DocumentInfo, Host, KnowledgeError, SearchHit};
use crate::tool_meta;
use crate::tools::{
    knowledge_create, knowledge_list_bases, knowledge_list_documents, knowledge_read,
    knowledge_remove, knowledge_search, knowledge_update,
};

#[derive(Default)]
struct MockHost {
    calls: RefCell<Vec<String>>,
    fail: Option<KnowledgeError>,
}

impl MockHost {
    fn failing(e: KnowledgeError) -> Self {
        Self {
            fail: Some(e),
            ..Self::default()
        }
    }

    fn record(&self, call: String) -> Result<(), KnowledgeError> {
        self.calls.borrow_mut().push(call);
        self.fail.map_or(Ok(()), Err)
    }
}

impl Host for MockHost {
    fn create(&self, kb: &str, name: &str, text: &str) -> Result<Created, KnowledgeError> {
        self.record(format!("create {kb}|{name}|{}", text.len()))?;
        Ok(Created {
            document_id: "doc-1".into(),
            created: true,
        })
    }

    fn read(&self, kb: &str, id: &str) -> Result<Document, KnowledgeError> {
        self.record(format!("read {kb}|{id}"))?;
        Ok(Document {
            name: "Notes".into(),
            text: "hello".into(),
            truncated: false,
        })
    }

    fn list(&self, kb: &str) -> Result<Vec<DocumentInfo>, KnowledgeError> {
        self.record(format!("list {kb}"))?;
        Ok(vec![DocumentInfo {
            document_id: "doc-1".into(),
            name: "Notes".into(),
            size_chars: 5,
        }])
    }

    fn remove(&self, kb: &str, id: &str) -> Result<bool, KnowledgeError> {
        self.record(format!("remove {kb}|{id}"))?;
        Ok(true)
    }

    fn list_bases(&self) -> Result<Vec<BaseInfo>, KnowledgeError> {
        self.record("list_bases".into())?;
        Ok(vec![BaseInfo {
            name: "KB".into(),
            description: "Team notes".into(),
            document_count: 2,
        }])
    }

    fn search(&self, kb: &str, query: &str, limit: u32) -> Result<Vec<SearchHit>, KnowledgeError> {
        self.record(format!("search {kb}|{query}|{limit}"))?;
        Ok(vec![SearchHit {
            document_id: "doc-1".into(),
            name: "Notes".into(),
            passage: "hello".into(),
            score: 0.5,
        }])
    }

    fn update(&self, kb: &str, id: &str, text: &str) -> Result<String, KnowledgeError> {
        self.record(format!("update {kb}|{id}|{}", text.len()))?;
        Ok("doc-2".into())
    }
}

#[test]
fn create_trims_the_base_name_and_returns_the_document_id() {
    let h = MockHost::default();
    let out = knowledge_create(
        &h,
        r#"{"knowledge_base":"  Team notes ","name":"Meeting","text":"hello"}"#,
    );
    assert_eq!(
        out,
        json!({"ok": true, "document_id": "doc-1", "created": true})
    );
    assert_eq!(*h.calls.borrow(), vec!["create Team notes|Meeting|5"]);
}

#[test]
fn read_list_and_remove_pass_their_arguments_through() {
    let h = MockHost::default();
    let read = knowledge_read(&h, r#"{"knowledge_base":"KB","document_id":" doc-1 "}"#);
    assert_eq!(
        read,
        json!({"ok": true, "name": "Notes", "text": "hello", "truncated": false})
    );
    let list = knowledge_list_documents(&h, r#"{"knowledge_base":"KB"}"#);
    assert_eq!(
        list,
        json!({"ok": true, "documents": [{"document_id": "doc-1", "name": "Notes", "size_chars": 5}]})
    );
    let removed = knowledge_remove(&h, r#"{"knowledge_base":"KB","document_id":"doc-1"}"#);
    assert_eq!(removed, json!({"ok": true, "removed": true}));
    assert_eq!(
        *h.calls.borrow(),
        vec!["read KB|doc-1", "list KB", "remove KB|doc-1"]
    );
}

#[test]
fn list_bases_search_and_update_pass_their_arguments_through() {
    let h = MockHost::default();
    assert_eq!(
        knowledge_list_bases(&h, "{}"),
        json!({"ok": true, "bases": [{"name": "KB", "description": "Team notes", "document_count": 2}]})
    );
    assert_eq!(
        knowledge_search(
            &h,
            r#"{"knowledge_base":"KB","query":" refunds ","limit":20}"#
        ),
        json!({"ok": true, "hits": [{"document_id": "doc-1", "name": "Notes", "passage": "hello", "score": 0.5}]})
    );
    assert_eq!(
        knowledge_search(&h, r#"{"knowledge_base":"KB","query":"q"}"#)["ok"],
        true
    );
    assert_eq!(
        knowledge_update(
            &h,
            r#"{"knowledge_base":"KB","document_id":"doc-1","text":"new"}"#
        ),
        json!({"ok": true, "document_id": "doc-2"})
    );
    assert_eq!(
        *h.calls.borrow(),
        vec![
            "list_bases",
            "search KB|refunds|20",
            "search KB|q|5",
            "update KB|doc-1|3"
        ]
    );
}

#[test]
fn a_search_limit_outside_one_to_twenty_is_refused_not_cut() {
    let h = MockHost::default();
    for limit in [0, 21] {
        let out = knowledge_search(
            &h,
            &format!(r#"{{"knowledge_base":"KB","query":"q","limit":{limit}}}"#),
        );
        assert_eq!(out["error"]["code"], "invalid_input", "{out}");
    }
    assert!(h.calls.borrow().is_empty());
}

#[test]
fn blank_or_missing_arguments_never_reach_the_host() {
    let h = MockHost::default();
    let cases = [
        knowledge_create(&h, r#"{"knowledge_base":" ","name":"n","text":"t"}"#),
        knowledge_create(&h, r#"{"knowledge_base":"KB","name":"","text":"t"}"#),
        knowledge_create(&h, r#"{"knowledge_base":"KB","name":"n","text":"  "}"#),
        knowledge_create(&h, r#"{"knowledge_base":"KB","name":"n"}"#),
        knowledge_read(&h, r#"{"knowledge_base":"KB","document_id":""}"#),
        knowledge_list_documents(&h, "not json"),
        knowledge_remove(&h, r#"{"knowledge_base":"KB"}"#),
        knowledge_search(&h, r#"{"knowledge_base":"KB","query":"  "}"#),
        knowledge_update(&h, r#"{"knowledge_base":"KB","document_id":"d","text":""}"#),
        knowledge_update(&h, r#"{"knowledge_base":"KB","text":"t"}"#),
    ];
    for out in cases {
        assert_eq!(out["ok"], false, "{out}");
        assert_eq!(out["error"]["code"], "invalid_input", "{out}");
    }
    assert!(h.calls.borrow().is_empty());
}

#[test]
fn every_host_refusal_becomes_its_own_code_and_never_echoes_the_text() {
    let cases = [
        (KnowledgeError::NotAllowed, "not_allowed"),
        (KnowledgeError::NotFound, "not_found"),
        (KnowledgeError::TooLarge, "too_large"),
        (KnowledgeError::InvalidMediaType, "invalid_media_type"),
        (KnowledgeError::QuotaExceeded, "quota_exceeded"),
        (KnowledgeError::Unavailable, "unavailable"),
    ];
    for (err, code) in cases {
        let h = MockHost::failing(err);
        let outs = [
            knowledge_create(
                &h,
                r#"{"knowledge_base":"KB","name":"n","text":"SECRET-BODY"}"#,
            ),
            knowledge_update(
                &h,
                r#"{"knowledge_base":"KB","document_id":"d","text":"SECRET-BODY"}"#,
            ),
            knowledge_search(&h, r#"{"knowledge_base":"KB","query":"SECRET-BODY"}"#),
            knowledge_list_bases(&h, "{}"),
        ];
        for out in outs {
            assert_eq!(out["ok"], false);
            assert_eq!(out["error"]["code"], code);
            assert!(!out.to_string().contains("SECRET-BODY"), "{out}");
        }
    }
}

fn describe() -> Value {
    serde_json::from_str(include_str!("../describe.json")).expect("describe.json parses")
}

#[test]
fn describe_declares_exactly_the_tools_the_component_reports() {
    let d = describe();
    let tools = d["contributions"]["tools"].as_array().expect("tools array");
    let metas = tool_meta::all_tools();
    assert_eq!(tools.len(), metas.len());
    for (t, m) in tools.iter().zip(&metas) {
        assert_eq!(t["name"], m.name);
        assert_eq!(t["description"], m.description);
        assert_eq!(t["input_schema"], m.input_schema_json, "{}", m.name);
        assert_eq!(t["output_schema"], m.output_schema_json, "{}", m.name);
        assert_eq!(t["capabilities"], json!(m.capabilities), "{}", m.name);
        assert_eq!(t["secret_requirements"], json!([]), "{}", m.name);
        assert_eq!(t["export"], "greentic:extension-design/tools.invoke-tool");
        // Schemas are JSON documents, not just strings.
        let _: Value = serde_json::from_str(m.input_schema_json).expect("input schema");
        let _: Value = serde_json::from_str(m.output_schema_json).expect("output schema");
        let meta: Value =
            serde_json::from_str(m.agentic_worker_metadata).expect("agentic metadata");
        assert!(meta["side_effects"].is_string());
        // Every capability's op has a matching permission token.
        let op = m.capabilities[0].trim_start_matches("cap://knowledge/");
        let token = format!("knowledge://{op}");
        assert!(
            d["runtime"]["permissions"]["network"]
                .as_array()
                .unwrap()
                .iter()
                .any(|t| t == &json!(token)),
            "{token}"
        );
    }
}

#[test]
fn describe_declares_one_knowledge_permission_per_tool_and_nothing_else() {
    let d = describe();
    let perms = &d["runtime"]["permissions"];
    assert_eq!(
        perms["network"],
        json!([
            "knowledge://create",
            "knowledge://read",
            "knowledge://list",
            "knowledge://remove",
            "knowledge://list-bases",
            "knowledge://search",
            "knowledge://update"
        ])
    );
    assert_eq!(perms["secrets"], json!([]));
    assert_eq!(d["metadata"]["id"], "greentic.knowledge");
    assert_eq!(d["metadata"]["version"], env!("CARGO_PKG_VERSION"));
}
