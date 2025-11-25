//! Example of using OpenAI compatible image edit API
//!
//! This example demonstrates how to use the image editing functionality
//! with OpenAI or compatible services (like Gitee AI).
//!
//! ## Supported Models:
//! - OpenAI: gpt-image-1 (full features, including multi-image up to 16 images)
//! - Gitee AI: qwen-image-edit, HiDream-E1-Full, etc. (OpenAI compatible)
//! - Other: DALL-E 2 (single image with mask)
//!
//! ## Setup:
//! 1. Set your API key in environment variable:
//!    - `export GITEE_AI_API_KEY="your-api-key"`
//!    - Or `export OPENAI_API_KEY="your-openai-api-key"`
//!
//! 2. Ensure you have a test image file named `test_image.png` in the current directory,
//!    or change the `image_path` in the configuration below.
//!
//! ## How to get test images:
//! ```bash
//! # Option 1: Create a simple test image (with Pillow)
//! python3 -c "from PIL import Image, ImageDraw; img = Image.new('RGB', (512, 512), 'white'); draw = ImageDraw.Draw(img); draw.rectangle([100, 100, 412, 412], fill='lightblue'); draw.ellipse([200, 200, 312, 312], fill='yellow'); img.save('test_image.png')"
//!
//! # Option 2: Use uv with Pillow
//! uv run python -c "from PIL import Image, ImageDraw; img = Image.new('RGB', (512, 512), 'white'); draw = ImageDraw.Draw(img); draw.rectangle([100, 100, 412, 412], fill='lightblue'); draw.ellipse([200, 200, 312, 312], fill='yellow'); img.save('test_image.png')"
//!
//! # Option 3: Copy an existing image
//! cp /path/to/your/image.png test_image.png
//! ```
//!
//! ## For multi-image examples (Example 3):
//! Create additional test images:
//! ```bash
//! python3 -c "from PIL import Image, ImageDraw; img = Image.new('RGB', (512, 512), 'white'); draw = ImageDraw.Draw(img); draw.rectangle([100, 100, 412, 412], fill='lightgreen'); img.save('test_image2.png')"
//! python3 -c "from PIL import Image, ImageDraw; img = Image.new('RGB', (512, 512), 'white'); draw = ImageDraw.Draw(img); draw.rectangle([100, 100, 412, 412], fill='lightcoral'); img.save('test_image3.png')"
//! ```
//!
//! ## Run:
//! ```bash
//! cargo run -p fusion-ai --example image_edit_demo
//! ```

use fusion_ai::providers::openai_compatible::{Client, image_edit::ImageEditRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv().unwrap();
  // Configuration - Edit these values as needed
  let config = ImageEditConfig {
    base_url: "https://ai.gitee.com/v1", // Use OpenAI: "https://api.openai.com/v1"
    api_key: env::var("GITEE_AI_API_KEY").expect("Please set GITEE_AI_API_KEY"),
    model: "qwen-image-edit", // For OpenAI: "gpt-image-1"
    image_path: "test_image.png".to_string(),
    output_path: "edited_image.png".to_string(),
  };

  println!("=== Image Edit Demo ===");
  println!("Provider: {}", config.base_url);
  println!("Model: {}", config.model);
  println!("======================\n");

  // Example 1: Basic image edit with detailed prompt
  basic_image_edit(&config).await?;

  // Example 2: Change style and add elements
  println!("\n");
  style_transfer_edit(&config).await?;

  // Example 3: Multi-image editing (gpt-image-1 only - requires multiple test images)
  println!("\n");
  match multi_image_edit(&config).await {
    Ok(_) => println!("✓ Multi-image example completed successfully"),
    Err(e) => println!("⚠ Multi-image example skipped or failed: {}", e),
  }

  Ok(())
}

/// Configuration for image editing
struct ImageEditConfig {
  base_url: &'static str,
  api_key: String,
  model: &'static str,
  image_path: String,
  output_path: String,
}

/// Example 1: Basic image edit - modify specific elements
async fn basic_image_edit(config: &ImageEditConfig) -> Result<(), Box<dyn std::error::Error>> {
  println!("=== Example 1: Basic Image Edit ===");

  // Create client
  let client = Client::builder(&config.api_key).base_url(config.base_url).build();

  let model = client.image_edit_model(config.model);

  // Load image as raw bytes
  let image_data = load_image_bytes(&config.image_path)?;

  // Create request using the builder pattern
  let request = ImageEditRequest::new(
    image_data,
    "Change the background to a beautiful beach at sunset, keep the main subject unchanged".to_string(),
    "1024x1024".to_string(),
  )
  .with_n(1);

  println!("Generating edited image...");
  let response = model.image_edit(request).await?;

  // Save the edited image
  std::fs::write(&config.output_path, response.image)?;
  println!("✓ Edited image saved to: {}", config.output_path);
  println!("  Created at: {}", response.response.created);

  Ok(())
}

/// Example 2: Style transfer - transform image style
async fn style_transfer_edit(config: &ImageEditConfig) -> Result<(), Box<dyn std::error::Error>> {
  println!("=== Example 2: Style Transfer ===");

  let client = Client::builder(&config.api_key).base_url(config.base_url).build();

  let model = client.image_edit_model(config.model);

  let image_data = load_image_bytes(&config.image_path)?;

  // Create request with optional parameters
  let request = ImageEditRequest::new(
    image_data,
    "Transform this image into a cyberpunk style with neon lights, futuristic elements, and dramatic lighting"
      .to_string(),
    "1024x1024".to_string(),
  )
  .with_n(1);

  println!("Applying style transformation...");
  let response = model.image_edit(request).await?;

  let output_path = "styled_image.png";
  std::fs::write(output_path, response.image)?;
  println!("✓ Styled image saved to: {}", output_path);

  Ok(())
}

