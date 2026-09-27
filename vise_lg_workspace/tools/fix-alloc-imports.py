#!/usr/bin/env python3
"""Add the missing `alloc` imports to the no_std sources.

`use alloc::vec::Vec` imports the *type* `Vec`. The `vec!` macro is a different
item and needs `use alloc::vec;`. `Box` is likewise two names in practice — the
type `Box<T>` and the constructor `Box::new(..)` — and a file can use one without
ever writing the other. Both forms are matched, because an import added for
`Box<T>` leaves `Box::new` undeclared and vice versa.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
# (regex that means "this file uses it", the `use` line it needs)
WANTED = [
    (r"\bBox\s*::|\bBox<", "use alloc::boxed::Box;"),
    (r"\bvec!\[", "use alloc::vec;"),
    (r"\bformat!\[", "use alloc::format;"),
    (r"\bString::", "use alloc::string::String;"),
    (r"\bToString\b", "use alloc::string::ToString;"),
    (r"\bVec<", "use alloc::vec::Vec;"),
]

fixed = []
skipped = 0

for path in sorted(ROOT.rglob("src/**/*.rs")):
    text = path.read_text(encoding="utf-8")
    if "#![no_std]" not in text:
        skipped += 1
        continue

    needed = [line for pattern, line in WANTED
              if re.search(pattern, text) and line not in text]
    if not needed:
        continue

    lines = text.split("\n")
    # Insert after the last existing `use alloc::` so the imports stay grouped;
    # fall back to just after `extern crate alloc` when there is none yet.
    anchors = [i for i, l in enumerate(lines) if l.startswith("use alloc::")]
    if not anchors:
        externs = [i for i, l in enumerate(lines) if l.startswith("extern crate alloc")]
        at = externs[0] if externs else 0
    else:
        at = anchors[-1]
    lines[at + 1 : at + 1] = needed
    path.write_text("\n".join(lines), encoding="utf-8")
    fixed.append(f"{path.relative_to(ROOT)}: {', '.join(needed)}")

print("\n".join(fixed) if fixed else "(nothing to fix)")
print(f"scanned {len(list(ROOT.rglob('src/**/*.rs')))} files, {skipped} with std",
      file=sys.stderr)
