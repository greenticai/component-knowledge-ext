# greentic.knowledge

A Greentic design extension that gives an agentic worker tools over its own
knowledge bases: find them, search them, read documents, and keep them up to
date. It exposes seven tools:

| tool | capabilities | what it does |
|---|---|---|
| `knowledge_list_bases` | `cap://knowledge/list-bases`, `agentic_worker` | list the bases this worker may use (name, description, document count) |
| `knowledge_search` | `cap://knowledge/search`, `agentic_worker` | return the most relevant passages of a base (`limit` 1-20, default 5) |
| `knowledge_list_documents` | `cap://knowledge/list`, `agentic_worker` | list a base's documents (id, name, size) |
| `knowledge_read` | `cap://knowledge/read`, `agentic_worker` | read one document in full (may be truncated by the host) |
| `knowledge_create` | `cap://knowledge/create`, `agentic_worker` | save new text as a document |
| `knowledge_update` | `cap://knowledge/update`, `agentic_worker` | replace a document's whole text; returns the id afterwards |
| `knowledge_remove` | `cap://knowledge/remove`, `agentic_worker` | remove a document from a base (the library copy is kept) |

Each tool is a thin wrapper over the host import
`greentic:extension-host/knowledge@0.1.0`. The extension holds no secrets and
makes no network calls.

## Rules the host enforces, not this extension

- The tenant, the team and the set of bases a call may touch come from the
  host: only the bases bound to the calling worker. A tool names a base only
  by the name an operator sees; any other base answers `not_allowed`, exactly
  like a base that does not exist.
- Size caps, accepted content kinds (plain text, markdown, CSV, JSON),
  quotas, how search matches (semantic or lexical) and how long a passage is
  are the host's. The tool only reports `too_large`, `invalid_media_type` or
  `quota_exceeded`.
- Document ids are content hashes: an identical document is reused
  (`created: false`), and `knowledge_update` may return a different id.
- New or updated text becomes searchable only after the base's vectors are
  rebuilt ("Prepare for search") or its automatic re-sync runs.

Each operation is declared as a `knowledge://<op>` token in
`runtime.permissions.network` (the contract's `Permissions` is
`deny_unknown_fields`, so there is no separate `knowledge` key). An operation
not declared there is refused by the host.

## Results

Every tool returns JSON with `ok`. A failure is a value, never a trap:
`{"ok":false,"error":{"code":"…","message":"…"}}`, with `code` one of
`invalid_input`, `not_allowed`, `not_found`, `too_large`,
`invalid_media_type`, `quota_exceeded`, `unavailable`. A `limit` outside 1-20
is refused (`invalid_input`), not silently cut. `unavailable` means the host
could not complete the operation right now; from `knowledge_search` it most
often means the base has no embedding or search setup yet (it must be prepared
for search), and the message says so.

## Where it works

It needs a host that installs the knowledge port. In the designer v1 that is
Test chat (in process) only; Run Demo, the Test chat runner sidecar and every
deployed lane answer `unavailable`. The component imports all seven
functions, so it needs `greentic-ext-runtime` >= 1.2.43225676975; an older
host (no `knowledge` interface, or one without `list-bases`, `search` and
`update`) refuses to load it. A host whose port does not implement those
three answers `unavailable` for them.

## Build

```bash
bash ci/local_check.sh   # describe.json drift check, fmt, clippy, tests, wasm build
./build.sh               # builds and packs greentic.knowledge-<version>.gtxpack
```

The tools in `describe.json` are generated from `src/tool_meta.rs`
(`python3 -I ci/gen_describe.py`); `cargo test` and `local_check.sh` fail when
they differ. `build.sh` honours `CARGO_TARGET_DIR`. Releasing is a version bump
in `Cargo.toml` and `describe.json` on `main` (stable) or `develop` (`-dev`);
the repo's first commit never publishes.
