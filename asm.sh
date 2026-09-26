#!/bin/bash
F=/home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-gpu/src/edid.rs
cat /tmp/p1.rs /tmp/p2.rs /tmp/p3.rs /tmp/p4.rs /tmp/p5.rs > $F
{
  echo ''
  echo '#[cfg(test)]'
  cat /tmp/edid_tests.rs /tmp/edid_tests2.rs /tmp/edid_tests3.rs
} >> $F
{
  echo "=== assembled ==="
  wc -l $F
  echo "open:  $(grep -o '{' $F | wc -l)"
  echo "close: $(grep -o '}' $F | wc -l)"
  cd /tmp
  cargo test --manifest-path /home/ctrl/TrangorgeOS/driverspace_workspace/Cargo.toml -p ds-fw-gpu > /tmp/e3.log 2>&1
  echo "EXIT-$?" >> /tmp/e3.log
} > /tmp/asm.log 2>&1
