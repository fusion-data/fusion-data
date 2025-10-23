//! Edit Image 图像处理工具函数
//!
//! 提供各种图像操作的实现，包括模糊、边框、合成、创建、裁剪、绘制、
//! 信息获取、调整大小、旋转、剪切、文字添加和透明处理等。
//!
//! 基于 Rust 生态系统中的图像处理库，主要是 `image` 和 `imageproc` 库。

use std::io::Cursor;

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, imageops::FilterType};
use imageproc::drawing::{
  draw_filled_circle_mut, draw_filled_rect_mut, draw_hollow_circle_mut, draw_hollow_rect_mut, draw_line_segment_mut,
};
use serde_json::Value;

use hetumind_core::workflow::{ExecutionData, NodeExecutionError};

use super::{ImageFormat as HetumindImageFormat, ImageOperation, ImageOperationConfig, ImageOperationMode};

/// 图像信息结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImageInfo {
  /// 图像格式
  pub format: String,
  /// 图像宽度
  pub width: u32,
  /// 图像高度
  pub height: u32,
  /// 颜色深度
  pub color_depth: u8,
  /// 颜色空间
  pub color_space: String,
  /// 是否有透明通道
  pub has_alpha: bool,
  /// 文件大小（字节）
  pub file_size: usize,
}

/// 验证输入数据是否包含有效的图像数据
pub fn validate_image_data(
  input_item: &ExecutionData,
  _data_property_name: &str,
  item_index: usize,
) -> Result<Vec<u8>, NodeExecutionError> {
  // 检查是否存在二进制数据
  let _binary_data = input_item.binary().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Input item {} does not contain binary data", item_index),
  })?;

  // TODO 从 BinaryDataManager 中获取实际的二进制数据，然后进行 base64 计算

  Ok(vec![])
}

/// 获取图像信息
pub fn get_image_info(image_data: &[u8], item_index: usize) -> Result<ImageInfo, NodeExecutionError> {
  let img = image::load_from_memory(image_data).map_err(|e| NodeExecutionError::DataProcessingError {
    message: format!("Failed to load image data for item {}: {}", item_index, e),
  })?;

  let format = image::guess_format(image_data).map_err(|e| NodeExecutionError::DataProcessingError {
    message: format!("Failed to determine image format for item {}: {}", item_index, e),
  })?;

  let format_str = match format {
    ImageFormat::Png => "png",
    ImageFormat::Jpeg => "jpeg",
    ImageFormat::Gif => "gif",
    ImageFormat::WebP => "webp",
    ImageFormat::Tiff => "tiff",
    ImageFormat::Bmp => "bmp",
    _ => "unknown",
  };

  Ok(ImageInfo {
    format: format_str.to_string(),
    width: img.width(),
    height: img.height(),
    color_depth: 8, // 大多数常见格式都是8位
    color_space: if img.color().has_alpha() { "RGBA".to_string() } else { "RGB".to_string() },
    has_alpha: img.color().has_alpha(),
    file_size: image_data.len(),
  })
}

/// 处理图像数据，应用操作序列
pub fn process_image_data(
  image_data: &[u8],
  operations: &[ImageOperationConfig],
  operation_mode: &ImageOperationMode,
  item_index: usize,
) -> Result<Vec<u8>, NodeExecutionError> {
  // 加载图像
  let mut img = image::load_from_memory(image_data).map_err(|e| NodeExecutionError::DataProcessingError {
    message: format!("Failed to load image data for item {}: {}", item_index, e),
  })?;

  // 应用操作序列
  for (op_index, operation) in operations.iter().enumerate() {
    log::debug!("应用操作 {}/{}: {:?}", op_index + 1, operations.len(), operation.operation);

    match apply_image_operation(&mut img, operation, item_index) {
      Ok(()) => {
        log::debug!("操作 {}/{} 成功完成", op_index + 1, operations.len());
      }
      Err(e) => {
        log::error!("操作 {}/{} 失败: {}", op_index + 1, operations.len(), e);

        // 根据操作模式决定是否继续
        match operation_mode {
          ImageOperationMode::MultiStep => {
            // 多步骤模式下，如果配置为在第一个错误时停止，则立即返回错误
            // 这里简化处理，总是返回错误
            return Err(e);
          }
          ImageOperationMode::Single => {
            return Err(e);
          }
        }
      }
    }
  }

  // 将处理后的图像编码为字节
  let mut output_data = Vec::new();
  img.write_to(&mut Cursor::new(&mut output_data), ImageFormat::Png).map_err(|e| {
    NodeExecutionError::DataProcessingError {
      message: format!("Failed to encode processed image for item {}: {}", item_index, e),
    }
  })?;

  Ok(output_data)
}

