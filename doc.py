#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
doc_generator.py
=================

PDF documentation generator for an entire repository/codebase, without
compiling or running anything. Works purely statically:

  - for Python it uses the `ast` module (parsing only, zero code execution),
  - for every other language it uses regex heuristics (function/class
    signatures, access modifiers such as public/private/pub/export/static).

The resulting PDF has 4 sections, in this order:

  1. PUBLIC API / ABI          - signatures only, of everything exported
                                  to the outside world (public / export / pub)
  2. INTERNAL FUNCTIONS        - signatures of everything private/local
  3. CODE FRAGMENTS + COMMENTS - for each definition: the comment/docstring
                                  directly above it, plus a code snippet
  4. .md FILES                 - full content of every Markdown file, in order

Every entry in every section shows the EXACT file path (relative to the
scanned root directory) and the line number.

Usage:
    python doc_generator.py /path/to/project -o documentation.pdf

Options:
    -o, --output FILE          output PDF file name (default: documentation.pdf)
    --max-body-lines N         how many lines of code to show in the fragments
                                section (default: 12)
    --max-items-per-section N  hard cap on entries in section 1/2, so the PDF
                                doesn't explode on huge repos (default: 4000)
    --exclude DIR1,DIR2        extra directories to skip
    --ext .py,.js,...          restrict the scan to these extensions only
    --split                    write one PDF per section instead of a single
                                file (recommended for large repos)

