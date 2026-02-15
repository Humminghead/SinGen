use std::env;
use std::f32::consts::TAU;
use std::io::Write;
use std::process;
use std::vec::Vec;

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

// https://ccrma.stanford.edu/courses/422-winter-2014/projects/WaveFormat/
#[repr(C, packed)]
#[allow(dead_code)]
struct WavHeader {
    chunk_id: [u8; 4],      // 0
    chunk_size: u32,        //4
    format: [u8; 4],        //8
    subchunk_1_id: [u8; 4], //12
    subchunk_1_size: u32,   // 16
    audio_format: u16,      // 20
    num_channels: u16,      // 22
    sample_rate: u32,       // 24
    byte_rate: u32,         // 28
    block_align: u16,       // 32
    bits_per_sample: u16,   // 34
    subchunk_2_id: [u8; 4], //36
    subchunk_2_size: u32,   //40
}

impl WavHeader {
    pub fn new() -> Self {
        Self {
            chunk_id: *b"RIFF",
            chunk_size: 0,
            format: *b"WAVE",
            subchunk_1_id: *b"fmt ",
            subchunk_1_size: 16,
            audio_format: 0x0001, //WINDOWS PCM
            num_channels: 1,
            sample_rate: 44_100,
            byte_rate: 176_400,
            block_align: 2,
            bits_per_sample: 16,
            subchunk_2_id: *b"data",
            subchunk_2_size: 0,
        }
    }
}

// Get the maximum absolute value for a given sample width.
// Digital Audio Representation:
/*
|----------|-----------------------------|------------------|
|  Format  |   Integer Type              |   Max Positive   |
|----------|-----------------------------|------------------|
|  16-bit  |  int16_t                    |        32767     |
|  24-bit  |  int32_t (in 24 bits)       |     8,388,607    |
|  32-bit  |  int32_t                    |  2,147,483,647   |
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
    WavFile,
}

impl OutputFormat {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hex" => Some(OutputFormat::Hex),
            "carray" | "c" => Some(OutputFormat::CArray),
            "rustarray" | "rust" => Some(OutputFormat::RustArray),
            "raw" | "bytes" => Some(OutputFormat::RawBytes),
            "info" => Some(OutputFormat::Info),
            "wav" => Some(OutputFormat::WavFile),
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
    println!("                           wav      - Windows audio file format (stdout)");
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

/// Generate a linear chirp from `f0` Hz to `f1` Hz over `duration_secs`.
/// Returns a vector of floating‑point samples in the range [-1.0, 1.0].
fn generate_linear_chirp(
    f0: f32,            // start frequency (Hz)
    f1: f32,            // end frequency (Hz)
    sample_rate: f32,   // samples per second
    duration_secs: f32, // total duration in seconds
) -> Vec<f32> {
    let dt = 1.0 / sample_rate;
    let num_samples = (duration_secs * sample_rate).round() as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut phase = 0.0;

    for i in 0..num_samples {
        let t = i as f32 * dt;
        // Instantaneous frequency at time t (linear interpolation)
        let freq = f0 + (f1 - f0) * (t / duration_secs);
        // Phase increment for this sample
        phase += TAU * freq * dt;
        // Keep phase in [-π, π] range to avoid floating-point drift (optional)
        phase = phase.rem_euclid(TAU);
        samples.push(phase.sin());
    }

    samples
}

fn float_samples_to_bytes(samples: &[f32], channels: u8, sample_width: SampleWidth) -> Vec<u8> {
    let max_val = get_range(sample_width);
    let mut buffer = Vec::with_capacity(samples.len() * channels as usize * sample_width as usize);

    for &sample in samples {
        let scaled = (sample * max_val).round() as i32;
        let bytes = scaled.to_le_bytes();
        for _ in 0..channels {
            for b in &bytes[0..sample_width as usize] {
                buffer.push(*b);
            }
        }
    }
    buffer
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

fn create_wav_file_array(
    buffer: &[u8],
    sample_rate: u32,
    channels: u16,
    sample_width: SampleWidth,
) -> Vec<u8> {
    let wav_header_len = std::mem::size_of::<WavHeader>();
    let buffer_len = buffer.len();

    let mut wav_hdr = WavHeader::new();
    wav_hdr.chunk_size = (36 + buffer_len) as u32; // 4 + (24) + 8 + buffer_len
    wav_hdr.num_channels = channels;
    wav_hdr.sample_rate = sample_rate;
    wav_hdr.byte_rate = sample_rate as u32 * channels as u32 * sample_width as u32;
    wav_hdr.block_align = channels * sample_width as u16; // fixed formula
    wav_hdr.bits_per_sample = sample_width as u16 * 8;
    wav_hdr.subchunk_2_size = buffer_len as u32;

    let mut file = Vec::with_capacity(wav_header_len + buffer_len);
    let ptr = &wav_hdr as *const WavHeader as *const u8;
    // SAFETY: WavHeader is repr(C, packed) so it has no padding.
    file.write_all(unsafe { std::slice::from_raw_parts(ptr, wav_header_len) })
        .unwrap();
    file.write_all(buffer).unwrap();
    file
}

fn main() {
    let config = parse_args();

    let total_samples =
        ((config.duration_ms * config.sample_rate as f32) / 1000.0).round() as usize;
    let total_bytes = total_samples * (config.sample_width as u8 * config.channels) as usize;

    let float_samples = generate_linear_chirp(
        config.frequency,
        config.frequency,
        config.sample_rate as f32,
        config.duration_ms / 1000.0,
    );
    let buffer = float_samples_to_bytes(&float_samples, config.channels, config.sample_width);

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
        OutputFormat::WavFile => {
            let file = create_wav_file_array(
                &buffer,
                config.sample_rate,
                config.channels as u16,
                config.sample_width,
            );
            print_raw_bytes(file.as_ref());
        }
    }
}
