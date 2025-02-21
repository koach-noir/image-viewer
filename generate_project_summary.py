import os
import fnmatch
import chardet

def is_binary(file_path):
    with open(file_path, 'rb') as file:
        return b'\0' in file.read(1024)

def read_file_contents(file_path):
    encodings = ['utf-8', 'shift_jis']
    for encoding in encodings:
        try:
            with open(file_path, 'r', encoding=encoding) as file:
                return file.read()
        except UnicodeDecodeError:
            pass
    return ''

def is_ignored(path, project_dir, gitignore_patterns, summaryignore_patterns, structureignore_patterns, additional_ignore_patterns, check_structure=False):
    relative_path = os.path.relpath(path, project_dir)
    relative_path = relative_path.replace('\\', '/') # Windowsパスを統一
    
    # ディレクトリ構造チェック時は structureignore_patterns を使用
    patterns_to_check = gitignore_patterns + additional_ignore_patterns
    if check_structure:
        patterns_to_check += structureignore_patterns
    else:
        patterns_to_check += summaryignore_patterns
    
    for pattern in patterns_to_check:
        pattern = pattern.replace('\\', '/') # パターンも統一
        
        # **をサポートするためのパターン処理
        if '**' in pattern:
            pattern = pattern.replace('**/', '')
            if pattern.startswith('/'):
                pattern = pattern[1:]
            pattern = f"*{pattern}*"
        
        if fnmatch.fnmatch(relative_path, pattern):
            return True
            
        if fnmatch.fnmatch(f'/{relative_path}', pattern):
            return True
    
    return False

def generate_project_summary(project_dir):
    project_name = os.path.basename(project_dir)
    summary = f'# {project_name}\n\n## Directory Structure\n\n'

    gitignore_patterns = read_gitignore(project_dir)
    summaryignore_patterns = read_summaryignore(project_dir)
    structureignore_patterns = read_structureignore(project_dir)
    additional_ignore_patterns = ['generate_project_summary.py', '.summaryignore', '.summarystructureignore', f'{project_name}_project_summary.md', '.git']

    file_contents_section = "\n## File Contents\n\n"

    def traverse_directory(root, level, include_contents=True):
        nonlocal summary, file_contents_section
        indent = '  ' * level
        relative_path = os.path.relpath(root, project_dir)
        
        # 明らかに除外すべきディレクトリを早期に除外
        if os.path.basename(root) in ['node_modules', '.git', 'dist']:
            return

        # Directory Structure用の無視判定
        if is_ignored(root, project_dir, gitignore_patterns, summaryignore_patterns, structureignore_patterns, additional_ignore_patterns, check_structure=True):
            return

        summary += f'{indent}- {os.path.basename(root)}/\n'

        subindent = '  ' * (level + 1)
        items = os.listdir(root)
        
        # ディレクトリとファイルを分離して処理
        dirs = []
        files = []
        for item in items:
            item_path = os.path.join(root, item)
            if os.path.isdir(item_path):
                dirs.append(item)
            else:
                files.append(item)

        # ディレクトリとファイルをそれぞれソート
        for item in sorted(dirs):
            item_path = os.path.join(root, item)
            traverse_directory(item_path, level + 1, include_contents)
            
        for item in sorted(files):
            item_path = os.path.join(root, item)
            
            # Directory Structure用の無視判定
            if not is_ignored(item_path, project_dir, gitignore_patterns, summaryignore_patterns, structureignore_patterns, additional_ignore_patterns, check_structure=True):
                summary += f'{subindent}- {item}\n'
            
            # File Contentsセクションの処理
            if include_contents and not is_binary(item_path):
                if not is_ignored(item_path, project_dir, gitignore_patterns, summaryignore_patterns, structureignore_patterns, additional_ignore_patterns, check_structure=False):
                    content = read_file_contents(item_path)
                    if content.strip():
                        relative_file_path = os.path.relpath(item_path, project_dir)
                        file_contents_section += f'### {relative_file_path}\n\n```\n{content}\n```\n\n'

    traverse_directory(project_dir, 0)

    output_path = os.path.join(project_dir, f'{project_name}_project_summary.md')
    with open(output_path, 'w', encoding='utf-8') as file:
        file.write(summary + file_contents_section)

def read_gitignore(project_dir):
    gitignore_path = os.path.join(project_dir, '.gitignore')
    if os.path.exists(gitignore_path):
        with open(gitignore_path, 'r') as file:
            patterns = [line.strip() for line in file if line.strip() and not line.startswith('#')]
            expanded_patterns = []
            for pattern in patterns:
                expanded_patterns.append(pattern)
                if '/' in pattern:
                    expanded_patterns.append(pattern.replace('/', '\\'))
                if '\\' in pattern:
                    expanded_patterns.append(pattern.replace('\\', '/'))
            return expanded_patterns
    return []

def read_summaryignore(project_dir):
    summaryignore_path = os.path.join(project_dir, '.summaryignore')
    if os.path.exists(summaryignore_path):
        with open(summaryignore_path, 'r') as file:
            patterns = [line.strip() for line in file if line.strip() and not line.startswith('#')]
            expanded_patterns = []
            for pattern in patterns:
                expanded_patterns.append(pattern)
                if '/' in pattern:
                    expanded_patterns.append(pattern.replace('/', '\\'))
                if '\\' in pattern:
                    expanded_patterns.append(pattern.replace('\\', '/'))
            return expanded_patterns
    return []

def read_structureignore(project_dir):
    structureignore_path = os.path.join(project_dir, '.summarystructureignore')
    if os.path.exists(structureignore_path):
        with open(structureignore_path, 'r') as file:
            patterns = [line.strip() for line in file if line.strip() and not line.startswith('#')]
            expanded_patterns = []
            for pattern in patterns:
                expanded_patterns.append(pattern)
                if '/' in pattern:
                    expanded_patterns.append(pattern.replace('/', '\\'))
                if '\\' in pattern:
                    expanded_patterns.append(pattern.replace('\\', '/'))
            return expanded_patterns
    return []

if __name__ == '__main__':
    project_directory = input('Enter the project directory path (leave blank for current directory): ')
    if not project_directory:
        project_directory = os.getcwd()
    generate_project_summary(project_directory)
