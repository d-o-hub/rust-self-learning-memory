#!/usr/bin/env python3
"""
measure-current.sh - Baseline performance measurement for build-compile

Measures token count, timing, and memory for current build-compile implementation.
This establishes BEFORE metrics to justify optimization decisions.
"""

import subprocess
import time
import json
import os
import sys
from pathlib import Path

try:
    import tiktoken
except ImportError:
    print("❌ Installing tiktoken...")
    subprocess.run([sys.executable, "-m", "pip", "install", "tiktoken", "-q"])
    import tiktoken

def count_tokens(file_path: str, model: str = "gpt-4") -> int:
    """Count tokens in a file using tiktoken."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        encoder = tiktoken.encoding_for_model(model)
        tokens = encoder.encode(content)
        return len(tokens), len(content)
    except Exception as e:
        print(f"⚠️  Error counting tokens in {file_path}: {e}")
        return 0, 0

def measure_build_performance() -> dict:
    """Measure actual build timing and resource usage."""
    results = {}
    
    # Test check mode (fastest)
    print("\n📊 Measuring: cargo check --all")
    start = time.time()
    try:
        result = subprocess.run(
            ["cargo", "check", "--all"],
            capture_output=True,
            text=True,
            timeout=120
        )
        elapsed = time.time() - start
        results['check_time'] = elapsed
        results['check_success'] = result.returncode == 0
    except subprocess.TimeoutExpired:
        results['check_time'] = 120.0
        results['check_success'] = False
    except Exception as e:
        results['check_time'] = -1
        results['check_success'] = False
    
    # Test dev build (medium)
    print("\n📊 Measuring: cargo build --workspace")
    start = time.time()
    try:
        result = subprocess.run(
            ["cargo", "build", "--workspace"],
            capture_output=True,
            text=True,
            timeout=300
        )
        elapsed = time.time() - start
        results['dev_time'] = elapsed
        results['dev_success'] = result.returncode == 0
    except subprocess.TimeoutExpired:
        results['dev_time'] = 300.0
        results['dev_success'] = False
    except Exception as e:
        results['dev_time'] = -1
        results['dev_success'] = False
    
    # Count target size
    try:
        result = subprocess.run(
            ["du", "-sb", "target/"],
            capture_output=True,
            text=True
        )
        size_mb = float(result.stdout.strip()) / (1024 * 1024)
        results['target_size_mb'] = round(size_mb, 2)
    except:
        results['target_size_mb'] = 0
    
    return results

def main():
    print("🔍 Phase 1: Baseline Performance Measurement")
    print("=" * 60)
    
    project_root = Path.cwd()
    agent_file = project_root / ".opencode" / "agent" / "build-compile.md"
    skill_file = project_root / ".opencode" / "skill" / "build-rust" / "SKILL.md"
    
    if not agent_file.exists():
        print(f"⚠️  Agent file not found: {agent_file}")
        print("Using current state as baseline...")
    
    print(f"\n📂 Project Root: {project_root}")
    
    # Measure token counts
    results = {}
    total_tokens = 0
    total_chars = 0
    
    files_to_measure = [
        ("Agent", agent_file),
        ("Skill", skill_file),
    ]
    
    for name, path in files_to_measure:
        if path.exists():
            print(f"\n📄 {name}: {path.name}")
            tokens, chars = count_tokens(str(path))
            print(f"   Tokens: {tokens:,}")
            print(f"   Characters: {chars:,}")
            print(f"   Token/Char: {tokens/chars:.3f}")
            results[f"{name.lower()}_tokens"] = tokens
            results[f"{name.lower()}_chars"] = chars
            total_tokens += tokens
            total_chars += chars
        else:
            print(f"\n⚠️  {name}: {path.name} (not found)")
    
    results['total_tokens'] = total_tokens
    results['total_chars'] = total_chars
    
    # Measure build performance
    build_results = measure_build_performance()
    results.update(build_results)
    
    # Calculate metrics
    print("\n" + "=" * 60)
    print("📊 BASELINE METRICS")
    print("=" * 60)
    
    print(f"\n📝 Token Count:")
    print(f"   Total: {total_tokens:,} tokens")
    print(f"   Agent: {results.get('agent_tokens', 0):,} tokens")
    print(f"   Skill: {results.get('skill_tokens', 0):,} tokens")
    
    print(f"\n⏱️  Build Performance:")
    print(f"   Check: {results.get('check_time', 0):.1f}s")
    print(f"   Dev: {results.get('dev_time', 0):.1f}s")
    print(f"   Success: {'✅' if results.get('check_success') and results.get('dev_success') else '❌'}")
    
    print(f"\n💾 Target Size:")
    print(f"   {results.get('target_size_mb', 0):.1f} MB")
    
    # Save results
    output_file = project_root / "metrics" / "baseline.json"
    output_file.parent.mkdir(exist_ok=True)
    
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n💾 Results saved: {output_file}")
    
    # Calculate optimization decision criteria
    print("\n" + "=" * 60)
    print("🎯 OPTIMIZATION DECISION CRITERIA")
    print("=" * 60)
    
    print("\nProceed to Phase 2 (Agent+Skill split) IF:")
    print("  ✓ Token reduction > 50%")
    print("  ✓ OR Timing improvement > 20%")
    print("  ✓ AND Implementation cost < 4 hours")
    
    print("\nCurrent status:")
    print(f"  - Baseline established: {total_tokens:,} tokens")
    print(f"  - Next step: Implement optimization")
    print(f"  - Measure: {total_tokens * 0.5:,.0f} tokens (50% target)")
    
    return results

if __name__ == "__main__":
    main()
