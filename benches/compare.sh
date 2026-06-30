#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
# SPDX-License-Identifier: MIT
#
# Reproducible head-to-head benchmark: twc-rs vs the official Python twc-cli.
#
# Usage:
#   cargo build --release --features tui
#   python3 -m venv /tmp/twcbench && /tmp/twcbench/bin/pip install twc-cli
#   benches/compare.sh ./target/release/twc-rs /tmp/twcbench/bin/twc
#
# Measures cold-start latency (mean over N runs), peak RSS and on-disk size.
# All numbers are produced live on the host running the script.
set -euo pipefail

OURS="${1:-./target/release/twc-rs}"
OFFICIAL="${2:-}"
RUNS="${RUNS:-50}"

bench() { # bin args... -> mean ms/run
  local bin="$1"; shift
  "$bin" "$@" >/dev/null 2>&1 || true   # warm caches
  local start end
  start=$(date +%s.%N)
  for _ in $(seq 1 "$RUNS"); do "$bin" "$@" >/dev/null 2>&1 || true; done
  end=$(date +%s.%N)
  awk -v s="$start" -v e="$end" -v n="$RUNS" 'BEGIN{printf "%.1f", (e-s)/n*1000}'
}

rss() { # bin args... -> peak RSS KB (getrusage RUSAGE_CHILDREN)
  python3 - "$@" <<'PY'
import subprocess, resource, sys
subprocess.run(sys.argv[1:], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
print(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)
PY
}

echo "host: $(uname -srm)"
echo "runs: $RUNS"
echo
printf '%-22s %12s %12s\n' "metric" "twc-rs" "official"
printf '%-22s %12s %12s\n' "start --version (ms)" "$(bench "$OURS" --version)" "${OFFICIAL:+$(bench "$OFFICIAL" --version)}"
printf '%-22s %12s %12s\n' "start --help (ms)" "$(bench "$OURS" --help)" "${OFFICIAL:+$(bench "$OFFICIAL" --help)}"
printf '%-22s %12s %12s\n' "peak RSS (KB)" "$(rss "$OURS" --version)" "${OFFICIAL:+$(rss "$OFFICIAL" --version)}"
printf '%-22s %12s\n' "binary size" "$(du -h "$OURS" | cut -f1)"
