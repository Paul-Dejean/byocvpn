#!/usr/bin/env python3
"""Migrate println!/eprintln! to log crate macros and inject `use log::*;`."""

import os
import re

CRATE_ROOTS = [
    'crates/core/src',
    'crates/daemon/src',
    'crates/cli/src',
    'crates/infra/aws/src',
    'crates/infra/azure/src',
    'crates/infra/gcp/src',
    'crates/infra/oracle/src',
    'crates/ui/src-tauri/src',
]

SKIP_DIRS = {'target', 'node_modules', '.git', 'gen'}


def find_rs_files(root):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for filename in filenames:
            if filename.endswith('.rs'):
                yield os.path.join(dirpath, filename)


def migrate_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    has_println = 'println!' in content
    has_eprintln = 'eprintln!' in content

    if not has_println and not has_eprintln:
        return False

    new_content = content
    # Replace eprintln! BEFORE println! to avoid 'e' prefix being left behind
    new_content = new_content.replace('eprintln!(', 'error!(')
    new_content = new_content.replace('println!(', 'info!(')

    if 'use log::' not in new_content:
        lines = new_content.split('\n')
        # Find the end of the last complete use statement (handles multi-line blocks)
        last_use_end = -1
        i = 0
        while i < len(lines):
            if re.match(r'^use ', lines[i]):
                j = i
                # scan forward until the statement ends with ';'
                while j < len(lines):
                    if lines[j].rstrip().endswith(';'):
                        last_use_end = j
                        break
                    j += 1
                i = j + 1
            else:
                i += 1

        import_line = 'use log::*;'
        if last_use_end >= 0:
            lines.insert(last_use_end + 1, import_line)
        else:
            lines.insert(0, import_line)

        new_content = '\n'.join(lines)

    if new_content != content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False


if __name__ == '__main__':
    workspace = '/Users/paul/projects/on-demand-vpn'
    changed = 0
    total = 0
    for crate_root in CRATE_ROOTS:
        full_root = os.path.join(workspace, crate_root)
        for filepath in find_rs_files(full_root):
            total += 1
            if migrate_file(filepath):
                changed += 1
                print(f'  {filepath.replace(workspace + "/", "")}')

    print(f'\nDone: {changed}/{total} files modified')
