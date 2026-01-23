#!/usr/bin/env python3
"""
Count unwrap/expect calls in Rust code, separating production from test code.

Usage:
    python3 scripts/count_unwrap_expect.py memory-core/src memory-storage-turso/src

Outputs:
    For each file: prod=X, test=Y
    Summary: TOTAL: Production=X, Test=Y
"""

import re
import sys
from pathlib import Path

def count_unwraps_in_file(file_path):
    """Count unwrap/expect calls, distinguishing production from test code."""
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return 0, 0
    
    in_test_module = False
    production_count = 0
    test_count = 0
    brace_count = 0
    
    for line_num, line in enumerate(lines, 1):
        # Check for test module boundaries
        if '#[cfg(test)]' in line or '#[test]' in line:
            in_test_module = True
        elif in_test_module:
            # Count braces to detect end of test module
            brace_count += line.count('{') - line.count('}')
            if brace_count < 0:
                in_test_module = False
                brace_count = 0
        
        # Count unwrap/expect (exclude comments)
        # Remove comments first
        code_line = line.split('//')[0].strip()
        
        if '.unwrap()' in code_line or '.expect(' in code_line:
            if in_test_module or 'test' in str(file_path):
                test_count += 1
            else:
                production_count += 1
                
    return production_count, test_count

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 count_unwrap_expect.py <dir1> [dir2] ...")
        sys.exit(1)
    
    total_prod = 0
    total_test = 0
    
    for crate_dir in sys.argv[1:]:
        crate_path = Path(crate_dir)
        if not crate_path.exists():
            print(f"Warning: {crate_dir} does not exist", file=sys.stderr)
            continue
        
        for rs_file in crate_path.rglob('*.rs'):
            prod, test = count_unwraps_in_file(rs_file)
            if prod > 0 or test > 0:
                print(f"{rs_file}: prod={prod}, test={test}")
                total_prod += prod
                total_test += test
    
    print(f"\nTOTAL: Production={total_prod}, Test={total_test}")
    
    # Check if under target
    if total_prod < 50:
        print(f"✅ UNDER TARGET: {total_prod} < 50")
    else:
        print(f"⚠️  EXCEEDS TARGET: {total_prod} >= 50")
    
    return 0 if total_prod < 50 else 1

if __name__ == '__main__':
    sys.exit(main())
