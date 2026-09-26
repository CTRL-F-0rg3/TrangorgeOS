#!/bin/bash
cd /tmp
cargo test --manifest-path /home/ctrl/TrangorgeOS/driverspace_workspace/Cargo.toml -p ds-fw-gpu > /tmp/e2.log 2>&1
echo "EXIT-$?" >> /tmp/e2.log
