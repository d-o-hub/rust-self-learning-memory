#!/usr/bin/env bash
# export-fixtures.sh — Convert recorded traces to test fixtures
# Usage: bash export-fixtures.sh [trace-dir] [output-dir]
set -euo pipefail

TRACE_DIR="${1:-.memory-traces}"
OUTPUT_DIR="${2:-memory-cli/tests/fixtures}"

mkdir -p "$OUTPUT_DIR"

python3 << PYEOF
import json, glob, os, sys

trace_dir = "$TRACE_DIR"
output_dir = "$OUTPUT_DIR"

traces = sorted(glob.glob(os.path.join(trace_dir, '*.json')))
traces = [t for t in traces if not t.endswith('.results.json')]

if not traces:
    print(f"No traces found in {trace_dir}/")
    print("Record agent sessions first using memory-harness skill.")
    sys.exit(1)

fixtures = []
for path in traces:
    with open(path) as f:
        trace = json.load(f)

    fixtures.append({
        'task': trace.get('task', 'Unknown task'),
        'domain': trace.get('domain', 'general'),
        'steps': [{
            'tool': s.get('tool', 'unknown'),
            'action': s.get('action', ''),
            'latency_ms': s.get('latency_ms', 0),
            'success': s.get('success', True),
            'observation': s.get('observation', ''),
        } for s in trace.get('steps', [])],
        'outcome': trace.get('outcome', 'success'),
        'project': trace.get('project', 'unknown'),
    })

output_path = os.path.join(output_dir, 'real_traces.json')
with open(output_path, 'w') as f:
    json.dump(fixtures, f, indent=2)

# Also generate a Rust-friendly fixture summary
summary_path = os.path.join(output_dir, 'README.md')
with open(summary_path, 'w') as f:
    f.write("# Test Fixtures from Real Agent Sessions\n\n")
    f.write(f"Generated from {len(fixtures)} recorded traces.\n\n")
    f.write("| # | Task | Domain | Steps | Outcome |\n")
    f.write("|---|------|--------|-------|---------|\n")
    for i, fix in enumerate(fixtures):
        task_short = fix['task'][:50] + ('...' if len(fix['task']) > 50 else '')
        f.write(f"| {i+1} | {task_short} | {fix['domain']} | {len(fix['steps'])} | {fix['outcome']} |\n")
    f.write(f"\nTotal steps across all traces: {sum(len(f['steps']) for f in fixtures)}\n")

print(f"Exported {len(fixtures)} traces to {output_path}")
print(f"Summary written to {summary_path}")
PYEOF