/// 应用单个图像操作
pub fn apply_image_operation(
  img: &mut DynamicImage,
  operation: &ImageOperationConfig,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  match operation.operation {
    ImageOperation::Blur => apply_blur_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Border => apply_border_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Composite => apply_composite_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Create => apply_create_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Crop => apply_crop_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Draw => apply_draw_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Information => {
      // Information 操作在主流程中特殊处理
    }
    ImageOperation::Resize => apply_resize_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Rotate => apply_rotate_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Shear => apply_shear_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Text => apply_text_operation(img, &operation.parameters, item_index)?,
    ImageOperation::Transparent => apply_transparent_operation(img, &operation.parameters, item_index)?,
  }
  Ok(())
}

/// 应用模糊操作
fn apply_blur_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let _blur = parameters["blur"].as_f64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'blur' parameter for item {}", item_index),
  })? as f32;

  let sigma = parameters["sigma"].as_f64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'sigma' parameter for item {}", item_index),
  })? as f32;

  let blurred = imageproc::filter::gaussian_blur_f32(&img.to_rgba8(), sigma);
  *img = DynamicImage::ImageRgba8(blurred);

  Ok(())
}

/// 应用边框操作
fn apply_border_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let border_color_str =
    parameters["border_color"].as_str().ok_or_else(|| NodeExecutionError::DataProcessingError {
      message: format!("Missing or invalid 'border_color' parameter for item {}", item_index),
    })?;

  let border_width = parameters["border_width"].as_u64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'border_width' parameter for item {}", item_index),
  })? as u32;

  let border_height = parameters["border_height"].as_u64().unwrap_or(border_width.into()) as u32;

  // 解析颜色
  let border_color = parse_color(border_color_str);

  // 创建带边框的新图像
  let new_width = img.width() + 2 * border_width;
  let new_height = img.height() + 2 * border_height;
  let mut new_img = ImageBuffer::from_pixel(new_width, new_height, border_color);

  // 将原图像粘贴到中心
  image::imageops::overlay(&mut new_img, img, border_width as i64, border_height as i64);

  *img = DynamicImage::ImageRgba8(new_img);

  Ok(())
}

/// 应用合成操作
fn apply_composite_operation(
  _img: &mut DynamicImage,
  _parameters: &Value,
  _item_index: usize,
) -> Result<(), NodeExecutionError> {
  // 注意：这里简化处理，实际实现需要获取另一个图像数据
  // 在完整实现中，需要从 ExecutionData 中获取合成图像
  log::warn!("Composite operation is not fully implemented in this example");
  Ok(())
}

/// 应用创建操作
fn apply_create_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let background_color_str =
    parameters["background_color"].as_str().ok_or_else(|| NodeExecutionError::DataProcessingError {
      message: format!("Missing or invalid 'background_color' parameter for item {}", item_index),
    })?;

  let width = parameters["width"].as_u64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'width' parameter for item {}", item_index),
  })? as u32;

  let height = parameters["height"].as_u64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'height' parameter for item {}", item_index),
  })? as u32;

  let background_color = parse_color(background_color_str);
  let new_img = ImageBuffer::from_pixel(width, height, background_color);

  *img = DynamicImage::ImageRgba8(new_img);

  Ok(())
}

/// 应用裁剪操作
fn apply_crop_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let width = parameters["width"].as_u64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'width' parameter for item {}", item_index),
  })? as u32;

  let height = parameters["height"].as_u64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'height' parameter for item {}", item_index),
  })? as u32;

  let position_x = parameters["position_x"].as_i64().unwrap_or(0);
  let position_y = parameters["position_y"].as_i64().unwrap_or(0);

  *img = image::DynamicImage::ImageRgba8(
    image::imageops::crop(img, position_x.try_into().unwrap(), position_y.try_into().unwrap(), width, height)
      .to_image(),
  );

  Ok(())
}

