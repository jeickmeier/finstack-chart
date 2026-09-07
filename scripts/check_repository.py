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
    "chart-wasm": {"chart-core", "chart-export"},
    "chart-gallery": {"chart-core", "chart-export", "gpui-charts", "gpui-charts-kit"},
}
PORTABLE_TARGETS = ("aarch64-apple-darwin", "x86_64-unknown-linux-gnu", "wasm32-unknown-unknown")


def metadata_for(target=None):
    command = ["cargo", "metadata", "--format-version", "1", "--locked"]
    if target is not None:
        command += ["--filter-platform", target]
    return json.loads(subprocess.check_output(command, cwd=ROOT, text=True))


def dependency_ids(root_id, nodes):
    """Walk normal/build/dev edges present in this resolved feature/target graph."""
    seen = set()
    pending = list(nodes[root_id]["dependencies"])
    while pending:
        key = pending.pop()
        if key not in seen:
            seen.add(key)
            pending.extend(nodes[key]["dependencies"])
    return seen


def main():
    metadata = metadata_for()
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

    # Check explicit supported target graphs, not only the invoking machine's graph.
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    forbidden = {"gpui", "gpui-pre", "gpui-component", "gpui-kit", "gpui-charts",
                 "gpui-charts-kit", "pyo3", "wasm-bindgen", "js-sys", "web-sys",
                 "chart-python", "chart-wasm", "chart-gallery"}
    for target in PORTABLE_TARGETS:
        target_metadata = metadata_for(target)
        target_nodes = {n["id"]: n for n in target_metadata["resolve"]["nodes"]}
        target_packages = {p["id"]: p for p in target_metadata["packages"]}
        for package in members:
            if package["name"] not in {"chart-core", "chart-export"}:
                continue
            for key in dependency_ids(package["id"], target_nodes):
                dep_name = target_packages[key]["name"]
                if dep_name in forbidden or dep_name.startswith("gpui-pre"):
                    errors.append(f"{target}: {package['name']}: forbidden transitive host dependency {dep_name}")

    # ARC-04: the Kit and standalone consumers must resolve one GPUI identity.
    gpui_packages = [p for p in packages.values() if p["name"] in {"gpui", "gpui-pre"}]
    if len(gpui_packages) != 1 or gpui_packages[0]["name"] != "gpui-pre":
        errors.append("Expected exactly one gpui-pre identity; reconcile ADR-001.")
    for package in members:
        for dep in package["dependencies"]:
            if dep["name"] in {"gpui-pre", "gpui-pre-platform", "gpui-kit"}:
                if not dep["req"].startswith("="):
                    errors.append(f"{package['name']}: host dependency {dep['name']} needs an exact pin.")
    for standalone in (p for p in members if p["name"] == "gpui-charts"):
        for key in dependency_ids(standalone["id"], nodes):
            if packages[key]["name"] in {"gpui-kit", "gpui-component", "gpui-charts-kit"}:
                errors.append("Standalone GPUI must not require Kit.")

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
    print("Core/export isolation targets: " + ", ".join(PORTABLE_TARGETS))
    print("Scope: default resolved features; graph checks are not target execution. External links/anchors and unlisted host packages need review.")


if __name__ == "__main__":
    main()
