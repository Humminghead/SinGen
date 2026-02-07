use std::env;
use std::f32::consts::PI;
use std::process;

static SUPPORTED_SAMPLE_RATES: [u32; 3] = [
    16_000, // 16 kHz is commonly used for speech and telephony applications
    44_100, // 44.1 kHz is the standard sample rate for audio CDs and is widely used in music production
    48_000, // 48 kHz is commonly used in professional audio and video production, as well as in some high-quality consumer audio formats
];

/// Audio sample width.
///
/// Stored in number of bytes per sample.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SampleWidth {
    /// 16 bit audio
    Width2Byte = 2,
    /// 24 bit audio
    Width3Byte = 3,
    /// 32 bit audio
    Width4Byte = 4,
}

impl SampleWidth {
    /// Parse from string (16, 24, 32)
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "16" => Some(SampleWidth::Width2Byte),
            "24" => Some(SampleWidth::Width3Byte),
            "32" => Some(SampleWidth::Width4Byte),
            _ => None,
        }
    }

    /// Get string representation
    fn to_str(&self) -> &'static str {
        match self {
            SampleWidth::Width2Byte => "16",
            SampleWidth::Width3Byte => "24",
            SampleWidth::Width4Byte => "32",
        }
    }
}

// Get the maximum absolute value for a given sample width.
// Digital Audio Representation:
/*
|----------|-----------------------------|------------------|
|  Format  |   Integer Type             |   Max Positive   |
|----------|-----------------------------|------------------|
|  16-bit  |  int16_t                   |        32767     |
|  24-bit  |  int32_t (in 24 bits)      |     8,388,607    |
|  32-bit  |  int32_t                   |  2,147,483,647   |
|----------|-----------------------------|------------------|
*/
fn get_range(sample_width: SampleWidth) -> f32 {
    match sample_width {
        SampleWidth::Width2Byte => 32767.0,
        SampleWidth::Width3Byte => 8388607.0,
        SampleWidth::Width4Byte => 2147483647.0,
    }
}

struct Config {
    frequency: f32,
    sample_rate: u32,
    channels: u8,
    sample_width: SampleWidth,
    duration_ms: f32,
    output_format: OutputFormat,
    analyze_only: bool,
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Hex,
    CArray,
    RustArray,
    RawBytes,
    Info,
}

impl OutputFormat {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hex" => Some(OutputFormat::Hex),
            "carray" | "c" => Some(OutputFormat::CArray),
            "rustarray" | "rust" => Some(OutputFormat::RustArray),
            "raw" | "bytes" => Some(OutputFormat::RawBytes),
            "info" => Some(OutputFormat::Info),
            _ => None,
        }
    }
}