/// 应用绘制操作
fn apply_draw_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let primitive_str = parameters["primitive"].as_str().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'primitive' parameter for item {}", item_index),
  })?;

  let color_str = parameters["color"].as_str().unwrap_or("#000000");
  let color = parse_color(color_str);

  match primitive_str {
    "rectangle" => {
      let start_x = parameters["start_position_x"].as_i64().unwrap_or(0);
      let start_y = parameters["start_position_y"].as_i64().unwrap_or(0);
      let end_x = parameters["end_position_x"].as_i64().unwrap_or(100);
      let end_y = parameters["end_position_y"].as_i64().unwrap_or(100);

      if let Some(rgba_img) = img.as_mut_rgba8() {
        let rect = imageproc::rect::Rect::at(start_x.try_into().unwrap(), start_y.try_into().unwrap())
          .of_size((end_x - start_x).try_into().unwrap(), (end_y - start_y).try_into().unwrap());
        draw_hollow_rect_mut(rgba_img, rect, color);
      }
    }
    "filled_rectangle" => {
      let start_x = parameters["start_position_x"].as_i64().unwrap_or(0);
      let start_y = parameters["start_position_y"].as_i64().unwrap_or(0);
      let end_x = parameters["end_position_x"].as_i64().unwrap_or(100);
      let end_y = parameters["end_position_y"].as_i64().unwrap_or(100);

      if let Some(rgba_img) = img.as_mut_rgba8() {
        let rect = imageproc::rect::Rect::at(start_x.try_into().unwrap(), start_y.try_into().unwrap())
          .of_size((end_x - start_x).try_into().unwrap(), (end_y - start_y).try_into().unwrap());
        draw_filled_rect_mut(rgba_img, rect, color);
      }
    }
    "circle" => {
      let center_x = parameters["center_x"].as_i64().unwrap_or(50);
      let center_y = parameters["center_y"].as_i64().unwrap_or(50);
      let radius = parameters["radius"].as_i64().unwrap_or(25);

      if let Some(rgba_img) = img.as_mut_rgba8() {
        draw_hollow_circle_mut(rgba_img, (center_x as i32, center_y as i32), radius as i32, color);
      }
    }
    "filled_circle" => {
      let center_x = parameters["center_x"].as_i64().unwrap_or(50);
      let center_y = parameters["center_y"].as_i64().unwrap_or(50);
      let radius = parameters["radius"].as_i64().unwrap_or(25);

      if let Some(rgba_img) = img.as_mut_rgba8() {
        draw_filled_circle_mut(rgba_img, (center_x as i32, center_y as i32), radius as i32, color);
      }
    }
    "line" => {
      let start_x = parameters["start_position_x"].as_f64().unwrap_or(0.0);
      let start_y = parameters["start_position_y"].as_f64().unwrap_or(0.0);
      let end_x = parameters["end_position_x"].as_f64().unwrap_or(100.0);
      let end_y = parameters["end_position_y"].as_f64().unwrap_or(100.0);

      if let Some(rgba_img) = img.as_mut_rgba8() {
        draw_line_segment_mut(rgba_img, (start_x as f32, start_y as f32), (end_x as f32, end_y as f32), color);
      }
    }
    _ => {
      return Err(NodeExecutionError::DataProcessingError {
        message: format!("Unsupported draw primitive: {}", primitive_str),
      });
    }
  }

  Ok(())
}

/// 应用调整大小操作
fn apply_resize_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let width = parameters["width"].as_u64().map(|w| w as u32);
  let height = parameters["height"].as_u64().map(|h| h as u32);

  if width.is_none() && height.is_none() {
    return Err(NodeExecutionError::DataProcessingError {
      message: format!("At least one of 'width' or 'height' must be specified for item {}", item_index),
    });
  }

  let resize_option_str = parameters["resize_option"].as_str().unwrap_or("fit_inside");

  // 计算目标尺寸
  let (target_width, target_height) =
    calculate_target_size(img.width(), img.height(), width, height, resize_option_str)?;

  let filter = match resize_option_str {
    "exact" => FilterType::Nearest,
    _ => FilterType::Lanczos3,
  };

  *img = img.resize(target_width, target_height, filter);

  Ok(())
}

