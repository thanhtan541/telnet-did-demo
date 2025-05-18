use image::Luma;
use qrcode::{render::unicode, QrCode};
use std::path::Path;

/// Generates a QR code from the input string and saves it as a PNG file.
///
/// # Arguments
/// * `data` - The string to encode in the QR code.
/// * `output_path` - The file path where the QR code PNG will be saved.
///
/// # Returns
/// * `Result<(), String>` - Ok if successful, or an error message if the operation fails.
pub fn generate_qr_code(data: &str, output_path: &str) -> Result<(), String> {
    if data.is_empty() {
        return Err("Data is empty".into());
    }

    // Create QR code
    let code =
        QrCode::new(data.as_bytes()).map_err(|e| format!("Failed to create QR code: {}", e))?;

    // Render QR code to image
    let image = code.render::<Luma<u8>>().module_dimensions(10, 10).build();

    // Create output file
    let path = Path::new(output_path);

    // Save image as PNG
    image
        .save(path)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(())
}

/// Prints a QR code to the terminal as ASCII art.
///
/// # Arguments
/// * `data` - The string to encode in the QR code.
///
/// # Returns
/// * `Result<String, String>` - The ASCII representation of the QR code if successful, or an error message.
pub fn print_qr_code(data: &str) -> Result<String, String> {
    if data.is_empty() {
        return Err("Data is empty".into());
    }
    // Create QR code
    let code =
        QrCode::new(data.as_bytes()).map_err(|e| format!("Failed to create QR code: {}", e))?;

    // Render QR code as ASCII (unicode blocks)
    let qr_string = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    Ok(qr_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_generate_qr_code_success() {
        let test_data = "https://example.com";
        let test_output = "test_qr.png";

        // Generate QR code
        let result = generate_qr_code(test_data, test_output);
        assert!(result.is_ok());

        // Check if file exists
        assert!(Path::new(test_output).exists());

        // Clean up
        fs::remove_file(test_output).unwrap();
    }

    #[test]
    fn test_generate_qr_code_empty_input() {
        let test_data = "";
        let test_output = "test_empty_qr_1.png";

        // Attempt to generate QR code with empty input
        let result = generate_qr_code(test_data, test_output);
        assert!(result.is_err());

        // Verify file was not created
        assert!(!Path::new(test_output).exists());
    }

    #[test]
    fn test_generate_qr_code_invalid_path() {
        let test_data = "https://example.com";
        let test_output = "/invalid/path/test_qr.png";

        // Attempt to generate QR code with invalid path
        let result = generate_qr_code(test_data, test_output);
        assert!(result.is_err());

        // Verify file was not created
        assert!(!Path::new(test_output).exists());
    }

    #[test]
    fn test_print_qr_code_success() {
        let test_data = "https://example.com";

        // Generate QR code for terminal
        let result = print_qr_code(test_data);
        assert!(result.is_ok());

        // Check if the output contains expected QR code characters
        let qr_string = result.unwrap();
        assert!(qr_string.contains('█'));
        assert!(qr_string.contains(' '));
        assert!(!qr_string.is_empty());
    }

    #[test]
    fn test_print_qr_code_empty_input() {
        let test_data = "";

        // Attempt to generate QR code with empty input
        let result = print_qr_code(test_data);
        assert!(result.is_err());
    }
}
