#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out

# Two independent Bend runtimes execute concurrently on the host.
# They do not share Rust queues or Bend heaps.
./out/bench > out/instance-1.txt 2>&1 &
first=$!
./out/bench > out/instance-2.txt 2>&1 &
second=$!

status=0
wait "$first" || status=1
wait "$second" || status=1
cat out/instance-1.txt out/instance-2.txt

grep -q 'ADAPTIVE_RESULT events=16384 checksum=134225920 expected=134225920 remaining=0 producers_done=4' out/instance-1.txt || status=1
grep -q 'ADAPTIVE_RESULT events=16384 checksum=134225920 expected=134225920 remaining=0 producers_done=4' out/instance-2.txt || status=1
exit "$status"
