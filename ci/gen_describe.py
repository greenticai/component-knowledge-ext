#!/usr/bin/env python3
"""Rewrite describe.json's tools from src/tool_meta.rs, so the two cannot drift.

Run after editing tool_meta.rs; `cargo test` fails if they disagree.
Usage: python3 -I ci/gen_describe.py
"""
import json
import pathlib
import re

root = pathlib.Path(__file__).resolve().parent.parent
src = (root / "src/tool_meta.rs").read_text()
consts = {m[1]: m[2] for m in re.finditer(r'const (\w+): &str = r#"(.*?)"#;', src, re.S)}
names = {m[1]: m[2] for m in re.finditer(r'pub const (\w+_TOOL): &str = "(.*?)";', src)}
tools = []
for block in re.findall(r"ToolMeta \{\n            (name:.*?)\n        \}", src, re.S):
    field = lambda k: re.search(rf"{k}: (.*?),\n", block, re.S)[1].strip()
    desc = json.loads(field("description"))
    op = re.search(r'caps\("(.*?)"\)', block)[1]
    tools.append({
        "name": names[field("name")],
        "export": "greentic:extension-design/tools.invoke-tool",
        "runtime_ref": "knowledge",
        "capabilities": [f"cap://knowledge/{op}", "agentic_worker"],
        "description": desc,
        "input_schema": consts[field("input_schema_json")],
        "output_schema": consts[field("output_schema_json")],
        "secret_requirements": [],
    })
path = root / "describe.json"
d = json.loads(path.read_text())
d["contributions"]["tools"] = tools
d["runtime"]["permissions"]["network"] = [
    f"knowledge://{op}" for op in
    ["create", "read", "list", "remove", "list-bases", "search", "update"]
]
path.write_text(json.dumps(d, indent=2, ensure_ascii=False) + "\n")
print(f"wrote {len(tools)} tools")
