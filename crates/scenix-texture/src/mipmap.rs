use alloc::vec;
use alloc::vec::Vec;

use scenix_core::ValidationError;

/// Generates an RGBA8 mip chain including the base level.
pub fn generate(data: &[u8], width: u32, height: u32) -> Result<Vec<Vec<u8>>, ValidationError> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|value| value.checked_mul(4))
        .ok_or(ValidationError::OutOfRange)?;
    if width == 0 || height == 0 || data.len() != expected {
        return Err(ValidationError::OutOfRange);
    }

    let mut levels = Vec::new();
    levels.push(data.to_vec());
    let mut current_width = width;
    let mut current_height = height;

    while current_width > 1 || current_height > 1 {
        let current = &levels[levels.len() - 1];
        let next_width = (current_width / 2).max(1);
        let next_height = (current_height / 2).max(1);
        let mut next = vec![0_u8; next_width as usize * next_height as usize * 4];

        for y in 0..next_height {
            for x in 0..next_width {
                let mut sum = [0_u32; 4];
                let mut count = 0_u32;
                for oy in 0..2 {
                    for ox in 0..2 {
                        let sx = (x * 2 + ox).min(current_width - 1);
                        let sy = (y * 2 + oy).min(current_height - 1);
                        let index = ((sy * current_width + sx) * 4) as usize;
                        for channel in 0..4 {
                            sum[channel] += current[index + channel] as u32;
                        }
                        count += 1;
                    }
                }

                let out = ((y * next_width + x) * 4) as usize;
                for channel in 0..4 {
                    next[out + channel] = (sum[channel] / count) as u8;
                }
            }
        }

        levels.push(next);
        current_width = next_width;
        current_height = next_height;
    }

    Ok(levels)
}
