//! Edit Image 图像处理节点实现
//!
//! 参考 n8n 的 Edit Image 节点设计，用于处理和编辑图像。
//! 支持多种图像操作，包括模糊、边框、裁剪、绘制、调整大小、旋转、添加文本等。
//!
//! # 主要功能特性
//! - **多样化图像操作**: 支持11种核心图像操作
//! - **多步骤处理**: 支持在一个节点中执行多个操作序列
//! - **图像信息获取**: 可以提取图像的元数据信息
//! - **灵活的输出格式**: 支持多种图像格式输出（BMP、GIF、JPEG、PNG、TIFF、WebP）
//! - **质量控制**: 可调节图像质量和压缩级别
//! - **字体支持**: 自动获取系统字体并支持自定义字体
//! - **二进制数据处理**: 完整的二进制数据流处理支持
//!
//! # 支持的图像操作类型
//! - `Blur`: 高斯模糊处理
//! - `Border`: 添加图像边框
//! - `Composite`: 图像合成和叠加
//! - `Create`: 创建新图像
//! - `Crop`: 裁剪图像
//! - `Draw`: 绘制基本形状（圆形、线条、矩形）
//! - `Rotate`: 旋转图像
//! - `Resize`: 改变图像尺寸
//! - `Shear`: X/Y轴剪切变换
//! - `Text`: 添加文字到图像
//! - `Transparent`: 使指定颜色透明
//! - `Information`: 获取图像元数据

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{FlowNodeRef, Node, NodeDefinition, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod edit_image_v1;
mod utils;

use edit_image_v1::EditImageV1;

use crate::constants::EDIT_IMAGE_NODE_KIND;

/// 图像操作类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageOperation {
  /// 高斯模糊
  Blur,
  /// 添加边框
  Border,
  /// 图像合成
  Composite,
  /// 创建新图像
  Create,
  /// 裁剪图像
  Crop,
  /// 绘制形状
  Draw,
  /// 获取图像信息
  Information,
  /// 调整大小
  Resize,
  /// 旋转图像
  Rotate,
  /// 剪切变换
  Shear,
  /// 添加文字
  Text,
  /// 透明处理
  Transparent,
}

/// 图像格式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
  /// BMP 格式
  Bmp,
  /// GIF 格式
  Gif,
  /// JPEG 格式
  Jpeg,
  /// PNG 格式
  Png,
  /// TIFF 格式
  Tiff,
  /// WebP 格式
  WebP,
}

/// 调整大小选项
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeOption {
  /// 保持宽高比，缩小以适应
  FitInside,
  /// 保持宽高比，放大以填充
  FitOutside,
  /// 填充指定尺寸，可能裁剪
  Cover,
  /// 精确拉伸到指定尺寸
  Exact,
}

/// 图像合成操作符
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositeOperator {
  /// 默认合成操作
  Over,
  /// 在下方合成
  Under,
  /// 相加
  Plus,
  /// 相减
  Minus,
  /// 相乘
  Multiply,
  /// 差值
  Difference,
}

/// 绘制基本形状类型
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawPrimitive {
  /// 圆形
  Circle,
  /// 椭圆
  Ellipse,
  /// 直线
  Line,
  /// 矩形
  Rectangle,
  /// 圆角矩形
  RoundedRectangle,
  /// 多边形
  Polygon,
}

/// 单个图像操作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOperationConfig {
  /// 操作类型
  pub operation: ImageOperation,
  /// 操作参数
  pub parameters: Value,
  /// 操作描述（可选）
  pub description: Option<String>,
}

impl ImageOperationConfig {
  /// 验证操作配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    match self.operation {
      ImageOperation::Blur => {
        if self.parameters.get("blur").is_none() || self.parameters.get("sigma").is_none() {
          return Err("Blur operation requires 'blur' and 'sigma' parameters".to_string());
        }
      }
      ImageOperation::Border => {
        if self.parameters.get("border_color").is_none() || self.parameters.get("border_width").is_none() {
          return Err("Border operation requires 'border_color' and 'border_width' parameters".to_string());
        }
      }
      ImageOperation::Create => {
        if self.parameters.get("background_color").is_none()
          || self.parameters.get("width").is_none()
          || self.parameters.get("height").is_none()
        {
          return Err("Create operation requires 'background_color', 'width' and 'height' parameters".to_string());
        }
      }
      ImageOperation::Crop => {
        if self.parameters.get("width").is_none() || self.parameters.get("height").is_none() {
          return Err("Crop operation requires 'width' and 'height' parameters".to_string());
        }
      }
      ImageOperation::Resize => {
        if self.parameters.get("width").is_none() && self.parameters.get("height").is_none() {
          return Err("Resize operation requires at least 'width' or 'height' parameter".to_string());
        }
      }
      ImageOperation::Rotate => {
        if self.parameters.get("degrees").is_none() {
          return Err("Rotate operation requires 'degrees' parameter".to_string());
        }
      }
      ImageOperation::Text => {
        if self.parameters.get("text").is_none() {
          return Err("Text operation requires 'text' parameter".to_string());
        }
      }
      ImageOperation::Transparent => {
        if self.parameters.get("color").is_none() {
          return Err("Transparent operation requires 'color' parameter".to_string());
        }
      }
      ImageOperation::Composite => {
        if self.parameters.get("data_property_name").is_none() {
          return Err("Composite operation requires 'data_property_name' parameter".to_string());
        }
      }
      ImageOperation::Draw => {
        if self.parameters.get("primitive").is_none() {
          return Err("Draw operation requires 'primitive' parameter".to_string());
        }
      }
      ImageOperation::Shear => {
        if self.parameters.get("degrees_x").is_none() && self.parameters.get("degrees_y").is_none() {
          return Err("Shear operation requires at least 'degrees_x' or 'degrees_y' parameter".to_string());
        }
      }
      ImageOperation::Information => {
        // Information 操作不需要参数
      }
    }
    Ok(())
  }
}

