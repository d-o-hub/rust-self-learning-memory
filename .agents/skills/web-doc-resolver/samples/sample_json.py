"""Sample usage with JSON output."""

import json
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "scripts"))

from resolve import resolve


def main():
    print("=" * 60)
    print("Web Doc Resolver - JSON Output")
    print("=" * 60)
    
    # Example 1: URL as JSON
    print("\n--- Example 1: URL Resolution (JSON) ---")
    result = resolve("https://docs.rust-lang.org/book/")
    print(json.dumps(result, indent=2))
    
    # Example 2: Query as JSON
    print("\n--- Example 2: Query Resolution (JSON) ---")
    result = resolve("Tokio runtime configuration")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
