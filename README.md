# greentic.knowledge-writer

A Greentic design extension that lets an agentic worker write to its own
knowledge bases. It exposes four tools:

| tool | capabilities | what it does |
|---|---|---|
| `knowledge_create` | `cap://knowledge/create`, `agentic_worker` | create a text document in a base |
| `knowledge_read` | `cap://knowledge/read`, `agentic_worker` | read one document of a base |
| `knowledge_list` | `cap://knowledge/list`, `agentic_worker` | list a base's documents |
| `knowledge_remove` | `cap://knowledge/remove`, `agentic_worker` | remove a document from a base (the library copy is kept) |

Each tool is a thin wrapper over the host import
`greentic:extension-host/knowledge@0.1.0`. The extension holds no secrets and
makes no network calls.

## Rules the host enforces, not this extension

- The tenant, the team and the set of bases a call may touch come from the
  host. A tool names a base only by the name an operator sees; a base the
  worker is not bound to answers `not_allowed`, exactly like a base that does
  not exist.
- Size caps, accepted content kinds (plain text, markdown, CSV, JSON) and
  quotas are the host's. The tool only reports `too_large`,
  `invalid_media_type` or `quota_exceeded`.
- An identical document is reused (`created: false`), never duplicated. There
  is no update: remove, then create.
- A new document becomes searchable only after the base's vectors are rebuilt
  ("Prepare for search") or its automatic re-sync runs.

Each operation is declared as a `knowledge://<op>` token in
`runtime.permissions.network` (the contract's `Permissions` is
`deny_unknown_fields`, so there is no separate `knowledge` key). An operation
not declared there is refused by the host.

## Results

Every tool returns JSON with `ok`. A failure is a value, never a trap:
`{"ok":false,"error":{"code":"…","message":"…"}}`, with `code` one of
`invalid_input`, `not_allowed`, `not_found`, `too_large`,
`invalid_media_type`, `quota_exceeded`, `unavailable`.

## Where it works

It needs a host that installs the knowledge port. In the designer v1 that is
Test chat (in process) only; Run Demo, the Test chat runner sidecar and every
deployed lane answer `unavailable`. An older host that does not know the
`knowledge` interface refuses to load the extension.

## Build

```bash
bash ci/local_check.sh   # fmt, clippy, tests, wasm build
./build.sh               # builds and packs greentic.knowledge-writer-<version>.gtxpack
```

`build.sh` honours `CARGO_TARGET_DIR`. Releasing is a version bump in
`Cargo.toml` and `describe.json` on `main` (stable) or `develop` (`-dev`); the
repo's first commit never publishes.