/// 多步骤操作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiStepConfig {
  /// 操作序列
  pub operations: Vec<ImageOperationConfig>,
  /// 是否在第一个操作失败时停止
  pub stop_on_first_error: Option<bool>,
}

/// Edit Image 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditImageConfig {
  /// 操作模式：单操作或多步骤
  pub operation_mode: ImageOperationMode,
  /// 数据属性名称（包含二进制图像数据）
  pub data_property_name: String,
  /// 输出选项
  pub output_options: ImageOutputOptions,
  /// 单操作配置
  pub single_operation: Option<ImageOperationConfig>,
  /// 多步骤配置
  pub multi_step_config: Option<MultiStepConfig>,
}

/// 操作模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageOperationMode {
  /// 单操作模式
  Single,
  /// 多步骤模式
  MultiStep,
}

/// 图像输出选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOutputOptions {
  /// 输出格式
  pub format: Option<ImageFormat>,
  /// 图像质量 (0-100)
  pub quality: Option<u8>,
  /// 输出文件名模板
  pub file_name_template: Option<String>,
  /// 是否保留原始元数据
  pub preserve_metadata: Option<bool>,
}

impl Default for ImageOutputOptions {
  fn default() -> Self {
    Self { format: Some(ImageFormat::Png), quality: Some(90), file_name_template: None, preserve_metadata: Some(false) }
  }
}

impl EditImageConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    // 验证数据属性名称
    if self.data_property_name.trim().is_empty() {
      return Err("Data property name cannot be empty".to_string());
    }

    // 验证操作配置
    match self.operation_mode {
      ImageOperationMode::Single => {
        if self.single_operation.is_none() {
          return Err("Single operation mode requires single_operation config".to_string());
        }
        if let Some(ref operation) = self.single_operation {
          operation.validate()?;
        }
      }
      ImageOperationMode::MultiStep => {
        if self.multi_step_config.is_none() {
          return Err("Multi-step mode requires multi_step_config".to_string());
        }
        if let Some(ref multi_step) = self.multi_step_config {
          if multi_step.operations.is_empty() {
            return Err("Multi-step config cannot be empty".to_string());
          }
          for (index, operation) in multi_step.operations.iter().enumerate() {
            operation.validate().map_err(|e| format!("Invalid operation at index {}: {}", index, e))?;
          }
        }
      }
    }

    // 验证质量参数
    if let Some(quality) = self.output_options.quality
      && quality > 100
    {
      return Err("Quality cannot exceed 100".to_string());
    }

    Ok(())
  }
}

/// Edit Image 节点实现
pub struct EditImageNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl EditImageNode {
  /// 创建新的 EditImage 节点
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(EditImageV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(EDIT_IMAGE_NODE_KIND, "Edit Image")
      .add_group(NodeGroupKind::Transform)
      .add_group(NodeGroupKind::Input)
      .add_group(NodeGroupKind::Output)
      .with_description(
        "Process and edit images with various operations including blur, border, crop, resize, rotate, text, and more.",
      )
      .with_icon("image")
  }
}

impl Node for EditImageNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

