use fusion_ai::utils::base64_json_to_image;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // 从当前脚本目录读取 base64_image.json 文件
  let json_path = "base64_image.json";

  // 读取 JSON 文件内容
  let json_content = fs::read_to_string(json_path)?;

  // 指定输出文件名
  let output_filename = "runs/saved_image_from_json.jpg";

  // 调用函数进行转换
  match base64_json_to_image(&json_content, output_filename, "b64_json") {
    Ok(()) => {
      println!("操作完成！");
    }
    Err(e) => {
      eprintln!("操作失败: {}", e);
    }
  }

  Ok(())
}