/// 应用旋转操作
fn apply_rotate_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let degrees = parameters["degrees"].as_f64().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'degrees' parameter for item {}", item_index),
  })?;

  // 简化的旋转实现 - 使用 image 库的基本旋转功能
  let background_color_str = parameters["background_color"].as_str().unwrap_or("#ffffff");
  let _background_color = parse_color(background_color_str);

  // 使用 image 库的 rotate 方法（90度的倍数）
  if (degrees % 90.0).abs() < f64::EPSILON {
    let rotations = ((degrees / 90.0).rem_euclid(4.0) as i32).rem_euclid(4);
    for _ in 0..rotations {
      *img = img.rotate90();
    }
  } else {
    // 对于非90度倍数的旋转，记录警告但不执行
    log::warn!("Non-90-degree rotation ({} degrees) is not fully implemented", degrees);
  }

  Ok(())
}

/// 应用剪切操作
fn apply_shear_operation(
  _img: &mut DynamicImage,
  _parameters: &Value,
  _item_index: usize,
) -> Result<(), NodeExecutionError> {
  let _degrees_x = _parameters["degrees_x"].as_f64().unwrap_or(0.0);
  let _degrees_y = _parameters["degrees_y"].as_f64().unwrap_or(0.0);

  // TODO: 实现剪切变换
  // 由于复杂性，暂时跳过剪切操作的实现
  log::warn!("Shear operation is not fully implemented in this example");

  Ok(())
}

/// 应用文字操作
fn apply_text_operation(
  _img: &mut DynamicImage,
  _parameters: &Value,
  _item_index: usize,
) -> Result<(), NodeExecutionError> {
  let _text = _parameters["text"].as_str().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'text' parameter for item {}", _item_index),
  })?;

  let font_color_str = _parameters["font_color"].as_str().unwrap_or("#000000");
  let _font_color = parse_color(font_color_str);

  let _font_size = _parameters["font_size"].as_u64().unwrap_or(24) as f32;
  let _position_x = _parameters["position_x"].as_i64().unwrap_or(10);
  let _position_y = _parameters["position_y"].as_i64().unwrap_or(50);

  // TODO: 实现文字渲染功能
  // 由于字体处理的复杂性，这里先跳过文字渲染的实现
  // 在实际应用中，需要：
  // 1. 加载字体文件（从系统字体目录或资源文件）
  // 2. 使用 imageproc::drawing::draw_text_mut 渲染文字
  // 3. 处理文字换行和字体度量

  log::warn!("Text operation is not fully implemented in this example");

  Ok(())
}

/// 应用透明操作
fn apply_transparent_operation(
  img: &mut DynamicImage,
  parameters: &Value,
  item_index: usize,
) -> Result<(), NodeExecutionError> {
  let color_str = parameters["color"].as_str().ok_or_else(|| NodeExecutionError::DataProcessingError {
    message: format!("Missing or invalid 'color' parameter for item {}", item_index),
  })?;

  let target_color = parse_color(color_str);

  if let Some(rgba_img) = img.as_mut_rgba8() {
    for pixel in rgba_img.pixels_mut() {
      if *pixel == target_color {
        pixel[3] = 0; // 设置 alpha 通道为 0（完全透明）
      }
    }
  }

  Ok(())
}

