"""Sample usage without API keys - uses free tools only."""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "scripts"))

from resolve import resolve, resolve_url, resolve_query


def main():
    print("=" * 60)
    print("Web Doc Resolver - Basic Usage (Free Tools)")
    print("=" * 60)
    
    # Example 1: Resolve a URL
    print("\n--- Example 1: URL Resolution ---")
    result = resolve("https://docs.python.org/3/")
    print(f"Source: {result['source']}")
    print(f"Content (first 500 chars):\n{result['content'][:500]}...")
    
    # Example 2: Resolve a query
    print("\n--- Example 2: Query Resolution ---")
    result = resolve("Rust async programming best practices")
    print(f"Source: {result['source']}")
    print(f"Content (first 500 chars):\n{result['content'][:500]}...")
    
    # Example 3: Direct URL resolution
    print("\n--- Example 3: Direct URL ---")
    result = resolve_url("https://tokio.rs/tokio/tutorial")
    print(f"Source: {result['source']}")
    print(f"Content (first 300 chars):\n{result['content'][:300]}...")
    
    # Example 4: Direct query resolution
    print("\n--- Example 4: Direct Query ---")
    result = resolve_query("OpenAI API best practices 2026")
    print(f"Source: {result['source']}")
    print(f"Content (first 300 chars):\n{result['content'][:300]}...")


if __name__ == "__main__":
    main()
