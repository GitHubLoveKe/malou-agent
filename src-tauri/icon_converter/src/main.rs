use image::{ImageFormat, RgbaImage};
use std::fs::File;
use std::io::Write;

fn jpg_to_ico(jpg_path: &str, ico_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 读取JPG图片
    let img = image::open(jpg_path)?;
    
    // 调整大小到合适的图标尺寸
    let sizes = vec![(16, 16), (32, 32), (48, 48)];
    let mut ico_data = Vec::new();
    
    // ICO文件头
    ico_data.extend_from_slice(&[0u8, 0, 1, 0]); // Reserved, Type
    ico_data.extend_from_slice(&(sizes.len() as u16).to_le_bytes()); // Number of images
    
    let mut offset = 6 + sizes.len() * 16; // Header + directory entries
    
    // 为每个尺寸创建目录条目
    for &(width, height) in &sizes {
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let rgba_img = resized.to_rgba8();
        
        // 计算图像数据大小
        let data_size = rgba_img.width() * rgba_img.height() * 4 + 40; // BMP header size
        
        // 目录条目
        ico_data.push(width as u8);  // Width
        ico_data.push(height as u8); // Height
        ico_data.push(0);            // Color palette
        ico_data.push(0);            // Reserved
        ico_data.extend_from_slice(&[1u8, 0]); // Color planes
        ico_data.extend_from_slice(&[32u8, 0]); // Bits per pixel
        ico_data.extend_from_slice(&(data_size as u32).to_le_bytes()); // Image size
        ico_data.extend_from_slice(&(offset as u32).to_le_bytes());    // Image offset
        
        offset += data_size as usize;
    }
    
    // 为每个尺寸添加图像数据
    for &(width, height) in &sizes {
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let rgba_img = resized.to_rgba8();
        
        // BMP header (40 bytes)
        let mut bmp_header = Vec::new();
        bmp_header.extend_from_slice(&[40u8, 0, 0, 0]); // Header size
        bmp_header.extend_from_slice(&(width as i32).to_le_bytes());  // Width
        bmp_header.extend_from_slice(&((height as i32) * 2).to_le_bytes()); // Height (doubled for AND mask)
        bmp_header.extend_from_slice(&[1u8, 0]);        // Planes
        bmp_header.extend_from_slice(&[32u8, 0]);       // Bits per pixel
        bmp_header.extend_from_slice(&[0u8; 4]);        // Compression
        bmp_header.extend_from_slice(&[0u8; 4]);        // Image size
        bmp_header.extend_from_slice(&[0u8; 4]);        // X pixels per meter
        bmp_header.extend_from_slice(&[0u8; 4]);        // Y pixels per meter
        bmp_header.extend_from_slice(&[0u8; 4]);        // Colors used
        bmp_header.extend_from_slice(&[0u8; 4]);        // Important colors
        
        ico_data.extend_from_slice(&bmp_header);
        
        // 图像数据 (bottom-up BMP format)
        for y in (0..height).rev() {
            for x in 0..width {
                let pixel = rgba_img.get_pixel(x, y);
                let rgba = pixel.0;
                ico_data.extend_from_slice(&[rgba[2], rgba[1], rgba[0], rgba[3]]); // BGRA format
            }
        }
        
        // AND mask (all zeros for full opacity)
        let mask_size = ((width + 31) / 32) * 4 * height;
        ico_data.extend_from_slice(&vec![0u8; mask_size]);
    }
    
    // 写入ICO文件
    let mut file = File::create(ico_path)?;
    file.write_all(&ico_data)?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let jpg_path = "../../malou.jpg";
    let ico_path = "../icons/icon.ico";
    
    // 创建图标目录
    std::fs::create_dir_all("../icons")?;
    
    // 转换图像
    jpg_to_ico(jpg_path, ico_path)?;
    println!("成功将 {} 转换为 {}", jpg_path, ico_path);
    
    Ok(())
}