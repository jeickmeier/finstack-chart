#!/usr/bin/env python3
"""Check workspace boundaries and local Markdown file links; Python 3.9+, stdlib only."""

import json
import re
import subprocess
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
ALLOWED = {
    "chart-core": set(),
    "chart-export": {"chart-core"},
    "gpui-charts": {"chart-core"},
    "gpui-charts-kit": {"chart-core", "gpui-charts"},
    "chart-python": {"chart-core", "chart-export"},
    "chart-wasm": {"chart-core"},
    "chart-gallery": {"chart-core", "chart-export", "gpui-charts", "gpui-charts-kit"},
}


def main():
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--locked"], cwd=ROOT, text=True
    ))
    packages = {p["id"]: p for p in metadata["packages"]}
    members = [packages[key] for key in metadata["workspace_members"]]
    errors = []
    if {p["name"] for p in members} != set(ALLOWED):
        errors.append("Workspace membership changed; reconcile ARC-01 and the boundary policy.")
    defaults = {packages[key]["name"] for key in metadata["workspace_default_members"]}
    if defaults != {"chart-core", "chart-export", "gpui-charts"}:
        errors.append("Default members must keep Kit, gallery and binding proofs opt-in.")
    for package in members:
        name = package["name"]
        if package["publish"] != []:
            errors.append(f"{name}: publication must remain disabled until release preparation.")
        for dep in package["dependencies"]:
            if dep["name"] in ALLOWED and dep["name"] not in ALLOWED.get(name, set()):
                errors.append(f"{name}: forbidden workspace edge to {dep['name']}")

    # Inspect the resolved transitive graph, including build/dev dependencies.
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    forbidden = {"gpui", "gpui-pre", "gpui-component", "gpui-kit", "gpui-charts",
                 "gpui-charts-kit", "pyo3", "wasm-bindgen", "js-sys", "web-sys",
                 "chart-python", "chart-wasm", "chart-gallery"}
    for package in members:
        if package["name"] not in {"chart-core", "chart-export"}:
            continue
        seen = set()
        pending = list(nodes[package["id"]]["dependencies"])
        while pending:
            key = pending.pop()
            if key in seen:
                continue
            seen.add(key)
            dep_name = packages[key]["name"]
            if dep_name in forbidden or dep_name.startswith("gpui-pre"):
                errors.append(f"{package['name']}: forbidden transitive host dependency {dep_name}")
            pending.extend(nodes[key]["dependencies"])

    # ARC-04: the Kit and standalone consumers must resolve one GPUI identity.
    gpui_packages = [p for p in packages.values() if p["name"] in {"gpui", "gpui-pre"}]
    if len(gpui_packages) != 1 or gpui_packages[0]["name"] != "gpui-pre":
        errors.append("Expected exactly one gpui-pre identity; reconcile ADR-001.")
    for package in members:
        for dep in package["dependencies"]:
            if dep["name"] in {"gpui-pre", "gpui-pre-platform", "gpui-kit"}:
                if not dep["req"].startswith("="):
                    errors.append(f"{package['name']}: host dependency {dep['name']} needs an exact pin.")
    standalone = next(p for p in members if p["name"] == "gpui-charts")
    seen = set()
    pending = list(nodes[standalone["id"]]["dependencies"])
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen.add(key)
        if packages[key]["name"] in {"gpui-kit", "gpui-component", "gpui-charts-kit"}:
            errors.append("Standalone GPUI must not require Kit.")
        pending.extend(nodes[key]["dependencies"])

    documents = [ROOT / "README.md", ROOT / "AGENTS.md"]
    documents += list((ROOT / "docs").rglob("*.md"))
    documents += list((ROOT / ".agents" / "skills").rglob("*.md"))
    documents += list((ROOT / "fixtures").rglob("*.md"))
    documents += list((ROOT / "benches").rglob("*.md"))
    for document in documents:
        if not document.is_file():
            errors.append(f"Missing document: {document.relative_to(ROOT)}")
            continue
        body = re.sub(r"```.*?```", "", document.read_text(), flags=re.S)
        for target in re.findall(r"\[[^\]]*\]\(([^\s)]+)\)", body):
            url = urlsplit(target)
            if url.scheme or url.netloc or not url.path:
                continue
            if not (document.parent / unquote(url.path)).exists():
                errors.append(f"{document.relative_to(ROOT)}: broken local link {target}")
    if errors:
        raise SystemExit("\n".join(errors))
    print("PASS: workspace edges, host isolation, single pinned GPUI identity, optional Kit and local Markdown file links.")
    print("Scope: default resolved features; external links/anchors and unlisted host packages need review.")


if __name__ == "__main__":
    main()
