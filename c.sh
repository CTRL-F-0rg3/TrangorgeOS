#!/bin/bash
cd /home/ctrl/TrangorgeOS
{
  echo "=== edid.c ==="
  cat kernel_Workspace/kernel/src/displayport/edid.c
  echo "=== link.c ==="
  cat kernel_Workspace/kernel/src/displayport/link.c
  echo "=== hdmi kernel files ==="
  find kernel_Workspace/kernel/src/hdmi -type f 2>/dev/null | sort
  echo "=== hdmi mode.h ==="
  cat kernel_Workspace/kernel/src/hdmi/mode.h 2>/dev/null
} > /tmp/c.log 2>&1
echo DONE >> /tmp/c.log
