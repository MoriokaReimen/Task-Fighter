#!/usr/bin/env python3
import os
import re
import sys

CARGO_VERSION_REGEX = re.compile(
    r'(?P<prefix>\[package\]\n(?:.*\n)*?version\s*=\s*")(?P<version>[^"]+)(?P<suffix>")',
    re.MULTILINE
)

NUSPEC_VERSION_REGEX = re.compile(
    r'(?P<prefix><version>)(?P<version>[^<]+)(?P<suffix></version>)',
    re.IGNORECASE
)

def update_file_content(file_path, regex, new_version):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        if not regex.search(content):
            return False

        new_content = regex.sub(
            lambda m: f"{m.group('prefix')}{new_version}{m.group('suffix')}",
            content
        )

        if content != new_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            return True
            
    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
    
    return False

def main():
    if len(sys.argv) < 2:
        print("Usage: python script/update_version.py <new_version>", file=sys.stderr)
        print("Example: python script/update_version.py 0.2.0", file=sys.stderr)
        sys.exit(1)

    new_version = sys.argv[1]
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(script_dir)

    print(f"Scanning repository at: {repo_root}")
    print(f"Updating files to v{new_version}...")
    
    updated_count = 0
    
    for root, dirs, files in os.walk(repo_root):
        if '.git' in dirs: dirs.remove('.git')
        if 'target' in dirs: dirs.remove('target')
        if 'script' in dirs: dirs.remove('script')

        for file in files:
            file_path = os.path.join(root, file)
            is_updated = False

            if file == 'Cargo.toml':
                if update_file_content(file_path, CARGO_VERSION_REGEX, new_version):
                    is_updated = True

            elif file.endswith('.nuspec'):
                if update_file_content(file_path, NUSPEC_VERSION_REGEX, new_version):
                    is_updated = True

            if is_updated:
                rel_path = os.path.relpath(file_path, repo_root)
                print(f" Updated: {rel_path}")
                updated_count += 1

    print(f"\nDone! Successfully updated {updated_count} file(s).")

if __name__ == '__main__':
    main()
