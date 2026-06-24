import sys
import re

def cleanup(filename):
    with open(filename, 'r') as f:
        content = f.read()

    # Replace the specific double 'with' pattern
    # uses: ./.github/actions/setup-rust
    # with:
    #   job-name: ...
    # with:
    #   components: ...

    new_content = re.sub(
        r'(uses: \./\.github/actions/setup-rust\n\s+with:\n\s+job-name: [^\n]+)\n\s+with:',
        r'\1',
        content
    )

    with open(filename, 'w') as f:
        f.write(new_content)

if __name__ == "__main__":
    for arg in sys.argv[1:]:
        cleanup(arg)
