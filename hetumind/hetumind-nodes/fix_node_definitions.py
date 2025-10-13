#!/usr/bin/env python3
"""
修复 NodeDefinition API 使用的脚本
将旧的构建器模式转换为新的链式方法调用
"""

import os
import re
import glob

def fix_node_definition_file(file_path):
    """修复单个文件中的 NodeDefinition API 使用"""
    print(f"正在修复文件: {file_path}")
    
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    
    # 1. 修复 .version() 调用
    content = re.sub(
        r'\.version\(([^)]+)\)',
        r'',  # 移除 version 调用，因为它现在是字段
        content
    )
    
    # 2. 修复 .inputs() 调用
    content = re.sub(
        r'\.inputs\(\[([^\]]+)\]\)',
        lambda m: f'.add_input({m.group(1).strip()})',
        content
    )
    
    # 3. 修复 .outputs() 调用
    content = re.sub(
        r'\.outputs\(\[([^\]]+)\]\)',
        lambda m: f'.add_output({m.group(1).strip()})',
        content
    )
    
    # 4. 修复 .properties() 调用 - 这个比较复杂，需要特殊处理
    # 先找到 .properties([ 的位置
    properties_pattern = r'\.properties\(\['
    properties_matches = list(re.finditer(properties_pattern, content))
    
    if properties_matches:
        # 从后往前处理，避免位置偏移
        for match in reversed(properties_matches):
            start_pos = match.start()
            # 找到对应的结束括号
            bracket_count = 0
            pos = match.end() - 1  # 从 [ 开始
            end_pos = None
            
            while pos < len(content):
                if content[pos] == '[':
                    bracket_count += 1
                elif content[pos] == ']':
                    bracket_count -= 1
                    if bracket_count == 0:
                        end_pos = pos + 2  # 包含 ])
                        break
                pos += 1
            
            if end_pos:
                # 提取 properties 内容
                properties_content = content[match.end()-1:end_pos-1]  # 不包含最后的 ])
                
                # 分割各个属性
                properties_items = []
                bracket_count = 0
                paren_count = 0
                current_item = ""
                i = 1  # 跳过开头的 [
                
                while i < len(properties_content) - 1:  # 跳过结尾的 ]
                    char = properties_content[i]
                    current_item += char
                    
                    if char == '[':
                        bracket_count += 1
                    elif char == ']':
                        bracket_count -= 1
                    elif char == '(':
                        paren_count += 1
                    elif char == ')':
                        paren_count -= 1
                    elif char == ',' and bracket_count == 0 and paren_count == 0:
                        # 找到一个完整的属性项
                        properties_items.append(current_item[:-1].strip())  # 移除末尾的逗号
                        current_item = ""
                        i += 1
                        continue
                    
                    i += 1
                
                # 添加最后一个项目
                if current_item.strip():
                    properties_items.append(current_item.strip())
                
                # 构建新的链式调用
                new_calls = ""
                for item in properties_items:
                    if item.strip():
                        new_calls += f".add_property({item.strip()})"
                
                # 替换原内容
                content = content[:start_pos] + new_calls + content[end_pos:]
    
    # 5. 修复 NodePropertyKindOptions::builder() 调用
    content = re.sub(
        r'NodePropertyKindOptions::builder\(\)([^.]*?)(?=\s*[,)])',
        lambda m: f'NodePropertyKindOptions::builder(){m.group(1)}.build()',
        content,
        flags=re.DOTALL
    )
    
    # 6. 修复 NodeProperty::builder() 调用 - 确保都有 .build()
    # 找到所有 NodeProperty::builder() 开始的调用
    builder_pattern = r'NodeProperty::builder\(\)'
    builder_matches = list(re.finditer(builder_pattern, content))
    
    if builder_matches:
        for match in reversed(builder_matches):
            start_pos = match.end()
            # 找到这个 builder 调用的结束位置
            pos = start_pos
            paren_count = 0
            dot_chain = True
            
            while pos < len(content) and dot_chain:
                if content[pos] == '(':
                    paren_count += 1
                elif content[pos] == ')':
                    paren_count -= 1
                elif content[pos] == '.' and paren_count == 0:
                    # 继续链式调用
                    pass
                elif content[pos] in [',', ';', '}', ']'] and paren_count == 0:
                    # 链式调用结束
                    break
                pos += 1
            
            # 检查是否已经有 .build()
            chain_content = content[start_pos:pos]
            if '.build()' not in chain_content:
                # 在适当位置添加 .build()
                content = content[:pos] + '.build()' + content[pos:]
    
    # 7. 修复 base? 的使用
    content = re.sub(
        r'let\s+definition\s*=\s*base\?\s*;',
        'let definition = base;',
        content
    )
    
    # 8. 确保 version 字段在 NodeDefinition::new() 中设置
    # 查找 NodeDefinition::new() 调用
    new_pattern = r'NodeDefinition::new\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)'
    new_matches = list(re.finditer(new_pattern, content))
    
    for match in reversed(new_matches):
        node_type = match.group(1)
        display_name = match.group(2)
        # 添加 version 参数
        new_call = f'NodeDefinition::new("{node_type}", "{display_name}", Version::new(1, 0, 0))'
        content = content[:match.start()] + new_call + content[match.end():]
    
    # 只有在内容发生变化时才写入文件
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"已修复文件: {file_path}")
        return True
    else:
        print(f"文件无需修复: {file_path}")
        return False

def main():
    """主函数"""
    # 获取所有需要修复的 Rust 文件
    rust_files = []
    
    # 搜索 src 目录下的所有 .rs 文件
    for root, dirs, files in os.walk('src'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"找到 {len(rust_files)} 个 Rust 文件")
    
    fixed_count = 0
    for file_path in rust_files:
        try:
            if fix_node_definition_file(file_path):
                fixed_count += 1
        except Exception as e:
            print(f"修复文件 {file_path} 时出错: {e}")
    
    print(f"总共修复了 {fixed_count} 个文件")

if __name__ == '__main__':
    main()