Requires: pip install reportlab
"""

from __future__ import annotations

import argparse
import ast
import os
import re
import sys
from dataclasses import dataclass, field
from xml.sax.saxutils import escape

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import mm
from reportlab.lib.enums import TA_LEFT
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, PageBreak, Preformatted, KeepTogether
)
from reportlab.lib import colors


# --------------------------------------------------------------------------- #
# Configuration: directories to skip, extensions, etc.
# --------------------------------------------------------------------------- #

DEFAULT_SKIP_DIRS = {
    ".git", ".hg", ".svn", "node_modules", "venv", ".venv", "env",
    "__pycache__", "dist", "build", "target", ".idea", ".vscode",
    "vendor", "bin", "obj", ".mypy_cache", ".pytest_cache", "coverage",
    ".next", ".gradle", ".tox", "site-packages",
}

# extension -> language name (used for grouping in the PDF)
LANG_BY_EXT = {
    ".py": "Python",
    ".js": "JavaScript", ".jsx": "JavaScript (JSX)", ".mjs": "JavaScript",
    ".cjs": "JavaScript",
    ".ts": "TypeScript", ".tsx": "TypeScript (TSX)",
    ".java": "Java",
    ".cs": "C#",
    ".go": "Go",
    ".rs": "Rust",
    ".c": "C", ".h": "C (header)",
    ".cpp": "C++", ".cc": "C++", ".cxx": "C++",
    ".hpp": "C++ (header)", ".hh": "C++ (header)",
    ".php": "PHP",
    ".rb": "Ruby",
    ".kt": "Kotlin", ".kts": "Kotlin",
    ".swift": "Swift",
    ".m": "Objective-C",
    ".scala": "Scala",
    ".d": "D",
    ".nim": "Nim",
    ".odin": "Odin",
    ".ads": "Ada (spec)", ".adb": "Ada (body)",
    ".asm": "Assembly", ".s": "Assembly", ".S": "Assembly",
}

MD_EXTS = {".md", ".markdown"}

# which prefix marks a comment line in each language (used to pull the
# comment/docstring directly above a definition)
COMMENT_PREFIXES = {
    ".py": ("#",),
    ".js": ("//", "/*", "*"), ".jsx": ("//", "/*", "*"), ".mjs": ("//", "/*", "*"),
    ".cjs": ("//", "/*", "*"),
    ".ts": ("//", "/*", "*"), ".tsx": ("//", "/*", "*"),
    ".java": ("//", "/*", "*"),
    ".cs": ("//", "/*", "*", "///"),
    ".go": ("//",),
    ".rs": ("//", "///", "//!"),
    ".c": ("//", "/*", "*"), ".h": ("//", "/*", "*"),
    ".cpp": ("//", "/*", "*"), ".cc": ("//", "/*", "*"), ".cxx": ("//", "/*", "*"),
    ".hpp": ("//", "/*", "*"), ".hh": ("//", "/*", "*"),
    ".php": ("//", "#", "/*", "*"),
    ".rb": ("#",),
    ".kt": ("//", "/*", "*"), ".kts": ("//", "/*", "*"),
    ".swift": ("//", "/*", "*"),
    ".m": ("//", "/*", "*"),
    ".scala": ("//", "/*", "*"),
    ".d": ("//", "/*", "*"),
    ".nim": ("#",),
    ".odin": ("//",),
    ".ads": ("--",), ".adb": ("--",),
    ".asm": (";",), ".s": (";", "//", "#"), ".S": (";", "//", "#"),
}


@dataclass
class Definition:
    path: str          # path relative to the scanned root
    lineno: int
    kind: str          # "function" / "class" / "method" / "type" / "const" / "export"
    name: str
    signature: str      # signature line (short, used in section 1/2)
    is_public: bool
    comment: str = ""    # comment/docstring directly above the definition
    body_snippet: str = ""  # first few lines of code (section 3)
    language: str = ""


@dataclass
class MarkdownFile:
    path: str
    content: str


# --------------------------------------------------------------------------- #
# Collecting files
# --------------------------------------------------------------------------- #

def iter_source_files(root: str, exts: set[str] | None, exclude_dirs: set[str]):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in exclude_dirs and not d.startswith(".")
                       or d in (".",)]
        for fn in filenames:
            ext = os.path.splitext(fn)[1].lower()
            full = os.path.join(dirpath, fn)
            rel = os.path.relpath(full, root)
            if ext in MD_EXTS:
                yield rel, full, ext
            elif exts is not None:
                if ext in exts:
                    yield rel, full, ext
            elif ext in LANG_BY_EXT:
                yield rel, full, ext


def read_text(path: str) -> str:
    for enc in ("utf-8", "utf-8-sig", "latin-1"):
        try:
            with open(path, "r", encoding=enc) as f:
                return f.read()
        except (UnicodeDecodeError, LookupError):
            continue
    with open(path, "r", encoding="utf-8", errors="replace") as f:
        return f.read()


# --------------------------------------------------------------------------- #
# Extracting the comment directly above line `lineno` (1-indexed)
# --------------------------------------------------------------------------- #

def extract_preceding_comment(lines: list[str], lineno_0based: int, ext: str) -> str:
    prefixes = COMMENT_PREFIXES.get(ext, ("#", "//"))
    collected = []
    i = lineno_0based - 1
    in_block_close = False
    while i >= 0:
        line = lines[i].rstrip("\n")
        stripped = line.strip()
        if stripped == "":
            i -= 1
            if collected:
                # a blank line ends the comment block (unless we haven't
                # collected anything yet)
                break
            continue
        if stripped.endswith("*/") and not stripped.startswith(("//",)):
            collected.append(line)
            in_block_close = True
            i -= 1
            continue
        if any(stripped.startswith(p) for p in prefixes) or in_block_close:
            collected.append(line)
            if stripped.startswith("/*"):
                in_block_close = False
                i -= 1
                continue
            i -= 1
            continue
        break
    collected.reverse()
    return "\n".join(collected).strip()


# --------------------------------------------------------------------------- #
# PYTHON - parsed via ast (no code execution)
# --------------------------------------------------------------------------- #

def parse_python(rel_path: str, full_path: str, source: str) -> list[Definition]:
    defs: list[Definition] = []
    try:
        tree = ast.parse(source, filename=rel_path)
    except SyntaxError:
        return defs

    lines = source.splitlines()

    all_list = None
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id == "__all__":
                    try:
                        all_list = ast.literal_eval(node.value)
                    except Exception:
                        all_list = None

    def is_public_name(name: str) -> bool:
        if all_list is not None:
            return name in all_list
        return not name.startswith("_")

    def sig_line(node) -> str:
        ln = node.lineno - 1
        if 0 <= ln < len(lines):
            text = lines[ln].strip()
            if not text.rstrip().endswith(":"):
                text += " ..."
            return text
        return f"{type(node).__name__} {getattr(node, 'name', '')}"

    def body_snippet(node, max_lines: int) -> str:
        start = node.lineno - 1
        end = getattr(node, "end_lineno", node.lineno) - 1
        snippet_lines = lines[start:min(end + 1, start + max_lines)]
        snippet = "\n".join(snippet_lines)
        if end - start + 1 > max_lines:
            snippet += "\n    # ... (truncated, see source file for more)"
        return snippet

    def handle(node, kind: str, class_ctx: str | None = None):
        name = node.name
        public = is_public_name(name)
        docstring = ast.get_docstring(node) or ""
        comment_above = extract_preceding_comment(lines, node.lineno - 1, ".py")
        comment = docstring if docstring else comment_above
        full_name = f"{class_ctx}.{name}" if class_ctx else name
        defs.append(Definition(
            path=rel_path, lineno=node.lineno, kind=kind, name=full_name,
            signature=sig_line(node), is_public=public, comment=comment,
            body_snippet=body_snippet(node, 12), language="Python",
        ))

    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            handle(node, "function")
        elif isinstance(node, ast.ClassDef):
            handle(node, "class")
            for sub in node.body:
                if isinstance(sub, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    handle(sub, "method", class_ctx=node.name)

    return defs


# --------------------------------------------------------------------------- #
# GENERIC REGEX PARSER FOR ALL OTHER LANGUAGES
# --------------------------------------------------------------------------- #

# Each rule: (kind, regex, group_index, decide_public)
# decide_public(match) -> bool, or None when visibility depends on context
# (tracked separately in parse_generic, e.g. Ruby's private/public keyword,
# Ada's "private" section, Odin's @(private) attribute)

def _rules_js_ts():
    return [
        ("function", re.compile(r"^\s*export\s+default\s+(async\s+)?function\s*\*?\s*(\w+)"), 2, lambda m: True),
        ("function", re.compile(r"^\s*export\s+(async\s+)?function\s*\*?\s+(\w+)"), 2, lambda m: True),
        ("class", re.compile(r"^\s*export\s+(default\s+)?class\s+(\w+)"), 2, lambda m: True),
        ("const", re.compile(r"^\s*export\s+(const|let|var)\s+(\w+)"), 2, lambda m: True),
        ("interface", re.compile(r"^\s*export\s+interface\s+(\w+)"), 1, lambda m: True),
        ("type", re.compile(r"^\s*export\s+type\s+(\w+)"), 1, lambda m: True),
        ("export", re.compile(r"^\s*module\.exports(\.(\w+))?\s*="), 2, lambda m: True),
        ("function", re.compile(r"^\s*(async\s+)?function\s*\*?\s+(\w+)"), 2, lambda m: False),
        ("const", re.compile(r"^\s*(const|let|var)\s+(\w+)\s*=\s*(\(|async\s*\(|function)"), 2, lambda m: False),
        ("method", re.compile(r"^\s*(private|#)\s*(\w+)\s*\("), 2, lambda m: False),
    ]


def _rules_java_cs():
    return [
        ("class", re.compile(r"^\s*public\s+(static\s+)?(final\s+)?(abstract\s+)?(class|interface|enum|record)\s+(\w+)"), 5, lambda m: True),
        ("method", re.compile(r"^\s*public\s+(static\s+)?(final\s+)?[\w<>\[\],\s]+?\s(\w+)\s*\("), 2, lambda m: True),
        ("class", re.compile(r"^\s*(private|protected|internal)\s+(static\s+)?(class|interface|enum|record)\s+(\w+)"), 4, lambda m: False),
        ("method", re.compile(r"^\s*(private|protected|internal)\s+(static\s+)?[\w<>\[\],\s]+?\s(\w+)\s*\("), 3, lambda m: False),
    ]


def _rules_go():
    return [
        ("type", re.compile(r"^\s*type\s+([A-Z]\w*)\s+(struct|interface)"), 1, lambda m: True),
        ("type", re.compile(r"^\s*type\s+([a-z_]\w*)\s+(struct|interface)"), 1, lambda m: False),
        ("function", re.compile(r"^\s*func\s+(\([^)]*\)\s*)?([A-Z]\w*)\s*\("), 2, lambda m: True),
        ("function", re.compile(r"^\s*func\s+(\([^)]*\)\s*)?([a-z_]\w*)\s*\("), 2, lambda m: False),
    ]


def _rules_rust():
    return [
        ("function", re.compile(r"^\s*pub(\([^)]*\))?\s+fn\s+(\w+)"), 2, lambda m: True),
        ("struct", re.compile(r"^\s*pub(\([^)]*\))?\s+struct\s+(\w+)"), 2, lambda m: True),
        ("enum", re.compile(r"^\s*pub(\([^)]*\))?\s+enum\s+(\w+)"), 2, lambda m: True),
        ("trait", re.compile(r"^\s*pub(\([^)]*\))?\s+trait\s+(\w+)"), 2, lambda m: True),
        ("function", re.compile(r"^\s*fn\s+(\w+)"), 1, lambda m: False),
        ("struct", re.compile(r"^\s*struct\s+(\w+)"), 1, lambda m: False),
    ]


def _rules_c_cpp():
    return [
        ("function", re.compile(r"^\s*static\s+[\w:\*&<>,\s]+?\s(\w+)\s*\([^;{]*\)\s*\{"), 1, lambda m: False),
        ("function", re.compile(r"^\s*[\w:\*&<>,\s]+?\s(\w+)\s*\([^;{]*\)\s*\{"), 1, lambda m: True),
        ("declaration", re.compile(r"^\s*[\w:\*&<>,\s]+?\s(\w+)\s*\([^;{]*\)\s*;"), 1, lambda m: True),
    ]


def _rules_php():
    return [
        ("method", re.compile(r"^\s*public\s+(static\s+)?function\s+(\w+)"), 2, lambda m: True),
        ("method", re.compile(r"^\s*(private|protected)\s+(static\s+)?function\s+(\w+)"), 3, lambda m: False),
        ("function", re.compile(r"^\s*function\s+(\w+)"), 1, lambda m: True),
        ("class", re.compile(r"^\s*class\s+(\w+)"), 1, lambda m: True),
    ]


def _rules_ruby():
    return [
        ("method", re.compile(r"^\s*def\s+(self\.)?(\w+[\?!]?)"), 2, None),  # visibility resolved contextually
        ("class", re.compile(r"^\s*class\s+(\w+)"), 1, lambda m: True),
        ("module", re.compile(r"^\s*module\s+(\w+)"), 1, lambda m: True),
    ]


def _rules_kotlin_swift():
    return [
        ("function", re.compile(r"^\s*(public|open)\s+fun\s+(\w+)"), 2, lambda m: True),
        ("function", re.compile(r"^\s*(private|internal|fileprivate)\s+fun\s+(\w+)"), 2, lambda m: False),
        ("function", re.compile(r"^\s*fun\s+(\w+)"), 1, lambda m: True),
        ("class", re.compile(r"^\s*(public|open)\s+class\s+(\w+)"), 2, lambda m: True),
        ("class", re.compile(r"^\s*(private|internal|fileprivate)\s+class\s+(\w+)"), 2, lambda m: False),
        ("class", re.compile(r"^\s*class\s+(\w+)"), 1, lambda m: True),
    ]


def _rules_d():
    return [
        ("class", re.compile(r"^\s*private\s+(class|struct|interface|enum|union)\s+(\w+)"), 2, lambda m: False),
        ("class", re.compile(r"^\s*(class|struct|interface|enum|union)\s+(\w+)"), 2, lambda m: True),
        ("function", re.compile(r"^\s*private\s+[\w!\[\]\*<>,\.\s]+?\s(\w+)\s*\([^;{]*\)\s*\{"), 1, lambda m: False),
        ("function", re.compile(r"^\s*[\w!\[\]\*<>,\.\s]+?\s(\w+)\s*\([^;{]*\)\s*\{"), 1, lambda m: True),
    ]


def _rules_nim():
    return [
        ("proc", re.compile(r"^\s*(proc|func|template|macro|method|iterator)\s+(\w+)\*"), 2, lambda m: True),
        ("proc", re.compile(r"^\s*(proc|func|template|macro|method|iterator)\s+(\w+)\b"), 2, lambda m: False),
        ("type", re.compile(r"^\s*type\s+(\w+)\*"), 1, lambda m: True),
        ("type", re.compile(r"^\s*type\s+(\w+)\b"), 1, lambda m: False),
    ]


_ODIN_PRIVATE_ATTR = re.compile(r"@\(\s*private")


def _rules_odin():
    # visibility resolved contextually (an @(private) attribute on the line
    # above) - see parse_generic
    return [
        ("proc", re.compile(r"^\s*(\w+)\s*::\s*proc\b"), 1, None),
        ("struct", re.compile(r"^\s*(\w+)\s*::\s*struct\b"), 1, None),
        ("enum", re.compile(r"^\s*(\w+)\s*::\s*enum\b"), 1, None),
        ("union", re.compile(r"^\s*(\w+)\s*::\s*union\b"), 1, None),
    ]


def _rules_ada_spec():
    # in a .ads file (package specification): visibility depends on whether
    # the declaration comes before or after the "private" section
    return [
        ("package", re.compile(r"^\s*package\s+(\w+)"), 1, lambda m: True),
        ("procedure", re.compile(r"^\s*procedure\s+(\w+)"), 1, None),
        ("function", re.compile(r"^\s*function\s+(\w+)"), 1, None),
    ]


def _rules_ada_body():
    # in a .adb file (implementation) - treated as internal detail
    return [
        ("procedure", re.compile(r"^\s*procedure\s+(\w+)"), 1, lambda m: False),
        ("function", re.compile(r"^\s*function\s+(\w+)"), 1, lambda m: False),
    ]


def _rules_asm():
    return [
        ("label", re.compile(r"^\s*\.?(global|globl)\s+(\w+)"), 2, lambda m: True),
        ("label", re.compile(r"^([A-Za-z_]\w*)\s*:"), 1, lambda m: False),
    ]


RULES_BY_EXT = {
    ".js": _rules_js_ts(), ".jsx": _rules_js_ts(), ".mjs": _rules_js_ts(), ".cjs": _rules_js_ts(),
    ".ts": _rules_js_ts(), ".tsx": _rules_js_ts(),
    ".java": _rules_java_cs(), ".cs": _rules_java_cs(),
    ".go": _rules_go(),
    ".rs": _rules_rust(),
    ".c": _rules_c_cpp(), ".h": _rules_c_cpp(),
    ".cpp": _rules_c_cpp(), ".cc": _rules_c_cpp(), ".cxx": _rules_c_cpp(),
    ".hpp": _rules_c_cpp(), ".hh": _rules_c_cpp(),
    ".php": _rules_php(),
    ".rb": _rules_ruby(),
    ".kt": _rules_kotlin_swift(), ".kts": _rules_kotlin_swift(),
    ".swift": _rules_kotlin_swift(),
    ".d": _rules_d(),
    ".nim": _rules_nim(),
    ".odin": _rules_odin(),
    ".ads": _rules_ada_spec(),
    ".adb": _rules_ada_body(),
    ".asm": _rules_asm(), ".s": _rules_asm(), ".S": _rules_asm(),
}


def parse_generic(rel_path: str, ext: str, source: str, max_body_lines: int) -> list[Definition]:
    rules = RULES_BY_EXT.get(ext)
    if not rules:
        return []
    lines = source.splitlines()
    lang = LANG_BY_EXT.get(ext, ext)
    defs: list[Definition] = []

    ruby_private_mode = False  # .rb only
    ada_private_mode = False   # .ads only

    for idx, raw_line in enumerate(lines):
        if ext == ".rb":
            s = raw_line.strip()
            if s == "private":
                ruby_private_mode = True
                continue
            if s in ("public", "protected"):
                ruby_private_mode = False
                continue

        if ext == ".ads":
            s = raw_line.strip()
            if s == "private":
                ada_private_mode = True
                continue

        for kind, regex, group_idx, decide_public in rules:
            m = regex.match(raw_line)
            if not m:
                continue
            try:
                name = m.group(group_idx) or m.group(m.lastindex)
            except Exception:
                name = m.group(m.lastindex or 1)
            if not name:
                continue
            if ext == ".rb" and decide_public is None:
                public = not ruby_private_mode
            elif ext == ".ads" and decide_public is None:
                public = not ada_private_mode
            elif ext == ".odin" and decide_public is None:
                # check the closest non-blank line above for an @(private) attribute
                public = True
                j = idx - 1
                while j >= 0 and lines[j].strip() == "":
                    j -= 1
                if j >= 0 and _ODIN_PRIVATE_ATTR.search(lines[j]):
                    public = False
            else:
                public = decide_public(m) if decide_public else True
            comment = extract_preceding_comment(lines, idx, ext)
            end = min(idx + max_body_lines, len(lines))
            snippet = "\n".join(lines[idx:end])
            if end - idx >= max_body_lines and end < len(lines):
                snippet += "\n    // ... (truncated, see source file for more)"
            defs.append(Definition(
                path=rel_path, lineno=idx + 1, kind=kind, name=name,
                signature=raw_line.strip(), is_public=public,
                comment=comment, body_snippet=snippet, language=lang,
            ))
            break  # one matching rule per line is enough
    return defs


# --------------------------------------------------------------------------- #
# Collecting everything
# --------------------------------------------------------------------------- #

def collect(root: str, exts: set[str] | None, exclude_dirs: set[str], max_body_lines: int):
    all_defs: list[Definition] = []
    md_files: list[MarkdownFile] = []

    for rel_path, full_path, ext in iter_source_files(root, exts, exclude_dirs):
        source = read_text(full_path)
        if ext in MD_EXTS:
            md_files.append(MarkdownFile(path=rel_path.replace(os.sep, "/"), content=source))
            continue
        rel_norm = rel_path.replace(os.sep, "/")
        if ext == ".py":
            all_defs.extend(parse_python(rel_norm, full_path, source))
        else:
            all_defs.extend(parse_generic(rel_norm, ext, source, max_body_lines))

    md_files.sort(key=lambda m: m.path)
    all_defs.sort(key=lambda d: (d.language, d.path, d.lineno))
    return all_defs, md_files


# --------------------------------------------------------------------------- #
# Building the PDF
# --------------------------------------------------------------------------- #

def build_styles():
    styles = getSampleStyleSheet()
    styles.add(ParagraphStyle(
        name="SectionTitle", parent=styles["Heading1"],
        fontSize=18, spaceAfter=14, spaceBefore=6, textColor=colors.HexColor("#1a1a2e"),
    ))
    styles.add(ParagraphStyle(
        name="LangHeading", parent=styles["Heading2"],
        fontSize=13, spaceBefore=14, spaceAfter=6, textColor=colors.HexColor("#16213e"),
    ))
    styles.add(ParagraphStyle(
        name="ItemPath", parent=styles["Normal"],
        fontSize=8.5, textColor=colors.HexColor("#555555"), fontName="Courier",
        spaceAfter=2,
    ))
    styles.add(ParagraphStyle(
        name="ItemSig", parent=styles["Normal"],
        fontSize=9.5, fontName="Courier-Bold", spaceAfter=4, textColor=colors.HexColor("#0f3460"),
        alignment=TA_LEFT,
    ))
    styles.add(ParagraphStyle(
        name="CommentText", parent=styles["Normal"],
        fontSize=8.5, fontName="Helvetica-Oblique", textColor=colors.HexColor("#4a4a4a"),
        spaceAfter=4, leftIndent=8,
    ))
    styles.add(ParagraphStyle(
        name="CodeBlock", parent=styles["Code"],
        fontSize=8, leading=10, backColor=colors.HexColor("#f4f4f8"),
        borderPadding=6, spaceAfter=10,
    ))
    styles.add(ParagraphStyle(
        name="MdHeading", parent=styles["Heading2"],
        fontSize=12, spaceBefore=16, spaceAfter=4, textColor=colors.HexColor("#16213e"),
    ))
    return styles


def group_by_language(defs: list[Definition]):
    groups: dict[str, list[Definition]] = {}
    for d in defs:
        groups.setdefault(d.language, []).append(d)
    return dict(sorted(groups.items()))


def path_line_label(d: Definition) -> str:
    return f"{d.path}  (line {d.lineno})"


def render_signature_section(story, styles, title: str, defs: list[Definition], max_items: int):
    story.append(Paragraph(escape(title), styles["SectionTitle"]))
    if not defs:
        story.append(Paragraph("No items in this section.", styles["Normal"]))
        return
    truncated = len(defs) > max_items
    defs = defs[:max_items]
    for lang, items in group_by_language(defs).items():
        story.append(Paragraph(escape(lang), styles["LangHeading"]))
        for d in items:
            block = [
                Paragraph(escape(path_line_label(d)), styles["ItemPath"]),
                Paragraph(escape(f"[{d.kind}] {d.signature}"), styles["ItemSig"]),
            ]
            story.append(KeepTogether(block))
    if truncated:
        story.append(Spacer(1, 6))
        story.append(Paragraph(
            f"(List truncated to {max_items} entries to keep the document size "
            f"manageable. Use --max-items-per-section to change this.)",
            styles["CommentText"],
        ))


def render_fragments_section(story, styles, defs: list[Definition], max_items: int):
    story.append(Paragraph("3. Code fragments with comments", styles["SectionTitle"]))
    if not defs:
        story.append(Paragraph("No items.", styles["Normal"]))
        return
    truncated = len(defs) > max_items
    defs = defs[:max_items]
    for lang, items in group_by_language(defs).items():
        story.append(Paragraph(escape(lang), styles["LangHeading"]))
        for d in items:
            block = [Paragraph(escape(path_line_label(d) + f" - [{'public' if d.is_public else 'internal'}] {d.name}"), styles["ItemPath"])]
            if d.comment:
                comment_html = escape(d.comment).replace("\n", "<br/>")
                block.append(Paragraph(comment_html, styles["CommentText"]))
            code = d.body_snippet or d.signature
            block.append(Preformatted(code, styles["CodeBlock"]))
            story.append(KeepTogether(block))
    if truncated:
        story.append(Spacer(1, 6))
        story.append(Paragraph(
            f"(List truncated to {max_items} entries to keep the document size manageable.)",
            styles["CommentText"],
        ))


def render_markdown_section(story, styles, md_files: list[MarkdownFile]):
    story.append(Paragraph("4. .md files (in order)", styles["SectionTitle"]))
    if not md_files:
        story.append(Paragraph("No .md files in the project.", styles["Normal"]))
        return
    for mf in md_files:
        story.append(Paragraph(escape(mf.path), styles["MdHeading"]))
        content = mf.content
        if len(content) > 20000:
            content = content[:20000] + "\n\n... (truncated, file is very long)"
        story.append(Preformatted(content, styles["CodeBlock"]))


def _new_doc(path: str):
    return SimpleDocTemplate(
        path, pagesize=A4,
        leftMargin=18 * mm, rightMargin=18 * mm, topMargin=16 * mm, bottomMargin=16 * mm,
    )


def _summary_paragraph(styles, root, public_defs, internal_defs, md_files):
    block = [
        Paragraph("Project technical documentation", styles["Title"]),
        Paragraph(escape(f"Source directory: {os.path.abspath(root)}"), styles["Normal"]),
        Spacer(1, 4),
        Paragraph(
            f"Public API/ABI: {len(public_defs)} | Internal: {len(internal_defs)} | "
            f".md files: {len(md_files)}", styles["Normal"],
        ),
    ]
    return block


def build_pdf(output_path: str, root: str, public_defs, internal_defs, all_defs_for_fragments,
              md_files, max_items: int, split: bool):
    styles = build_styles()

    if not split:
        doc = _new_doc(output_path)
        story = []
        story += _summary_paragraph(styles, root, public_defs, internal_defs, md_files)
        story.append(PageBreak())
        render_signature_section(story, styles, "1. Public API / ABI", public_defs, max_items)
        story.append(PageBreak())
        render_signature_section(story, styles, "2. Internal functions", internal_defs, max_items)
        story.append(PageBreak())
        render_fragments_section(story, styles, all_defs_for_fragments, max_items)
        story.append(PageBreak())
        render_markdown_section(story, styles, md_files)
        doc.build(story)
        return [output_path]

    # --split mode: one PDF per section, so the document doesn't get gigantic
    base, ext = os.path.splitext(output_path)
    ext = ext or ".pdf"
    outputs = []

    summary_path = f"{base}.00-summary{ext}"
    doc = _new_doc(summary_path)
    story = _summary_paragraph(styles, root, public_defs, internal_defs, md_files)
    doc.build(story)
    outputs.append(summary_path)

    sections = [
        ("01-public-api", lambda s: render_signature_section(
            s, styles, "1. Public API / ABI", public_defs, max_items)),
        ("02-internal", lambda s: render_signature_section(
            s, styles, "2. Internal functions", internal_defs, max_items)),
        ("03-code-fragments", lambda s: render_fragments_section(
            s, styles, all_defs_for_fragments, max_items)),
        ("04-md-files", lambda s: render_markdown_section(s, styles, md_files)),
    ]
    for suffix, render_fn in sections:
        path = f"{base}.{suffix}{ext}"
        doc = _new_doc(path)
        story = []
        render_fn(story)
        doc.build(story)
        outputs.append(path)

    return outputs


# --------------------------------------------------------------------------- #
# main
# --------------------------------------------------------------------------- #

def main():
    parser = argparse.ArgumentParser(description="PDF documentation generator for a whole repo (no compilation).")
    parser.add_argument("root", help="Root directory of the project to scan")
    parser.add_argument("-o", "--output", default="documentation.pdf", help="Output PDF file")
    parser.add_argument("--max-body-lines", type=int, default=12,
                         help="How many lines of code to show in the fragments section (default: 12)")
    parser.add_argument("--max-items-per-section", type=int, default=4000,
                         help="Cap on entries per section (default: 4000)")
    parser.add_argument("--exclude", default="", help="Extra directories to skip, comma-separated")
    parser.add_argument("--ext", default="", help="Restrict to these extensions only, e.g.: .py,.js,.go")
    parser.add_argument("--split", action="store_true",
                         help="Instead of one PDF, generate a separate file per section "
                              "(recommended for large repos - several hundred+ source files)")
    args = parser.parse_args()

    exclude_dirs = set(DEFAULT_SKIP_DIRS)
    if args.exclude:
        exclude_dirs |= {d.strip() for d in args.exclude.split(",") if d.strip()}

    exts = None
    if args.ext:
        exts = {e.strip() if e.strip().startswith(".") else "." + e.strip()
                for e in args.ext.split(",") if e.strip()}

    if not os.path.isdir(args.root):
        print(f"Error: directory does not exist: {args.root}", file=sys.stderr)
        sys.exit(1)

    print("Scanning files...")
    all_defs, md_files = collect(args.root, exts, exclude_dirs, args.max_body_lines)

    public_defs = [d for d in all_defs if d.is_public]
    internal_defs = [d for d in all_defs if not d.is_public]

    print(f"Found: {len(public_defs)} public, {len(internal_defs)} internal, "
          f"{len(md_files)} .md files")
    print("Generating PDF...")

    outputs = build_pdf(args.output, args.root, public_defs, internal_defs, all_defs, md_files,
                         args.max_items_per_section, args.split)

    for p in outputs:
        print(f"Done: {p}")


if __name__ == "__main__":
    main()