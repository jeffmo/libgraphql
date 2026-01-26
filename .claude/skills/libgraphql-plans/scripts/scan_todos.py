#!/usr/bin/env python3
"""
Scan a Rust codebase for TODO comments and similar markers.

Usage:
    python scan_todos.py <repo-path> [--json]

Outputs a structured list of TODOs grouped by which plans.md file they belong to.
"""

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Optional


@dataclass
class TodoItem:
    file: str           # Path relative to repo root
    line: int           # Line number (1-indexed)
    marker: str         # TODO, FIXME, NOTE, HACK, or SEMANTIC
    text: str           # The comment text
    plans_file: str     # Which plans.md this belongs to


# Explicit TODO markers
EXPLICIT_PATTERNS = [
    (r'//\s*TODO:?\s*(.*)$', 'TODO'),
    (r'//\s*FIXME:?\s*(.*)$', 'FIXME'),
    (r'//\s*NOTE:?\s*(.*)$', 'NOTE'),
    (r'//\s*HACK:?\s*(.*)$', 'HACK'),
    (r'/\*\s*TODO:?\s*(.*?)\s*\*/', 'TODO'),
    (r'/\*\s*FIXME:?\s*(.*?)\s*\*/', 'FIXME'),
]

# Semantic patterns suggesting future work (lower confidence)
SEMANTIC_PATTERNS = [
    r'//.*\b(fix this|clean ?up|reconsider|revisit)\b',
    r'//.*\b(temporary|workaround|should be changed)\b',
    r'//.*\b(will need to|should eventually|needs to be)\b',
]


def find_crate_for_file(file_path: Path, repo_root: Path) -> Optional[str]:
    """Determine which crate a file belongs to by looking for Cargo.toml."""
    current = file_path.parent
    while current != repo_root and current != current.parent:
        cargo_toml = current / "Cargo.toml"
        if cargo_toml.exists():
            return str(current.relative_to(repo_root))
        current = current.parent
    return None


def get_plans_file(file_path: Path, repo_root: Path) -> str:
    """Determine which plans.md file a TODO should be tracked in."""
    crate_path = find_crate_for_file(file_path, repo_root)
    if crate_path:
        return f"{crate_path}/plans.md"
    return "plans.md"  # Root-level plans.md


def scan_file(file_path: Path, repo_root: Path) -> list[TodoItem]:
    """Scan a single file for TODOs."""
    todos = []
    rel_path = str(file_path.relative_to(repo_root))
    plans_file = get_plans_file(file_path, repo_root)
    
    try:
        with open(file_path, 'r', encoding='utf-8', errors='replace') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"Warning: Could not read {file_path}: {e}", file=sys.stderr)
        return []
    
    for line_num, line in enumerate(lines, start=1):
        # Check explicit patterns
        for pattern, marker in EXPLICIT_PATTERNS:
            match = re.search(pattern, line, re.IGNORECASE)
            if match:
                text = match.group(1).strip() if match.lastindex else line.strip()
                todos.append(TodoItem(
                    file=rel_path,
                    line=line_num,
                    marker=marker,
                    text=text[:200],  # Truncate long comments
                    plans_file=plans_file
                ))
                break  # Only match one pattern per line
        else:
            # Check semantic patterns (lower confidence)
            for pattern in SEMANTIC_PATTERNS:
                if re.search(pattern, line, re.IGNORECASE):
                    # Extract the comment text
                    comment_match = re.search(r'//\s*(.*)$', line)
                    if comment_match:
                        todos.append(TodoItem(
                            file=rel_path,
                            line=line_num,
                            marker='SEMANTIC',
                            text=comment_match.group(1).strip()[:200],
                            plans_file=plans_file
                        ))
                    break
    
    return todos


def scan_repo(repo_path: Path) -> list[TodoItem]:
    """Scan all Rust files in a repository."""
    todos = []
    
    for root, dirs, files in os.walk(repo_path):
        # Skip hidden directories and common non-source directories
        dirs[:] = [d for d in dirs if not d.startswith('.') and d not in ('target', 'node_modules', 'vendor')]
        
        for file in files:
            if file.endswith('.rs'):
                file_path = Path(root) / file
                todos.extend(scan_file(file_path, repo_path))
    
    return todos


def group_by_plans_file(todos: list[TodoItem]) -> dict[str, list[TodoItem]]:
    """Group TODOs by which plans.md file they belong to."""
    grouped = {}
    for todo in todos:
        if todo.plans_file not in grouped:
            grouped[todo.plans_file] = []
        grouped[todo.plans_file].append(todo)
    return grouped


def format_table(todos: list[TodoItem]) -> str:
    """Format TODOs as a markdown table."""
    if not todos:
        return "No TODOs found."
    
    lines = ["| File | Line | Marker | TODO |", "|------|------|--------|------|"]
    for todo in sorted(todos, key=lambda t: (t.file, t.line)):
        text = todo.text.replace('|', '\\|')  # Escape pipes
        lines.append(f"| `{todo.file}` | {todo.line} | {todo.marker} | {text} |")
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description='Scan Rust codebase for TODOs')
    parser.add_argument('repo_path', type=Path, help='Path to repository root')
    parser.add_argument('--json', action='store_true', help='Output as JSON')
    parser.add_argument('--group', action='store_true', help='Group by plans.md file')
    args = parser.parse_args()
    
    if not args.repo_path.exists():
        print(f"Error: Path does not exist: {args.repo_path}", file=sys.stderr)
        sys.exit(1)
    
    todos = scan_repo(args.repo_path.resolve())
    
    if args.json:
        if args.group:
            grouped = group_by_plans_file(todos)
            output = {k: [asdict(t) for t in v] for k, v in grouped.items()}
        else:
            output = [asdict(t) for t in todos]
        print(json.dumps(output, indent=2))
    else:
        if args.group:
            grouped = group_by_plans_file(todos)
            for plans_file, file_todos in sorted(grouped.items()):
                print(f"\n## {plans_file}\n")
                print(format_table(file_todos))
        else:
            print(format_table(todos))
    
    # Summary to stderr
    print(f"\nFound {len(todos)} TODO(s) in {len(set(t.file for t in todos))} file(s)", file=sys.stderr)


if __name__ == '__main__':
    main()
