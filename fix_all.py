import os
import re

def edit_file(path, func):
    if not os.path.exists(path): return
    with open(path, 'r') as f:
        content = f.read()
    new_content = func(content)
    if new_content != content:
        with open(path, 'w') as f:
            f.write(new_content)
        print(f"Fixed {path}")

def fix_reward_score(content):
    # Match RewardScore { ... } and add missing fields
    pattern = r'RewardScore\s*\{([^}]*)\}'
    def repl(match):
        block = match.group(1)
        if 'effective_reward' in block:
            return match.group(0)
        # Find last field and append
        # Most have abstention_score or learning_bonus
        if 'abstention_score' in block:
            return f'RewardScore {{{block} raw_reward: 0.0, normalized_reward: 0.0, decayed_reward: 0.0, effective_reward: 0.0, }}'
        else:
            return f'RewardScore {{{block}, abstention_score: 0.0, raw_reward: 0.0, normalized_reward: 0.0, decayed_reward: 0.0, effective_reward: 0.0, }}'
    return re.sub(pattern, repl, content)

def fix_domain_stats(content):
    pattern = r'DomainStatistics\s*\{([^}]*)\}'
    def repl(match):
        block = match.group(1)
        if 'decay_half_life_days' in block:
            return match.group(0)
        return f'DomainStatistics {{{block} decay_half_life_days: 30.0, }}'
    return re.sub(pattern, repl, content)

files = [
    'memory-core/tests/common/assertions.rs',
    'memory-core/tests/capacity_property_tests.rs',
    'memory-core/tests/property_tests.rs',
    'memory-core/tests/reward_property_tests.rs',
    'memory-cli/tests/integration/test_fixtures.rs',
    'memory-cli/tests/unit/test_utils.rs',
    'memory-storage-redb/tests/serialization_property_tests.rs',
    'memory-storage-redb/tests/capacity_enforcement_test.rs',
    'memory-core/src/reward/external/merger.rs'
]

for f in files:
    edit_file(f, lambda c: fix_domain_stats(fix_reward_score(c)))