#[cfg(test)]
mod tests {
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};
  use serde_json::json;

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = EditImageNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), EDIT_IMAGE_NODE_KIND);
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input, NodeGroupKind::Output]);
    assert_eq!(&definition.display_name, "Edit Image");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = EditImageNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_image_operation_config_validation() {
    // 有效的模糊操作
    let valid_blur = ImageOperationConfig {
      operation: ImageOperation::Blur,
      parameters: json!({
        "blur": 2.0,
        "sigma": 1.0
      }),
      description: None,
    };
    assert!(valid_blur.validate().is_ok());

    // 无效的模糊操作（缺少参数）
    let invalid_blur = ImageOperationConfig {
      operation: ImageOperation::Blur,
      parameters: json!({
        "blur": 2.0
      }),
      description: None,
    };
    assert!(invalid_blur.validate().is_err());

    // 有效的创建操作
    let valid_create = ImageOperationConfig {
      operation: ImageOperation::Create,
      parameters: json!({
        "background_color": "#ffffff",
        "width": 800,
        "height": 600
      }),
      description: None,
    };
    assert!(valid_create.validate().is_ok());

    // 有效的信息操作（不需要参数）
    let valid_info =
      ImageOperationConfig { operation: ImageOperation::Information, parameters: json!({}), description: None };
    assert!(valid_info.validate().is_ok());
  }

  #[test]
  fn test_edit_image_config_validation() {
    // 有效的单操作配置
    let valid_single = EditImageConfig {
      operation_mode: ImageOperationMode::Single,
      data_property_name: "image".to_string(),
      output_options: ImageOutputOptions::default(),
      single_operation: Some(ImageOperationConfig {
        operation: ImageOperation::Resize,
        parameters: json!({
          "width": 800,
          "height": 600,
          "resize_option": "fit_inside"
        }),
        description: None,
      }),
      multi_step_config: None,
    };
    assert!(valid_single.validate().is_ok());

    // 无效的单操作配置（缺少操作）
    let invalid_single = EditImageConfig {
      operation_mode: ImageOperationMode::Single,
      data_property_name: "image".to_string(),
      output_options: ImageOutputOptions::default(),
      single_operation: None,
      multi_step_config: None,
    };
    assert!(invalid_single.validate().is_err());

    // 有效的多步骤配置
    let valid_multi_step = EditImageConfig {
      operation_mode: ImageOperationMode::MultiStep,
      data_property_name: "image".to_string(),
      output_options: ImageOutputOptions::default(),
      single_operation: None,
      multi_step_config: Some(MultiStepConfig {
        operations: vec![
          ImageOperationConfig {
            operation: ImageOperation::Resize,
            parameters: json!({
              "width": 800,
              "height": 600
            }),
            description: None,
          },
          ImageOperationConfig {
            operation: ImageOperation::Blur,
            parameters: json!({
              "blur": 2.0,
              "sigma": 1.0
            }),
            description: None,
          },
        ],
        stop_on_first_error: Some(false),
      }),
    };
    assert!(valid_multi_step.validate().is_ok());

    // 无效的多步骤配置（空操作列表）
    let invalid_multi_step = EditImageConfig {
      operation_mode: ImageOperationMode::MultiStep,
      data_property_name: "image".to_string(),
      output_options: ImageOutputOptions::default(),
      single_operation: None,
      multi_step_config: Some(MultiStepConfig { operations: vec![], stop_on_first_error: None }),
    };
    assert!(invalid_multi_step.validate().is_err());

    // 无效的数据属性名称
    let invalid_data_prop = EditImageConfig {
      operation_mode: ImageOperationMode::Single,
      data_property_name: "".to_string(),
      output_options: ImageOutputOptions::default(),
      single_operation: Some(ImageOperationConfig {
        operation: ImageOperation::Information,
        parameters: json!({}),
        description: None,
      }),
      multi_step_config: None,
    };
    assert!(invalid_data_prop.validate().is_err());

    // 无效的质量参数
    let invalid_quality = EditImageConfig {
      operation_mode: ImageOperationMode::Single,
      data_property_name: "image".to_string(),
      output_options: ImageOutputOptions {
        format: Some(ImageFormat::Jpeg),
        quality: Some(150), // 超过100
        file_name_template: None,
        preserve_metadata: None,
      },
      single_operation: Some(ImageOperationConfig {
        operation: ImageOperation::Information,
        parameters: json!({}),
        description: None,
      }),
      multi_step_config: None,
    };
    assert!(invalid_quality.validate().is_err());
  }

  #[test]
  fn test_serialization() {
    // 测试枚举序列化
    let operation = ImageOperation::Blur;
    let serialized = serde_json::to_string(&operation).unwrap();
    let deserialized: ImageOperation = serde_json::from_str(&serialized).unwrap();
    assert_eq!(operation, deserialized);

    let format = ImageFormat::Png;
    let serialized = serde_json::to_string(&format).unwrap();
    let deserialized: ImageFormat = serde_json::from_str(&serialized).unwrap();
    assert_eq!(format, deserialized);

    let mode = ImageOperationMode::MultiStep;
    let serialized = serde_json::to_string(&mode).unwrap();
    let deserialized: ImageOperationMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(mode, deserialized);
  }

  #[test]
  fn test_node_creation() {
    let node = EditImageNode::new();
    assert!(node.is_ok());

    let node = node.unwrap();
    assert_eq!(node.default_version().major, 1);
    assert_eq!(node.node_executors().len(), 1);
  }
}