fn print_usage() {    
    println!("Usage: singen [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -f, --frequency FREQ     Sine wave frequency in Hz (default: 440.0)");
    println!("  -r, --rate RATE          Sample rate in Hz (default: 16000)");
    println!("                           Supported: 16000, 44100, 48000");
    println!("  -c, --channels CH        Number of channels (1=mono, 2=stereo, default: 2)");
    println!("  -b, --bits BITS          Bit depth: 16, 24, or 32 (default: 16)");
    println!("  -d, --duration MS        Duration in milliseconds (default: 1.0)");
    println!("  -o, --output FORMAT      Output format:");
    println!("                           hex      - Hexadecimal values (default)");
    println!("                           carray   - C-style array declaration");
    println!("                           rustarray - Rust array declaration");
    println!("                           raw      - Raw binary bytes (stdout)");
    println!("                           info     - Only show buffer info, no data");
    println!("  -a, --analyze            Analyze only (don't generate data)");
    println!("  -h, --help               Show this help message");
    println!();
    println!("Examples:");
    println!("  singen -f 1000 -r 48000 -b 16 -d 10 -o carray");
    println!("  singen --frequency 440 --rate 44100 --channels 1 --bits 24");
    println!("  singen -r 16000 -d 1 -o rustarray -p");
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut config = Config {
        frequency: 440.0,
        sample_rate: 16_000,
        channels: 2,
        sample_width: SampleWidth::Width2Byte,
        duration_ms: 1.0,
        output_format: OutputFormat::Hex,
        analyze_only: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-f" | "--frequency" => {
                i += 1;
                if i < args.len() {
                    config.frequency = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid frequency value");
                        process::exit(1);
                    });
                }
            }
            "-r" | "--rate" => {
                i += 1;
                if i < args.len() {
                    config.sample_rate = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid sample rate");
                        process::exit(1);
                    });
                    if !SUPPORTED_SAMPLE_RATES.contains(&config.sample_rate) {
                        eprintln!(
                            "Warning: {} Hz is not in the standard supported rates list",
                            config.sample_rate
                        );
                    }
                }
            }
            "-c" | "--channels" => {
                i += 1;
                if i < args.len() {
                    let ch = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid channel count");
                        process::exit(1);
                    });
                    if ch != 1 && ch != 2 {
                        eprintln!("Error: Channel count must be 1 or 2");
                        process::exit(1);
                    }
                    config.channels = ch;
                }
            }
            "-b" | "--bits" => {
                i += 1;
                if i < args.len() {
                    config.sample_width = SampleWidth::from_str(&args[i]).unwrap_or_else(|| {
                        eprintln!("Error: Invalid bit depth. Must be 16, 24, or 32");
                        process::exit(1);
                    });
                }
            }
            "-d" | "--duration" => {
                i += 1;
                if i < args.len() {
                    config.duration_ms = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Error: Invalid duration");
                        process::exit(1);
                    });
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    config.output_format = OutputFormat::from_str(&args[i]).unwrap_or_else(|| {
                        eprintln!("Error: Invalid output format");
                        process::exit(1);
                    });
                }
            }
            "-a" | "--analyze" => {
                config.analyze_only = true;
                config.output_format = OutputFormat::Info;
            }
            _ => {
                eprintln!("Error: Unknown option: {}", args[i]);
                print_usage();
                process::exit(1);
            }
        }
        i += 1;
    }

    config
}

// Create a sine wave audio buffer for a given frequency, sample rate, channel count, and sample width.
fn create_sine_array(
    freq: f32,
    sample_rate: f32,
    channel_count: u8,
    sample_width: SampleWidth,
    duration_ms: f32,
) -> (Vec<u8>, usize, usize) {
    // Calculate the phase increment for each sample and the total number of samples needed for the specified duration
    let mut phase: f32 = 0.0;
    let phase_inc: f32 = freq / sample_rate * 2.0 * PI; // Radians per sample
    let total_samples: usize = ((duration_ms * sample_rate) / 1000.0).round() as usize; // Number of samples in the specified duration

    // If USB packet mode, adjust to fit 64-byte packets
    let bytes_per_sample = sample_width as usize;
    let bytes_per_frame = bytes_per_sample * channel_count as usize;
    let total_bytes = total_samples * bytes_per_frame;

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

    (buffer, total_samples, total_bytes)
}

fn print_buffer_info(config: &Config, total_samples: usize, total_bytes: usize) {
    println!("Sine Wave Generator - Configuration");
    println!("=====================================");
    println!("Frequency:      {} Hz", config.frequency);
    println!("Sample Rate:    {} Hz", config.sample_rate);
    println!(
        "Channels:       {} ({})",
        config.channels,
        if config.channels == 1 {
            "mono"
        } else {
            "stereo"
        }
    );
    println!("Bit Depth:      {}-bit", config.sample_width.to_str());
    println!("Duration:       {} ms", config.duration_ms);
    println!();
    println!("Buffer Analysis:");
    println!("  Samples:      {}", total_samples);
    println!("  Total bytes:  {}", total_bytes);

    // Calculate frequency info
    let period_samples = config.sample_rate as f32 / config.frequency;
    println!("\nFrequency Analysis:");
    println!("  Period:       {:.2} samples", period_samples);
    println!(
        "  Full cycles:  {:.2}",
        total_samples as f32 / period_samples
    );        
}

