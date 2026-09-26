#!/bin/bash
F=/home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-gpu/src/edid.rs
{
  echo "=== outline (items at column 0) ==="
  grep -n '^[a-z#/}]' $F
  echo "=== brace balance ==="
  echo "open:  $(grep -o '{' $F | wc -l)"
  echo "close: $(grep -o '}' $F | wc -l)"
} > /tmp/outline.log 2>&1
