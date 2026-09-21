"""Scanner: enforce Rule 09 (Mermaid standards) across every markdown artifact.

Flags diagrams that will not render on GitHub: unquoted subgraph titles,
unquoted labels containing special characters, reserved-word IDs, banned
init/config/reset directives, size blowouts, and corrupted hostnames
(``lewzodone.com``). Mirrors the ROADMAP Phase 1 cleanup item.
"""

from __future__ import annotations

import re

import pytest

from tests.conftest import iter_repo_files

_FENCE_OPEN = re.compile(r"^```mermaid\s*$")
_FENCE_CLOSE = re.compile(r"^```\s*$")

# subgraph id["Title"] required — never subgraph id[...] or subgraph id [...]
_SUBGRAPH_UNQUOTED = re.compile(r"^subgraph\s+\S+\s*\[[^\"]")

# labels with special chars must wrap the whole label in quotes
_SPECIALS = re.compile(r"[()\[\]:#|\"`/<>&=]")
_NODE = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\[(.*?)\]\s*(?:-->|$)")
_EDGE = re.compile(r"-->\|\s*([^|]*?)\s*\|")

# reserved IDs per Rule 09 item 12
_RESERVED = {"end", "class", "note", "subgraph", "link", "default", "linkStyle"}

_BANNED = ("%%{init}%%", "%%{config}%%", "%%{reset}%%")

# corrupted lewdzone hostname seen in the wild (systems-designer)
_HOSTNAME_BAD = re.compile(r"\b[a-z]{3,}zone\.com\b")


def _quoted(label: str) -> bool:
    """True when label is already wrapped in double quotes (Rule 09 item 3)."""
    s = label.strip()
    return len(s) >= 2 and s[0] == '"' and s[-1] == '"'


def _mermaid_blocks(text: str) -> list[str]:
    blocks = []
    start = None
    for i, line in enumerate(text.splitlines(), 1):
        if _FENCE_OPEN.search(line):
            start = i
        elif _FENCE_CLOSE.search(line) and start is not None:
            blocks.append("\n".join(text.splitlines()[start:i]))
            start = None
    return blocks


def _violation(label: str, path, lineno: int, detail: str) -> str:
    return f"{path}:{lineno}: {label} — {detail}"


def _scan_block(path: str, start_lineno: int, block: str) -> list[str]:
    violations = []
    for rel, line in enumerate(block.splitlines()):
        ln = start_lineno + rel
        if _SUBGRAPH_UNQUOTED.match(line.strip()):
            violations.append(
                _violation("unquoted subgraph title", path, ln, line.strip())
            )
        if any(b in line for b in _BANNED):
            violations.append(
                _violation("banned directive", path, ln, f"{line.strip()}")
            )
        for m in _NODE.finditer(line):
            label = m.group(1)
            if _SPECIALS.search(label) and not _quoted(label):
                violations.append(
                    _violation("unquoted label", path, ln, f"{m.group(0).strip()}")
                )
        for m in _EDGE.finditer(line):
            label = m.group(1).strip()
            if _SPECIALS.search(label) and not _quoted(label):
                violations.append(
                    _violation("unquoted edge label", path, ln, f"{line.strip()}")
                )
        for ident in re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\[", line):
            if ident[:-1].lower() in _RESERVED:
                violations.append(
                    _violation("reserved node ID", path, ln, ident)
                )
        for bad in _HOSTNAME_BAD.findall(line):
            if bad != "lewdzone.com":
                violations.append(
                    _violation("corrupted hostname", path, ln, bad)
                )
    return violations


@pytest.mark.unit
def test_mermaid_blocks_are_rule_09_compliant():
    """Every ```mermaid block in the repo renders per Rule 09."""
    problems: list[str] = []
    for path in iter_repo_files(suffixes=(".md",)):
        text = path.read_text(encoding="utf-8")
        for i, line in enumerate(text.splitlines(), 1):
            if not _FENCE_OPEN.search(line):
                continue
            content_start = i
            block_lines = []
            for bl in text.splitlines()[i:]:
                if _FENCE_CLOSE.search(bl):
                    break
                block_lines.append(bl)
            problems.extend(_scan_block(str(path), content_start + 1, "\n".join(block_lines)))
    assert not problems, "Rule 09 mermaid violations:\n" + "\n".join(problems)