fn print_buffer_hex(buffer: &[u8], bytes_per_line: usize) {
    let mut i: usize = 0;
    print!("[");
    while i < buffer.len() {
        if i > 0 {
            print!("\n ");
        }
        for j in 0..bytes_per_line {
            if i + j < buffer.len() {
                print!("0x{:02X}", buffer[i + j]);
                if i + j + 1 < buffer.len() && j < bytes_per_line - 1 {
                    print!(", ");
                }
            }
        }
        i += bytes_per_line;
    }
    println!("]");
}

fn print_c_array(buffer: &[u8], config: &Config) {
    let name = format!(
        "sine_{}hz_{}ms_{}bit_{}ch",
        config.sample_rate,
        config.duration_ms as u32,
        config.sample_width.to_str(),
        config.channels
    );

    println!(
        "// Sine wave: {} Hz, {} ms, {}-bit, {} channel{}",
        config.frequency,
        config.duration_ms,
        config.sample_width.to_str(),
        config.channels,
        if config.channels > 1 { "s" } else { "" }
    );
    println!("// Sample rate: {} Hz", config.sample_rate);
    println!("// Total bytes: {}", buffer.len());
    println!(
        "const uint8_t {}[{}] = {{",
        name.to_uppercase(),
        buffer.len()
    );

    for (i, chunk) in buffer.chunks(16).enumerate() {
        print!("    ");
        for (j, byte) in chunk.iter().enumerate() {
            print!("0x{:02X}", byte);
            if i * 16 + j < buffer.len() - 1 {
                print!(", ");
            }
        }
        if i * 16 < buffer.len() {
            println!();
        }
    }
    println!("}};");
}

fn print_rust_array(buffer: &[u8], config: &Config) {
    let name = format!(
        "SINE_{}HZ_{}MS_{}BIT_{}CH",
        config.sample_rate,
        config.duration_ms as u32,
        config.sample_width.to_str(),
        config.channels
    );

    println!(
        "// Sine wave: {} Hz, {} ms, {}-bit, {} channel{}",
        config.frequency,
        config.duration_ms,
        config.sample_width.to_str(),
        config.channels,
        if config.channels > 1 { "s" } else { "" }
    );
    println!("// Sample rate: {} Hz", config.sample_rate);
    println!("// Total bytes: {}", buffer.len());
    println!("pub const {}: [u8; {}] = [", name, buffer.len());

    for (i, chunk) in buffer.chunks(16).enumerate() {
        print!("    ");
        for (j, byte) in chunk.iter().enumerate() {
            print!("0x{:02X}", byte);
            if i * 16 + j < buffer.len() - 1 {
                print!(", ");
            }
        }
        if i * 16 < buffer.len() {
            println!();
        }
    }
    println!("];");
}

fn print_raw_bytes(buffer: &[u8]) {
    use std::io::{self, Write};
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(buffer).unwrap();
}

fn main() {
    let config = parse_args();

    let (buffer, total_samples, total_bytes) = create_sine_array(
        config.frequency,
        config.sample_rate as f32,
        config.channels,
        config.sample_width,
        config.duration_ms,
    );

    match config.output_format {
        OutputFormat::Info => {
            print_buffer_info(&config, total_samples, total_bytes);
        }
        OutputFormat::Hex => {
            print_buffer_info(&config, total_samples, total_bytes);
            println!("\nBuffer data (hexadecimal):");
            print_buffer_hex(&buffer, 16);
        }
        OutputFormat::CArray => {
            print_buffer_info(&config, total_samples, total_bytes);
            println!("\nC array declaration:");
            print_c_array(&buffer, &config);
        }
        OutputFormat::RustArray => {
            print_buffer_info(&config, total_samples, total_bytes);
            println!("\nRust array declaration:");
            print_rust_array(&buffer, &config);
        }
        OutputFormat::RawBytes => {
            print_raw_bytes(&buffer);
        }
    }
}