/// Example 3: Multi-image editing - combine multiple images into one (gpt-image-1 only)
///
/// This example demonstrates how to upload multiple images (up to 16 for gpt-image-1).
/// The API will combine them based on the prompt.
async fn multi_image_edit(config: &ImageEditConfig) -> Result<(), Box<dyn std::error::Error>> {
  println!("=== Example 3: Multi-Image Edit ===");
  println!("Note: This example requires multiple test images and only works with gpt-image-1\n");

  // Check if additional test images exist
  let image_paths = vec!["test_image.png", "test_image2.png", "test_image3.png"];
  let mut images = Vec::new();

  for path in &image_paths {
    if let Ok(data) = std::fs::read(path) {
      images.push(data);
      println!("✓ Loaded: {}", path);
    } else {
      println!("⚠ Skipping: {} (file not found)", path);
    }
  }

  if images.is_empty() {
    return Err("No test images found. Please create at least test_image.png".into());
  }

  if images.len() == 1 {
    println!("\nOnly one image found. Multi-image example requires multiple images.");
    println!("To create more test images:");
    println!(
      "  python3 -c \"from PIL import Image; Image.new('RGB', (512, 512), 'lightgreen').save('test_image2.png')\""
    );
    println!(
      "  python3 -c \"from PIL import Image; Image.new('RGB', (512, 512), 'lightcoral').save('test_image3.png')\""
    );
    return Ok(());
  }

  println!("\nUploading {} images to create a collage...", images.len());

  // For OpenAI gpt-image-1, use the appropriate model
  // For Gitee AI, multi-image support depends on the specific model
  let client = Client::builder(&config.api_key).base_url(config.base_url).build();

  // Try with gpt-image-1 if using OpenAI endpoint
  let model_name = if config.base_url.contains("openai.com") {
    "gpt-image-1"
  } else {
    config.model // Use the configured model for Gitee AI
  };

  let model = client.image_edit_model(model_name);

  let request = ImageEditRequest::new_with_images(
    images,
    "Create a beautiful collage arrangement with these images, blending them harmoniously".to_string(),
    "1536x1024".to_string(), // Landscape orientation for collage
  )
  .with_n(1)
  .with_quality("high".to_string());

  match model.image_edit(request).await {
    Ok(response) => {
      std::fs::write("multi_image_collage.png", response.image)?;
      println!("✓ Multi-image collage saved to: multi_image_collage.png");

      if let Some(usage) = response.response.usage {
        println!(
          "  Token usage: {} total ({} input, {} output)",
          usage.total_tokens, usage.input_tokens, usage.output_tokens
        );
      }
    }
    Err(e) => {
      println!("⚠ Multi-image edit not supported by this model: {}", e);
      println!("  Try with OpenAI gpt-image-1 for multi-image support");
    }
  }

  Ok(())
}

/// Example 3: Advanced options (gpt-image-1 specific features)
#[allow(dead_code)]
async fn advanced_edit_example(config: &ImageEditConfig) -> Result<(), Box<dyn std::error::Error>> {
  let client = Client::builder(&config.api_key).base_url(config.base_url).build();
  let model = client.image_edit_model(config.model);

  let image_data = load_image_bytes(&config.image_path)?;

  // Create request with all optional parameters (gpt-image-1 only)
  let request = ImageEditRequest::new(
    image_data,
    "Remove the background and enhance the main subject with ultra high quality".to_string(),
    "1024x1024".to_string(),
  )
  .with_n(2)
  .with_quality("high".to_string())
  .with_background("transparent".to_string())
  .with_output_format("png".to_string())
  .with_user("user_12345".to_string());

  println!("Generating with advanced options...");
  let response = model.image_edit(request).await?;

  if let Some(usage) = response.response.usage {
    println!(
      "Token usage: {} total ({} input, {} output)",
      usage.total_tokens, usage.input_tokens, usage.output_tokens
    );
    println!(
      "Image tokens: {}, Text tokens: {}",
      usage.input_tokens_details.image_tokens, usage.input_tokens_details.text_tokens
    );
  }

  Ok(())
}

/// Helper function: Load image from file and return raw bytes
fn load_image_bytes(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  // Check if file exists
  if !std::path::Path::new(path).exists() {
    eprintln!("Error: Image file not found: {}", path);
    eprintln!("\nPlease create a test image or download one.");
    eprintln!("\nYou can create a simple test image with:");
    eprintln!(
      "  python3 -c \"from PIL import Image, ImageDraw; img = Image.new('RGB', (512, 512), 'white'); draw = ImageDraw.Draw(img); draw.rectangle([100, 100, 412, 412], fill='lightblue'); draw.ellipse([200, 200, 312, 312], fill='yellow'); img.save('test_image.png')\""
    );
    eprintln!("  or");
    eprintln!("  curl -o test_image.png https://example.com/sample-image.png");
    std::process::exit(1);
  }

  let image_data = std::fs::read(path)?;
  Ok(image_data)
}
