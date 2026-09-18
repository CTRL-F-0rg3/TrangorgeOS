#!/usr/bin/env python3
"""
TrangorgeOS Professional Documentation Generator
Handles massive, multi-language codebases with hierarchical grouping, 
full documentation blocks, and complete markdown inclusion.
"""

import os
import re
from pathlib import Path
from collections import defaultdict
from datetime import datetime

class ProDocGenerator:
    def __init__(self, root_dir):
        self.root_dir = Path(root_dir).resolve()
        self.subsystems = defaultdict(lambda: defaultdict(list))
        self.markdown_files = []
        self.stats = {'total_files': 0, 'total_items': 0}
        
        self.ignore_dirs = {'.git', 'target', '.venv', 'build', 'node_modules', '__pycache__', 'out'}
        
        self.parsers = {
            '.rs': self._parse_rust,
            '.c': self._parse_c,
            '.h': self._parse_c,
            '.ads': self._parse_ada,
            '.adb': self._parse_ada,
            '.odin': self._parse_odin,
            '.nim': self._parse_nim,
            '.md': self._parse_markdown
        }

    def _get_subsystem(self, file_path: Path) -> str:
        try:
            rel_path = file_path.relative_to(self.root_dir)
            parts = list(rel_path.parts)
            if len(parts) > 2:
                return f"{parts[0]}/{parts[1]}"
            return parts[0] if parts else "root"
        except ValueError:
            return "unknown"

    def _parse_rust(self, file_path: Path, lines: list):
        docs = []
        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('///') or stripped.startswith('//!'):
                docs.append(stripped[3:].strip())
            else:
                match = re.search(r'^\s*(?:pub\s+)?(fn|struct|enum|trait|type|impl)\s+([A-Za-z0-9_]+)', stripped)
                if match:
                    kind, name = match.groups()
                    if kind != 'impl' or name != '':
                        self.subsystems[self._get_subsystem(file_path)][str(file_path)].append({
                            'lang': 'rust', 'kind': kind, 'name': name,
                            'signature': stripped[:200], 
                            'docs': '\n'.join(docs), 
                            'line': i + 1
                        })
                        self.stats['total_items'] += 1
                docs = []

    def _parse_c(self, file_path: Path, lines: list):
        docs = []
        in_block_comment = False
        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('/*'):
                in_block_comment = True
                docs.append(stripped[2:].strip())
            elif stripped.endswith('*/'):
                in_block_comment = False
                docs.append(stripped[:-2].strip().lstrip('*'))
            elif in_block_comment:
                docs.append(stripped.lstrip('*').strip())
            elif stripped.startswith('//'):
                docs.append(stripped[2:].strip())
            else:
                if re.match(r'^(?:static\s+|inline\s+|extern\s+)?[A-Za-z_][A-Za-z0-9_\s\*]*\s+[A-Za-z_][A-Za-z0-9_]*\s*\([^;]*\)\s*\{?', stripped):
                    match = re.search(r'([A-Za-z_][A-Za-z0-9_]*)\s*\(', stripped)
                    if match and not stripped.startswith('#') and not stripped.startswith('if') and not stripped.startswith('for') and not stripped.startswith('while'):
                        name = match.group(1)
                        self.subsystems[self._get_subsystem(file_path)][str(file_path)].append({
                            'lang': 'c', 'kind': 'function', 'name': name,
                            'signature': stripped[:200], 
                            'docs': '\n'.join(docs), 
                            'line': i + 1
                        })
                        self.stats['total_items'] += 1
                docs = []

    def _parse_ada(self, file_path: Path, lines: list):
        docs = []
        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('--'):
                docs.append(stripped[2:].strip())
            else:
                match = re.search(r'^\s*(procedure|function|package|task|type)\s+([A-Za-z0-9_]+)', stripped, re.IGNORECASE)
                if match:
                    kind, name = match.groups()
                    self.subsystems[self._get_subsystem(file_path)][str(file_path)].append({
                        'lang': 'ada', 'kind': kind.lower(), 'name': name,
                        'signature': stripped[:200], 
                        'docs': '\n'.join(docs), 
                        'line': i + 1
                    })
                    self.stats['total_items'] += 1
                docs = []

    def _parse_odin(self, file_path: Path, lines: list):
        docs = []
        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('//'):
                docs.append(stripped[2:].strip())
            else:
                match = re.search(r'^\s*([A-Za-z0-9_]+)\s*::\s*(proc|struct|enum|union|distinct)', stripped)
                if match:
                    name, kind = match.groups()
                    self.subsystems[self._get_subsystem(file_path)][str(file_path)].append({
                        'lang': 'odin', 'kind': kind, 'name': name,
                        'signature': stripped[:200], 
                        'docs': '\n'.join(docs), 
                        'line': i + 1
                    })
                    self.stats['total_items'] += 1
                docs = []

    def _parse_nim(self, file_path: Path, lines: list):
        docs = []
        for i, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith('##'):
                docs.append(stripped[2:].strip())
            else:
                match = re.search(r'^\s*(proc|type|const|var|let|template|macro)\s+([A-Za-z0-9_`]+)', stripped)
                if match:
                    kind, name = match.groups()
                    self.subsystems[self._get_subsystem(file_path)][str(file_path)].append({
                        'lang': 'nim', 'kind': kind, 'name': name,
                        'signature': stripped[:200], 
                        'docs': '\n'.join(docs), 
                        'line': i + 1
                    })
                    self.stats['total_items'] += 1
                docs = []

    def _parse_markdown(self, file_path: Path, content: str):
        # Zbieramy pełną zawartość plików markdown bez arbitralnego ucinania
        self.markdown_files.append({'path': str(file_path), 'content': content})

    def scan(self):
        print(f"Scanning {self.root_dir} (ignoring build artifacts)...")
        for root, dirs, files in os.walk(self.root_dir):
            dirs[:] = [d for d in dirs if d not in self.ignore_dirs]
            
            for file in files:
                file_path = Path(root) / file
                ext = file_path.suffix
                
                if ext in self.parsers:
                    try:
                        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                            lines = f.readlines()
                            self.stats['total_files'] += 1
                            self.parsers[ext](file_path, lines)
                    except Exception as e:
                        pass

    def generate_markdown(self, output_file: str):
        print(f"Generating structured Markdown: {output_file}")
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(f"# TrangorgeOS Architecture Documentation\n\n")
            f.write(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"**Files Scanned:** `{self.stats['total_files']}` | **API Items Extracted:** `{self.stats['total_items']}`\n")
            f.write(f"**Languages:** Rust, C/C++, Ada SPARK, Odin, Nim, Assembly\n\n")
            f.write("---\n\n")
            
            f.write("## Table of Contents\n\n")
            for subsystem in sorted(self.subsystems.keys()):
                anchor = subsystem.replace('/', '-').replace('_', '-').lower()
                f.write(f"- [{subsystem}](#{anchor})\n")
            f.write("\n---\n\n")

            for subsystem in sorted(self.subsystems.keys()):
                f.write(f"## {subsystem}\n\n")
                
                for file_path in sorted(self.subsystems[subsystem].keys()):
                    rel_file = Path(file_path).relative_to(self.root_dir)
                    item_count = len(self.subsystems[subsystem][file_path])
                    f.write(f"<details>\n<summary><b>{rel_file}</b> ({item_count} items)</summary>\n\n")
                    
                    for item in self.subsystems[subsystem][file_path]:
                        f.write(f"#### `{item['kind'].upper()}`: **{item['name']}** <sub>line {item['line']}</sub>\n")
                        f.write(f"```{item['lang']}\n{item['signature']}\n```\n")
                        if item['docs']:
                            f.write(f"> {item['docs'].replace('\n', ' \n> ')}\n\n")
                        else:
                            f.write("\n")
                    f.write("</details>\n\n")
            
            if self.markdown_files:
                f.write("## Existing Markdown Documentation\n\n")
                for md in self.markdown_files:
                    rel_md = Path(md['path']).relative_to(self.root_dir)
                    f.write(f"<details>\n<summary><b>{rel_md}</b></summary>\n\n")
                    f.write(f"{md['content']}\n\n</details>\n\n")
                    
        print("Markdown generation complete.")

if __name__ == '__main__':
    import sys
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    out_md = sys.argv[2] if len(sys.argv) > 2 else "trangorge_docs.md"
    
    gen = ProDocGenerator(root)
    gen.scan()
    gen.generate_markdown(out_md)
    print(f"\nTIP: Convert to PDF using Pandoc for best results:")
    print(f"   pandoc {out_md} -o trangorge_docs.pdf --pdf-engine=xelatex -V geometry:margin=1.5cm -V fontsize=10pt")