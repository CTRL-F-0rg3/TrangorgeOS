#!/bin/bash
# Survey the fs tree: what exists, what is declared, what is dead.
ROOT=/home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src
{
  echo "=== fs/ tree with line counts ==="
  find $ROOT/fs -name '*.rs' | sort | xargs wc -l

  echo
  echo "=== declared in fs/mod.rs ==="
  grep -n '^pub mod' $ROOT/fs/mod.rs

  echo
  echo "=== .rs files in fs/ NOT declared anywhere ==="
  for f in $(find $ROOT/fs -name '*.rs' | sort); do
    base=$(basename $f .rs)
    parent=$(basename $(dirname $f))
    if [ "$parent" = "fs" ]; then
      grep -q "mod $base" $ROOT/fs/mod.rs || echo "  UNDECLARED: $f"
    else
      grep -q "mod $base" $ROOT/fs/$parent/mod.rs 2>/dev/null || echo "  UNDECLARED: $f"
    fi
  done
} > /tmp/tree.log 2>&1
