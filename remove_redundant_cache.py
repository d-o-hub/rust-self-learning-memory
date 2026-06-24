import sys
import re

def cleanup(filename):
    with open(filename, 'r') as f:
        content = f.read()

    # Remove Swatinem/rust-cache block if it follows setup-rust
    # Note: This is a simple heuristic
    pattern = r'(\s+uses: \./\.github/actions/setup-rust\n(?:(?!\n\n).)*?\n)\s+- name: (?:Cache cargo registry|Configure rust-cache)\n\s+uses: Swatinem/rust-cache@[^\n]+(?:\n\s+with:\n(?:\s+[^\n]+\n)*)?'

    new_content = re.sub(pattern, r'\1', content, flags=re.DOTALL)

    if new_content != content:
        with open(filename, 'w') as f:
            f.write(new_content)
        print(f"Updated {filename}")

if __name__ == "__main__":
    for arg in sys.argv[1:]:
        cleanup(arg)
