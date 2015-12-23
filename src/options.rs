use std::ops::Deref;

pub enum ColorType {
    Rgba8,
    Rgb5a3,
}

pub fn color_type(input: Option<String>) -> Result<ColorType, String> {
    match input {
        Some(string) => {
            let colortype = string.to_uppercase();
            match colortype.deref() {
                "RGBA8" => Ok(ColorType::Rgba8),
                "RGB5A3" => Ok(ColorType::Rgb5a3),
                _ => Err(format!("Unknown color type {}", string)),
            }
        }
        None => Ok(ColorType::Rgba8),
    }
}
