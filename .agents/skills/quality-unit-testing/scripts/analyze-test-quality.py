#!/usr/bin/env python3
"""
Analyze Rust test file quality based on best practices.
Usage: python analyze-test-quality.py <test_file_path> [options]
"""

import re
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass, field

try:
    import tomli
    TOML_AVAILABLE = True
except ImportError:
    TOML_AVAILABLE = False


@dataclass
class AnalysisConfig:
    """Configuration for test quality analysis"""
    # Naming
    require_three_parts: bool = True
    min_parts: int = 3
    allow_two_part_simple: bool = False

    # Structure
    require_aaa_comments: str = "complex"  # "always", "complex", "never"
    complex_test_threshold: int = 10

    # Focus
    max_assertions: int = 5
    max_assertions_error: int = 10

    # Severity levels
    naming_violation: str = "medium"
    missing_aaa_simple: str = "low"
    missing_aaa_complex: str = "medium"
    too_many_assertions: str = "medium"

    @classmethod
    def from_toml(cls, config_path: Path) -> 'AnalysisConfig':
        """Load configuration from TOML file"""
        if not TOML_AVAILABLE:
            print("Warning: tomli not installed, using default config")
            return cls()

        if not config_path.exists():
            return cls()

        with open(config_path, 'rb') as f:
            data = tomli.load(f)

        config = cls()

        # Naming
        if 'naming' in data:
            config.require_three_parts = data['naming'].get('require_three_parts', True)
            config.min_parts = data['naming'].get('min_parts', 3)
            config.allow_two_part_simple = data['naming'].get('allow_two_part_simple', False)

        # Structure
        if 'structure' in data:
            config.require_aaa_comments = data['structure'].get('require_aaa_comments', 'complex')
            config.complex_test_threshold = data['structure'].get('complex_test_threshold', 10)

        # Focus
        if 'focus' in data:
            config.max_assertions = data['focus'].get('max_assertions', 5)
            config.max_assertions_error = data['focus'].get('max_assertions_error', 10)

        # Severity
        if 'severity' in data:
            config.naming_violation = data['severity'].get('naming_violation', 'medium')
            config.missing_aaa_simple = data['severity'].get('missing_aaa_simple', 'low')
            config.missing_aaa_complex = data['severity'].get('missing_aaa_complex', 'medium')
            config.too_many_assertions = data['severity'].get('too_many_assertions', 'medium')

        return config


