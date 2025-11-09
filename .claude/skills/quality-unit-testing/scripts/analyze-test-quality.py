#!/usr/bin/env python3
"""
Analyze Rust test file quality based on best practices.
Usage: python analyze-test-quality.py <test_file_path>
"""

import re
import sys
from pathlib import Path
from typing import List, Dict, Tuple

class RustTestQualityAnalyzer:
    def __init__(self, file_path: str):
        self.file_path = Path(file_path)
        self.content = self.file_path.read_text()
        self.lines = self.content.split('\n')
        self.issues = []
        self.score = 100

    def analyze(self) -> Dict:
        """Run all quality checks"""
        self.check_naming_convention()
        self.check_aaa_pattern()
        self.check_test_focus()
        self.check_for_anti_patterns()
        self.check_async_patterns()

        return {
            'file': str(self.file_path),
            'score': max(0, self.score),
            'issues': self.issues,
            'summary': self.generate_summary()
        }

    def check_naming_convention(self):
        """Check if test names follow test_<function>_<scenario>_<expected>"""
        test_pattern = r'fn\s+(test_\w+)'

        for i, line in enumerate(self.lines, 1):
            match = re.search(test_pattern, line)
            if match:
                test_name = match.group(1)

                # Check for underscores (good pattern)
                parts = test_name.split('_')[1:]  # Skip 'test' prefix
                if len(parts) < 3:
                    self.add_issue(
                        'naming',
                        f"Line {i}: Test name '{test_name}' doesn't follow " +
                        "test_<function>_<scenario>_<expected> pattern",
                        severity='medium'
                    )

                # Check for vague names
                vague_names = ['test1', 'test2', 'test_method', 'test_function']
                if test_name in vague_names:
                    self.add_issue(
                        'naming',
                        f"Line {i}: Vague test name '{test_name}'",
                        severity='high'
                    )

    def check_aaa_pattern(self):
        """Check for clear AAA (Arrange-Act-Assert) separation"""
        aaa_comments = ['arrange', 'act', 'assert']

        in_test = False
        test_start = 0
        aaa_found = {comment: False for comment in aaa_comments}

        for i, line in enumerate(self.lines):
            lower_line = line.lower()

            # Detect test start
            if re.search(r'#\[(tokio::)?test\]', line):
                in_test = True
                test_start = i
                aaa_found = {comment: False for comment in aaa_comments}

            # Detect test end
            if in_test and line.strip().startswith('}'):
                # Check if AAA comments were found
                missing = [c for c, found in aaa_found.items() if not found]
                if len(missing) == 3:  # No AAA comments at all
                    self.add_issue(
                        'structure',
                        f"Line {test_start + 1}: No AAA pattern comments found",
                        severity='low'  # Low because simple tests might not need it
                    )
                in_test = False

            # Check for AAA comments
            if in_test:
                for comment in aaa_comments:
                    if comment in lower_line and '//' in line:
                        aaa_found[comment] = True

    def check_test_focus(self):
        """Check if tests have single responsibility"""
        in_test = False
        assert_count = 0
        test_start = 0

        for i, line in enumerate(self.lines, 1):
            if re.search(r'#\[(tokio::)?test\]', line):
                in_test = True
                test_start = i
                assert_count = 0

            if in_test:
                # Count assertions
                if any(keyword in line for keyword in
                       ['assert!', 'assert_eq!', 'assert_ne!', 'assert_matches!']):
                    assert_count += 1

                # Detect test end
                if line.strip().startswith('}'):
                    if assert_count > 5:
                        self.add_issue(
                            'focus',
                            f"Line {test_start}: Test has {assert_count} assertions. " +
                            "Consider splitting into multiple tests",
                            severity='medium'
                        )
                    in_test = False

    def check_for_anti_patterns(self):
        """Check for common anti-patterns"""
        anti_patterns = [
            (r'thread::sleep\(', 'Uses thread::sleep() - potential flaky test'),
            (r'std::thread::sleep', 'Uses std::thread::sleep - potential flaky test'),
            (r'\.unwrap\(\).*\.unwrap\(\)', 'Multiple unwraps - use ? with Result<()>'),
            (r'#\[ignore\](?!\s*=)', 'Contains ignored test without reason'),
            (r'todo!\(', 'Contains todo!() - incomplete test'),
            (r'unimplemented!\(', 'Contains unimplemented!() - incomplete test'),
        ]

        for i, line in enumerate(self.lines, 1):
            for pattern, message in anti_patterns:
                if re.search(pattern, line):
                    self.add_issue(
                        'anti-pattern',
                        f"Line {i}: {message}",
                        severity='high' if 'sleep' in pattern or 'unwrap' in pattern else 'low'
                    )

    def check_async_patterns(self):
        """Check for async-specific issues"""
        in_async_test = False
        test_start = 0

        for i, line in enumerate(self.lines, 1):
            # Detect async test
            if '#[tokio::test]' in line:
                in_async_test = True
                test_start = i

            # Check for missing .await in async tests
            if in_async_test:
                # Look for async function calls without .await
                if re.search(r'\w+\(.*\);(?!.*\.await)', line) and 'async' in line:
                    # This is a heuristic - might have false positives
                    pass  # Too many false positives, skip for now

                # Reset at test end
                if line.strip().startswith('}'):
                    in_async_test = False

            # Check for async test without #[tokio::test]
            if 'async fn test_' in line and '#[test]' in self.lines[max(0, i-2)]:
                self.add_issue(
                    'async',
                    f"Line {i}: Async test without #[tokio::test] attribute",
                    severity='high'
                )

    def add_issue(self, category: str, message: str, severity: str):
        """Add an issue and adjust score"""
        self.issues.append({
            'category': category,
            'message': message,
            'severity': severity
        })

        # Adjust score based on severity
        if severity == 'high':
            self.score -= 10
        elif severity == 'medium':
            self.score -= 5
        else:
            self.score -= 2

    def generate_summary(self) -> str:
        """Generate summary report"""
        if self.score >= 90:
            grade = 'A (Excellent)'
        elif self.score >= 80:
            grade = 'B (Good)'
        elif self.score >= 70:
            grade = 'C (Acceptable)'
        elif self.score >= 60:
            grade = 'D (Needs Improvement)'
        else:
            grade = 'F (Poor)'

        return f"Quality Score: {self.score}/100 ({grade})"

def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze-test-quality.py <test_file_path>")
        sys.exit(1)

    file_path = sys.argv[1]

    if not Path(file_path).exists():
        print(f"Error: File '{file_path}' not found")
        sys.exit(1)

    analyzer = RustTestQualityAnalyzer(file_path)
    results = analyzer.analyze()

    print(f"\n{'='*60}")
    print(f"Rust Test Quality Analysis: {results['file']}")
    print(f"{'='*60}\n")

    print(results['summary'])
    print()

    if results['issues']:
        print(f"Found {len(results['issues'])} issues:\n")

        # Group by category
        by_category = {}
        for issue in results['issues']:
            cat = issue['category']
            if cat not in by_category:
                by_category[cat] = []
            by_category[cat].append(issue)

        for category, issues in by_category.items():
            print(f"\n{category.upper()}:")
            for issue in issues:
                severity_marker = {
                    'high': '❌',
                    'medium': '⚠️',
                    'low': 'ℹ️'
                }[issue['severity']]
                print(f"  {severity_marker} {issue['message']}")
    else:
        print("✅ No issues found!")

    print()

if __name__ == '__main__':
    main()
