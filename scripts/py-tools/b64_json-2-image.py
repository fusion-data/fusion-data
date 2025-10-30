import json
import base64
import re
import os


def base64_json_to_image(
    b64_json_string: str, output_path: str, data_key: str = "b64_json"
):
    """
    将包含 Base64 图片数据的 JSON 字符串转换为图片文件。

    :param b64_json_string: 包含 Base64 图片数据的 JSON 字符串。
    :param output_path: 保存图片的路径（例如 'output.png'）。
    :param data_key: JSON 中存储 Base64 数据的键名。
    :return: 如果成功返回 True，失败返回 False。
    """
    try:
        # --- 第 1 步: 解析 JSON ---
        # 将 JSON 字符串解析为 Python 字典
        data = json.loads(b64_json_string)

        # 从字典中获取 Base64 字符串
        base64_string = data[data_key]

        # --- 第 2 步: 清理数据 ---
        # 检查并移除可能存在的 Data URI 前缀
        # 例如: "data:image/png;base64,iVBORw0KGgo..."
        # 使用正则表达式匹配并移除 "data:image/...;base64," 部分
        base64_string = re.sub(r"^data:image/\w+;base64,", "", base64_string)

        # --- 第 3 步: 解码 Base64 ---
        # 将 Base64 字符串解码为二进制数据 (bytes)
        image_data = base64.b64decode(base64_string)

        # --- 第 4 步: 保存为文件 ---
        # 以二进制写入模式 ('wb') 打开文件
        with open(output_path, "wb") as image_file:
            # 将二进制数据写入文件
            image_file.write(image_data)

        print(f"图片成功保存到: {output_path}")
        return True

    except json.JSONDecodeError:
        print("错误: 无效的 JSON 字符串。")
    except KeyError:
        print(f"错误: JSON 中找不到键 '{data_key}'。")
    except base64.binascii.Error:
        print("错误: 无效的 Base64 字符串。")
    except IOError as e:
        print(f"错误: 无法写入文件 '{output_path}'。原因: {e}")
    except Exception as e:
        print(f"发生未知错误: {e}")

    return False


# --- 主程序入口 ---
# uv run -m scripts.py-tools.b64_json-2-image
if __name__ == "__main__":
    # 示例 b64_json 字符串
    # 这是一个 1x1 像素的红色 PNG 图片的 Base64 编码
    # 注意：实际数据可能包含 "data:image/png;base64," 这样的前缀
    # 从当前脚本目录读取 base64_image.json 文件
    json_path = os.path.join(os.path.dirname(__file__), "base64_image.json")
    with open(json_path, "r", encoding="utf-8") as f:
        example_b64_json = f.read()

    # 指定输出文件名
    output_filename = "saved_image.png"

    # 调用函数进行转换
    success = base64_json_to_image(
        example_b64_json, output_filename, data_key="b64_json"
    )

    if success:
        print("操作完成！")
    else:
        print("操作失败。")