class RustTestQualityAnalyzer:
    def __init__(self, file_path: str, config: Optional[AnalysisConfig] = None):
        self.file_path = Path(file_path)
        self.content = self.file_path.read_text()
        self.lines = self.content.split('\n')
        self.issues = []
        self.score = 100
        self.config = config or AnalysisConfig()

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

    def get_test_info(self, start_line: int) -> Tuple[int, bool]:
        """Get test line count and whether it's async"""
        line_count = 0
        is_async = False
        in_test = False

        for i in range(start_line, len(self.lines)):
            line = self.lines[i]

            if 'fn test_' in line or 'async fn test_' in line:
                in_test = True
                is_async = 'async' in line

            if in_test:
                line_count += 1
                if line.strip().startswith('}') and i > start_line:
                    break

        return line_count, is_async

    def is_test_complex(self, start_line: int) -> bool:
        """Determine if test is complex based on line count and structure"""
        line_count, is_async = self.get_test_info(start_line)
        return line_count > self.config.complex_test_threshold or is_async

    def check_naming_convention(self):
        """Check if test names follow test_<function>_<scenario>_<expected>"""
        test_pattern = r'fn\s+(test_\w+)'

        for i, line in enumerate(self.lines, 1):
            match = re.search(test_pattern, line)
            if match:
                test_name = match.group(1)

                # Check for underscores (good pattern)
                parts = test_name.split('_')[1:]  # Skip 'test' prefix

                # Check if test is simple
                is_simple = not self.is_test_complex(i - 1)

                # Allow two-part names for simple tests if configured
                min_required = 2 if (is_simple and self.config.allow_two_part_simple) else self.config.min_parts

                if len(parts) < min_required:
                    self.add_issue(
                        'naming',
                        f"Line {i}: Test name '{test_name}' doesn't follow " +
                        f"test_<function>_<scenario>_<expected> pattern ({len(parts)} parts, need {min_required})",
                        severity=self.config.naming_violation
                    )

                # Check for vague names
                vague_names = ['test1', 'test2', 'test_method', 'test_function', 'test_basic']
                if test_name in vague_names:
                    self.add_issue(
                        'naming',
                        f"Line {i}: Vague test name '{test_name}'",
                        severity='high'
                    )

    def check_aaa_pattern(self):
        """Check for clear AAA (Arrange-Act-Assert) separation"""
        if self.config.require_aaa_comments == "never":
            return

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
                    is_complex = self.is_test_complex(test_start)

                    # Determine severity based on test complexity
                    if self.config.require_aaa_comments == "always":
                        severity = self.config.missing_aaa_complex
                    elif is_complex:
                        severity = self.config.missing_aaa_complex
                    else:
                        severity = self.config.missing_aaa_simple

                    # Only report if required
                    if self.config.require_aaa_comments == "always" or is_complex:
                        test_type = "complex" if is_complex else "simple"
                        self.add_issue(
                            'structure',
                            f"Line {test_start + 1}: No AAA pattern comments found in {test_type} test",
                            severity=severity
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
                    if assert_count > self.config.max_assertions_error:
                        self.add_issue(
                            'focus',
                            f"Line {test_start}: Test has {assert_count} assertions (>{self.config.max_assertions_error}). " +
                            "This is too many - split into multiple tests",
                            severity='high'
                        )
                    elif assert_count > self.config.max_assertions:
                        self.add_issue(
                            'focus',
                            f"Line {test_start}: Test has {assert_count} assertions. " +
                            "Consider splitting into multiple tests",
                            severity=self.config.too_many_assertions
                        )
                    in_test = False

    def check_for_anti_patterns(self):
        """Check for common anti-patterns"""
        anti_patterns = [
            (r'thread::sleep\(', 'Uses thread::sleep() - potential flaky test', 'high'),
            (r'std::thread::sleep', 'Uses std::thread::sleep - potential flaky test', 'high'),
            (r'\.unwrap\(\).*\.unwrap\(\).*\.unwrap\(\)', 'Multiple unwraps (3+) - use ? with Result<()>', 'high'),
            (r'\.unwrap\(\).*\.unwrap\(\)', 'Multiple unwraps - consider using ? with Result<()>', 'medium'),
            (r'#\[ignore\](?!\s*=)', 'Contains ignored test without reason', 'medium'),
            (r'todo!\(', 'Contains todo!() - incomplete test', 'medium'),
            (r'unimplemented!\(', 'Contains unimplemented!() - incomplete test', 'medium'),
            (r'panic!\(', 'Uses panic!() - consider using assert! or #[should_panic]', 'low'),
            (r'\.expect\(".*"\).*\.expect\(".*"\)', 'Multiple .expect() calls - use ?', 'low'),
        ]

        for i, line in enumerate(self.lines, 1):
            for pattern, message, severity in anti_patterns:
                if re.search(pattern, line):
                    self.add_issue(
                        'anti-pattern',
                        f"Line {i}: {message}",
                        severity=severity
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
                # Reset at test end
                if line.strip().startswith('}'):
                    in_async_test = False

            # Check for async test without #[tokio::test]
            if 'async fn test_' in line and i > 1:
                prev_line = self.lines[i-2]
                if '#[test]' in prev_line and '#[tokio::test]' not in prev_line:
                    self.add_issue(
                        'async',
                        f"Line {i}: Async test without #[tokio::test] attribute",
                        severity='high'
                    )

            # Check for blocking operations in async context
            if in_async_test:
                if 'thread::sleep' in line or 'std::thread::sleep' in line:
                    self.add_issue(
                        'async',
                        f"Line {i}: Blocking sleep in async test - use tokio::time::sleep",
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
    parser = argparse.ArgumentParser(
        description='Analyze Rust test file quality',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Basic analysis
  python analyze-test-quality.py tests/lib.rs

  # With custom config
  python analyze-test-quality.py tests/lib.rs --config .test-quality.toml

  # With command-line options
  python analyze-test-quality.py tests/lib.rs --allow-two-part-names --max-assertions 7

  # Lenient mode (fewer warnings)
  python analyze-test-quality.py tests/lib.rs --lenient
        """
    )

    parser.add_argument('file_path', help='Path to the Rust test file')
    parser.add_argument('--config', type=Path, help='Path to TOML config file')
    parser.add_argument('--allow-two-part-names', action='store_true',
                       help='Allow two-part test names for simple tests')
    parser.add_argument('--max-assertions', type=int,
                       help='Maximum assertions before warning')
    parser.add_argument('--strict-aaa', action='store_true',
                       help='Require AAA comments for all tests')
    parser.add_argument('--lenient', action='store_true',
                       help='Use lenient settings (reduce warnings)')

    args = parser.parse_args()

    # Load config
    if args.config:
        config = AnalysisConfig.from_toml(args.config)
    else:
        config = AnalysisConfig()

    # Apply command-line overrides
    if args.allow_two_part_names:
        config.allow_two_part_simple = True
        config.min_parts = 2

    if args.max_assertions:
        config.max_assertions = args.max_assertions

    if args.strict_aaa:
        config.require_aaa_comments = "always"

    if args.lenient:
        config.require_aaa_comments = "complex"
        config.allow_two_part_simple = True
        config.min_parts = 2
        config.max_assertions = 7
        config.naming_violation = "low"
        config.missing_aaa_simple = "low"

    # Check file exists
    file_path = Path(args.file_path)
    if not file_path.exists():
        print(f"Error: File '{file_path}' not found")
        sys.exit(1)

    # Run analysis
    analyzer = RustTestQualityAnalyzer(str(file_path), config)
    results = analyzer.analyze()

    # Print results
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

    # Exit with non-zero code if score is too low
    if results['score'] < 60:
        sys.exit(1)


if __name__ == '__main__':
    main()