/// 准备输出格式
pub fn prepare_output_format(
  image_data: &[u8],
  output_options: &super::ImageOutputOptions,
  _data_property_name: &str,
  item_index: usize,
) -> Result<(Vec<u8>, Option<String>), NodeExecutionError> {
  // 加载图像
  let img = image::load_from_memory(image_data).map_err(|e| NodeExecutionError::DataProcessingError {
    message: format!("Failed to load image for output formatting (item {}): {}", item_index, e),
  })?;

  // 确定输出格式
  let output_format = match output_options.format.as_ref() {
    Some(HetumindImageFormat::Bmp) => ImageFormat::Bmp,
    Some(HetumindImageFormat::Gif) => ImageFormat::Gif,
    Some(HetumindImageFormat::Jpeg) => ImageFormat::Jpeg,
    Some(HetumindImageFormat::Png) => ImageFormat::Png,
    Some(HetumindImageFormat::Tiff) => ImageFormat::Tiff,
    Some(HetumindImageFormat::WebP) => ImageFormat::WebP,
    None => ImageFormat::Png, // 默认 PNG
  };

  // 生成输出文件名
  let file_name = output_options
    .file_name_template
    .as_ref()
    .map(|template| format!("{}.{}", template, format_extension(output_format)));

  // 编码图像
  let mut output_data = Vec::new();
  img.write_to(&mut Cursor::new(&mut output_data), output_format).map_err(|e| {
    NodeExecutionError::DataProcessingError {
      message: format!(
        "Failed to encode image to {} format (item {}): {:?}",
        format_extension(output_format),
        item_index,
        e
      ),
    }
  })?;

  Ok((output_data, file_name))
}

/// 解析颜色字符串
fn parse_color(color_str: &str) -> Rgba<u8> {
  // 简化的颜色解析，支持 #RRGGBB 和 #AARRGGBB 格式
  if color_str.starts_with('#') {
    let hex = &color_str[1..];
    if hex.len() == 6 {
      // #RRGGBB 格式
      let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
      let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
      let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
      return Rgba([r, g, b, 255]);
    } else if hex.len() == 8 {
      // #AARRGGBB 格式
      let a = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
      let r = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
      let g = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
      let b = u8::from_str_radix(&hex[6..8], 16).unwrap_or(0);
      return Rgba([r, g, b, a]);
    }
  }

  // 默认黑色
  Rgba([0, 0, 0, 255])
}

/// 计算目标尺寸
fn calculate_target_size(
  original_width: u32,
  original_height: u32,
  target_width: Option<u32>,
  target_height: Option<u32>,
  _resize_option: &str,
) -> Result<(u32, u32), NodeExecutionError> {
  match (target_width, target_height) {
    (Some(width), Some(height)) => Ok((width, height)),
    (Some(width), None) => {
      let aspect_ratio = original_height as f64 / original_width as f64;
      let height = (width as f64 * aspect_ratio) as u32;
      Ok((width, height))
    }
    (None, Some(height)) => {
      let aspect_ratio = original_width as f64 / original_height as f64;
      let width = (height as f64 * aspect_ratio) as u32;
      Ok((width, height))
    }
    (None, None) => {
      Err(NodeExecutionError::DataProcessingError { message: "Both width and height cannot be None".to_string() })
    }
  }
}

/// 获取格式的文件扩展名
fn format_extension(format: ImageFormat) -> &'static str {
  match format {
    ImageFormat::Png => "png",
    ImageFormat::Jpeg => "jpg",
    ImageFormat::Gif => "gif",
    ImageFormat::WebP => "webp",
    ImageFormat::Tiff => "tiff",
    ImageFormat::Bmp => "bmp",
    _ => "png",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_color() {
    let black = parse_color("#000000");
    assert_eq!(black, Rgba([0, 0, 0, 255]));

    let red = parse_color("#FF0000");
    assert_eq!(red, Rgba([255, 0, 0, 255]));

    let transparent_blue = parse_color("#770000FF");
    assert_eq!(transparent_blue, Rgba([0, 0, 255, 119]));
  }

  #[test]
  fn test_calculate_target_size() {
    // 只指定宽度
    let (width, height) = calculate_target_size(800, 600, Some(400), None, "fit_inside").unwrap();
    assert_eq!(width, 400);
    assert_eq!(height, 300);

    // 只指定高度
    let (width, height) = calculate_target_size(800, 600, None, Some(300), "fit_inside").unwrap();
    assert_eq!(width, 400);
    assert_eq!(height, 300);

    // 指定宽度和高度
    let (width, height) = calculate_target_size(800, 600, Some(500), Some(400), "fit_inside").unwrap();
    assert_eq!(width, 500);
    assert_eq!(height, 400);
  }

  #[test]
  fn test_format_extension() {
    assert_eq!(format_extension(ImageFormat::Png), "png");
    assert_eq!(format_extension(ImageFormat::Jpeg), "jpg");
    assert_eq!(format_extension(ImageFormat::Gif), "gif");
  }
}
