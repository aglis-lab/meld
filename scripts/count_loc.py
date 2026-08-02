#!/usr/bin/env python3
"""
Count lines of code (LOC) for all extensions in the meld project.
"""

import os
import sys
from pathlib import Path
from typing import Dict, Tuple
from collections import defaultdict

# File extensions to count
LANGUAGE_EXTENSIONS = {
    ".rs": "Rust",
    ".go": "Go",
    ".ts": "TypeScript",
    ".tsx": "TypeScript",
    ".js": "JavaScript",
    ".jsx": "JavaScript",
    ".html": "HTML",
    ".css": "CSS",
    ".json": "JSON",
    ".toml": "TOML",
    ".yaml": "YAML",
    ".yml": "YAML",
    ".py": "Python",
}

# Directories to skip
SKIP_DIRS = {
    "node_modules",
    "target",
    ".git",
    "build",
    "dist",
    ".vscode",
    "__pycache__",
    ".pytest_cache",
    ".DS_Store",
    "CACHEDIR.TAG",
}


def is_skip_path(path: Path) -> bool:
    """Check if path should be skipped."""
    for skip_dir in SKIP_DIRS:
        if skip_dir in path.parts:
            return True
    return False


def count_lines(file_path: Path) -> Tuple[int, int]:
    """
    Count total and non-empty lines in a file.
    Returns (total_lines, non_empty_lines)
    """
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
            total = len(lines)
            non_empty = sum(1 for line in lines if line.strip())
            return total, non_empty
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return 0, 0


def scan_directory(base_path: Path) -> Dict[str, Dict[str, int]]:
    """
    Scan directory and count LOC by language and extension.
    """
    stats = defaultdict(lambda: {"total": 0, "non_empty": 0, "files": 0})

    for file_path in base_path.rglob("*"):
        if file_path.is_file() and not is_skip_path(file_path):
            ext = file_path.suffix.lower()
            if ext in LANGUAGE_EXTENSIONS:
                lang = LANGUAGE_EXTENSIONS[ext]
                total, non_empty = count_lines(file_path)
                stats[lang]["total"] += total
                stats[lang]["non_empty"] += non_empty
                stats[lang]["files"] += 1

    return dict(stats)


def print_stats(title: str, stats: Dict[str, Dict[str, int]]) -> None:
    """Print statistics in a formatted table."""
    if not stats:
        print(f"{title}: No files found")
        return

    print(f"\n{title}")
    print("=" * 70)
    print(f"{'Language':<15} {'Files':>8} {'Total LOC':>12} {'Non-Empty':>12}")
    print("-" * 70)

    total_files = 0
    total_loc = 0
    total_non_empty = 0

    for lang in sorted(stats.keys()):
        data = stats[lang]
        files = data["files"]
        total = data["total"]
        non_empty = data["non_empty"]

        print(f"{lang:<15} {files:>8} {total:>12} {non_empty:>12}")

        total_files += files
        total_loc += total
        total_non_empty += non_empty

    print("-" * 70)
    print(f"{'TOTAL':<15} {total_files:>8} {total_loc:>12} {total_non_empty:>12}")
    print("=" * 70)


def main():
    """Main entry point."""
    base_path = Path(__file__).parent.parent

    print(f"Scanning {base_path} for LOC...")

    # Scan main Rust project
    rust_stats = scan_directory(base_path / "src")
    print_stats("Rust Implementation (src/)", rust_stats)

    # Scan Go implementation
    go_stats = scan_directory(base_path / "meld-go")
    print_stats("Go Implementation (meld-go/)", go_stats)

    # Scan TypeScript implementation
    ts_stats = scan_directory(base_path / "meld-ts")
    print_stats("TypeScript Implementation (meld-ts/)", ts_stats)

    # Scan Vite plugin
    vite_stats = scan_directory(base_path / "plugins" / "vite")
    print_stats("Vite Plugin (plugins/vite/)", vite_stats)

    # Scan examples
    examples_stats = scan_directory(base_path / "examples")
    print_stats("Examples (examples/)", examples_stats)

    # Scan templates
    templates_stats = scan_directory(base_path / "templates")
    print_stats("Templates (templates/)", templates_stats)

    # Scan benches
    benches_stats = scan_directory(base_path / "benches")
    print_stats("Benchmarks (benches/)", benches_stats)

    # Scan scripts
    scripts_stats = scan_directory(base_path / "scripts")
    print_stats("Scripts (scripts/)", scripts_stats)

    # Combined totals
    print("\n\nCOMBINED TOTALS")
    print("=" * 70)

    all_stats = defaultdict(lambda: {"total": 0, "non_empty": 0, "files": 0})

    for stats_dict in [
        rust_stats,
        go_stats,
        ts_stats,
        vite_stats,
        examples_stats,
        templates_stats,
        benches_stats,
        scripts_stats,
    ]:
        for lang, data in stats_dict.items():
            all_stats[lang]["total"] += data["total"]
            all_stats[lang]["non_empty"] += data["non_empty"]
            all_stats[lang]["files"] += data["files"]

    print_stats("All Extensions Combined", dict(all_stats))


if __name__ == "__main__":
    main()
