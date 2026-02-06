use std::f32::consts::PI;
use std::vec::Vec;

/// Supported sample rates for the sine wave generation.
static SUPPORTED_SAMPLE_RATES: [u32; 3] = [
    16_000, // 16 kHz is commonly used for speech and telephony applications
    44_100, // 44.1 kHz is the standard sample rate for audio CDs and is widely used in music production
    48_000, // 48 kHz is commonly used in professional audio and video production, as well as in some high-quality consumer audio formats
];

/// Duration of the sine wave in milliseconds.
static DURATION_MS: f32 = 1.0; // Duration of the sine wave in milliseconds

/// Audio sample width.
/// Stored in number of bytes per sample.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum SampleWidth {
    /// 16 bit audio
    Width2Byte = 2,
    /// 24 bit audio
    Width3Byte = 3,
    /// 32 bit audio
    Width4Byte = 4,
}

// Get the maximum absolute value for a given sample width.
// Digital Audio Representation:
/*
|----------|-----------------------------|------------------|
|  Format  |  	Integer Type             |   Max Positive   |
|----------|-----------------------------|------------------|
|  16-bit  |  	int16_t	                 |        32767     |
|  24-bit  |  	int32_t (in 24 bits)     |	    8,388,607   |
|  32-bit  |  	int32_t	                 | 	 2,147,483,647  |
|----------|-----------------------------|------------------|
 */
fn get_range(sample_width: SampleWidth) -> f32 {
    match sample_width {
        // 16 bit audio
        SampleWidth::Width2Byte => 32767.0,
        // 24 bit audio
        SampleWidth::Width3Byte => 8388607.0,
        // 32 bit audio
        SampleWidth::Width4Byte => 2147483647.0,
    }
}

// Create a sine wave audio buffer for a given frequency, sample rate, channel count, and sample width.
fn create_sine_array(
    freq: f32,
    sample_rate: f32,
    channel_count: u8,
    sample_width: SampleWidth,
    duration_ms: f32,
) -> Vec<u8> {
    // Calculate the phase increment for each sample and the total number of samples needed for the specified duration
    let mut phase: f32 = 0.0;
    let phase_inc: f32 = freq / sample_rate * 2.0 * PI; // Radians per sample
    let total_samples: usize = ((duration_ms * sample_rate) / 1000.0).round() as usize; // Number of samples in the specified duration
    let total_bytes: usize = total_samples * channel_count as usize * sample_width as usize;

    // Pre-allocate buffer with the total number of bytes needed for the sine wave
    let mut buffer = Vec::with_capacity(total_bytes);
    let max_value = get_range(sample_width);

    // Fill buffer with sine wave
    for _ in 0..total_samples {
        let sample = (phase.sin() * max_value) as i32;
        let bytes = sample.to_le_bytes();

        for _ in 0..channel_count {
            for n in 0..sample_width as usize {
                buffer.push(bytes[n]);
            }
        }

        // Keep phase reset when it exceeds 2PI to prevent discontinuities at the reset point
        phase = (phase + phase_inc) % (2.0 * PI);
    }
    buffer
}

fn main() {
    for &rate in SUPPORTED_SAMPLE_RATES.iter() {
        println!("\n====================================================================");
        println!(
            "Byte array for sample Rate: {} Hz ({} kHz), duration: {} ms",
            rate,
            rate as f32 / 1000.0,
            DURATION_MS
        );
        println!("--------------------------------------------------------------------");
        let buffer = create_sine_array(440.0, rate as f32, 2, SampleWidth::Width2Byte, DURATION_MS);

        let mut i: usize = 0;
        let bytes_in_line = 16;

        print!("[");
        while i < buffer.len() {
            buffer.iter().skip(i).take(bytes_in_line).for_each(|b| {
                print!("{:#02X}, ", b);
            });
            i += bytes_in_line;

            if i < buffer.len() {
                print!("\n");
            }
        }
        print!("]\n");
    }
}
