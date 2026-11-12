//! Greentic knowledge-writer design extension: `knowledge_create`,
//! `knowledge_read`, `knowledge_list` and `knowledge_remove`.
//!
//! All behaviour lives in pure modules behind `host::Host`; this file is only
//! the WIT export layer plus the adapter that implements `Host` with the real
//! `greentic:extension-host/knowledge` import.
#![allow(clippy::used_underscore_items)]

#[allow(warnings)]
#[rustfmt::skip]
pub mod bindings;

pub mod host;
pub mod tool_meta;
pub mod tools;
#[cfg(test)]
mod tools_tests;

pub use bindings::exports;

use bindings::exports::greentic::extension_base::{lifecycle, manifest};
use bindings::exports::greentic::extension_design::tools as wit_tools;
use bindings::greentic::extension_base::types;
use bindings::greentic::extension_host::knowledge as wit_knowledge;

pub struct Component;

/// The real host, reached through the imported knowledge interface.
struct WasmHost;

fn map_error(e: wit_knowledge::KnowledgeError) -> host::KnowledgeError {
    use wit_knowledge::KnowledgeError as W;
    match e {
        W::NotAllowed => host::KnowledgeError::NotAllowed,
        W::NotFound => host::KnowledgeError::NotFound,
        W::TooLarge => host::KnowledgeError::TooLarge,
        W::InvalidMediaType => host::KnowledgeError::InvalidMediaType,
        W::QuotaExceeded => host::KnowledgeError::QuotaExceeded,
        W::Unavailable => host::KnowledgeError::Unavailable,
    }
}

impl host::Host for WasmHost {
    fn create(
        &self,
        knowledge_base: &str,
        name: &str,
        text: &str,
    ) -> Result<host::Created, host::KnowledgeError> {
        let c = wit_knowledge::create(knowledge_base, name, text).map_err(map_error)?;
        Ok(host::Created {
            document_id: c.document_id,
            created: c.created,
        })
    }

    fn read(
        &self,
        knowledge_base: &str,
        document_id: &str,
    ) -> Result<host::Document, host::KnowledgeError> {
        let d = wit_knowledge::read(knowledge_base, document_id).map_err(map_error)?;
        Ok(host::Document {
            name: d.name,
            text: d.text,
            truncated: d.truncated,
        })
    }

    fn list(&self, knowledge_base: &str) -> Result<Vec<host::DocumentInfo>, host::KnowledgeError> {
        let rows = wit_knowledge::list(knowledge_base).map_err(map_error)?;
        Ok(rows
            .into_iter()
            .map(|r| host::DocumentInfo {
                document_id: r.document_id,
                name: r.name,
                size_chars: r.size_chars,
            })
            .collect())
    }

    fn remove(
        &self,
        knowledge_base: &str,
        document_id: &str,
    ) -> Result<bool, host::KnowledgeError> {
        wit_knowledge::remove(knowledge_base, document_id).map_err(map_error)
    }
}

impl manifest::Guest for Component {
    fn get_identity() -> types::ExtensionIdentity {
        types::ExtensionIdentity {
            id: "greentic.knowledge-writer".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            kind: types::Kind::Design,
        }
    }

    fn get_offered() -> Vec<types::CapabilityRef> {
        vec![types::CapabilityRef {
            id: "greentic:knowledge/write".into(),
            version: "1.0.0".into(),
        }]
    }

    fn get_required() -> Vec<types::CapabilityRef> {
        vec![]
    }
}

impl lifecycle::Guest for Component {
    fn init(_config_json: String) -> Result<(), types::ExtensionError> {
        Ok(())
    }

    fn shutdown() {}
}

impl wit_tools::Guest for Component {
    fn list_tools() -> Vec<wit_tools::ToolDefinition> {
        tool_meta::all_tools()
            .into_iter()
            .map(|m| wit_tools::ToolDefinition {
                name: m.name.into(),
                description: m.description.into(),
                input_schema_json: m.input_schema_json.into(),
                output_schema_json: Some(m.output_schema_json.into()),
                capabilities: Some(m.capabilities),
                agentic_worker_metadata: Some(m.agentic_worker_metadata.into()),
            })
            .collect()
    }

    /// Tool failures are values in the returned JSON; `Err` is reserved for
    /// an unknown tool name or a local encoding fault.
    fn invoke_tool(name: String, args_json: String) -> Result<String, types::ExtensionError> {
        let value = match name.as_str() {
            tool_meta::CREATE_TOOL => tools::knowledge_create(&WasmHost, &args_json),
            tool_meta::READ_TOOL => tools::knowledge_read(&WasmHost, &args_json),
            tool_meta::LIST_TOOL => tools::knowledge_list(&WasmHost, &args_json),
            tool_meta::REMOVE_TOOL => tools::knowledge_remove(&WasmHost, &args_json),
            other => {
                return Err(types::ExtensionError::InvalidInput(format!(
                    "unknown tool: {other}"
                )));
            }
        };
        serde_json::to_string(&value)
            .map_err(|e| types::ExtensionError::Internal(format!("encode output: {e}")))
    }
}

#[cfg(target_family = "wasm")]
bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod adapter_tests {
    use super::{map_error, wit_knowledge::KnowledgeError as W};
    use crate::host::KnowledgeError as H;

    #[test]
    fn every_wit_error_maps_to_its_own_variant() {
        let pairs = [
            (W::NotAllowed, H::NotAllowed),
            (W::NotFound, H::NotFound),
            (W::TooLarge, H::TooLarge),
            (W::InvalidMediaType, H::InvalidMediaType),
            (W::QuotaExceeded, H::QuotaExceeded),
            (W::Unavailable, H::Unavailable),
        ];
        for (w, h) in pairs {
            assert_eq!(map_error(w), h);
        }
    }
}
