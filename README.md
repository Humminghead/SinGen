# Sine Wave Array Generator

A command-line tool to generate sine wave audio buffers.

## Features

- **Multiple Sample Rates**: 16kHz, 44.1kHz, 48kHz (and custom)
- **Bit Depths**: 16-bit, 24-bit, 32-bit audio
- **Channel Configurations**: Mono (1 channel) or Stereo (2 channels)
- **Custom Duration**: Generate any length of audio in milliseconds
- **Multiple Output Formats**: Hex, C arrays, Rust arrays, raw binary, Waveform Audio File Format (PCM)
- **Analysis Mode**: Calculate buffer requirements and efficiency

## Use Cases

- **Embedded Development**: Generate test data for audio devices
- **Hardware Testing**: Create reference signals for audio quality testing
- **Education**: Understand digital audio generation and USB audio packetization
- **Prototyping**: Quickly generate audio buffers without external tools

### Prerequisites
- Rust toolchain (1.70+)

### Build from Source

```bash
# Clone and build
git clone <repository-url>
cd sine_generator
cargo build --release

# The binary will be at ./target/release/sine_generator
```
### Basic Examples

```bash
# Default settings (440Hz, 16kHz, stereo 16-bit, 1ms)
./sine_generator

# Generate 1kHz sine at 48kHz for 10ms, 24-bit
./sine_generator -f 1000 -r 48000 -b 24 -d 10

# Mono output with C array format
./sine_generator -c 1 -o carray

# Optimize for USB packets (64-byte boundaries)
./sine_generator -r 32000 -d 1 -p

# Rust array for embedded use
./sine_generator -r 16000 -d 1 -o rustarray

# Just show info without generating data
./sine_generator -a

# Raw binary output (pipe to file)
./sine_generator -r 16000 -d 10 -o raw > sinewave.bin

# Wav output (pipe to file)
./sine_generator -d 1000 -f 1000 -o wav > sinewave.wav


```

### Command Line Options

```
Sine Wave Generator for USB Audio Testing
Usage: sine_generator [OPTIONS]

Options:
  -f, --frequency FREQ     Sine wave frequency in Hz (default: 440.0)
  -r, --rate RATE          Sample rate in Hz (default: 16000)
                           Supported: 16000, 44100, 48000
  -c, --channels CH        Number of channels (1=mono, 2=stereo, default: 2)
  -b, --bits BITS          Bit depth: 16, 24, or 32 (default: 16)
  -d, --duration MS        Duration in milliseconds (default: 1.0)
  -o, --output FORMAT      Output format:
                           hex      - Hexadecimal values (default)
                           carray   - C-style array declaration
                           rustarray - Rust array declaration
                           raw      - Raw binary bytes (stdout)
                           info     - Only show buffer info, no data
  -p, --packet-mode        Optimize for USB packets (64-byte boundaries)
  -a, --analyze            Analyze only (don't generate data)
  -h, --help               Show this help message

Examples:
  sine_generator -f 1000 -r 48000 -b 16 -d 10 -o carray
  sine_generator --frequency 440 --rate 44100 --channels 1 --bits 24
  sine_generator -r 16000 -d 1 -o rustarray -p
```

## Output Formats

### 1. Hex Format (Default)
Prints hexadecimal values of the generated buffer:

```
[0x00, 0x00, 0x00, 0x00, 0x01, 0x16, 0x01, 0x16, 0x5B, 0x2B, 0x5B, 0x2B, 0x6A, 0x3F, 0x6A, 0x3F,
 0x96, 0x51, 0x96, 0x51, 0x54, 0x61, 0x54, 0x61, 0x2B, 0x6E, 0x2B, 0x6E, 0xBB, 0x77, 0xBB, 0x77,
 ...]
```

### 2. C Array Format
Generates C/C++ compatible array declarations:

```c
// Sine wave: 440 Hz, 1 ms, 16-bit, 2 channels
// Sample rate: 16000 Hz
// Total bytes: 64
const uint8_t SINE_16000HZ_1MS_16BIT_2CH[64] = {
    0x00, 0x00, 0x00, 0x00, 0x01, 0x16, 0x01, 0x16, 0x5B, 0x2B, 0x5B, 0x2B, 0x6A, 0x3F, 0x6A, 0x3F,
    0x96, 0x51, 0x96, 0x51, 0x54, 0x61, 0x54, 0x61, 0x2B, 0x6E, 0x2B, 0x6E, 0xBB, 0x77, 0xBB, 0x77,
    ...
};
```

### 3. Rust Array Format
Generates Rust array declarations:

```rust
// Sine wave: 440 Hz, 1 ms, 16-bit, 2 channels
// Sample rate: 16000 Hz
// Total bytes: 64
pub const SINE_16000HZ_1MS_16BIT_2CH: [u8; 64] = [
    0x00, 0x00, 0x00, 0x00, 0x01, 0x16, 0x01, 0x16, 0x5B, 0x2B, 0x5B, 0x2B, 0x6A, 0x3F, 0x6A, 0x3F,
    0x96, 0x51, 0x96, 0x51, 0x54, 0x61, 0x54, 0x61, 0x2B, 0x6E, 0x2B, 0x6E, 0xBB, 0x77, 0xBB, 0x77,
    ...
];
```

### 4. Raw Binary Format
Outputs raw binary data to stdout (useful for piping to files or other programs).

### 5. Info Format
Shows detailed analysis without generating the actual data buffer.

### Example Analysis Output

```
Sine Wave Generator - Configuration
=====================================
Frequency:      440.0 Hz
Sample Rate:    16000 Hz
Channels:       2 (stereo)
Bit Depth:      16-bit
Duration:       1.0 ms

Buffer Analysis:
  Samples:      16
  Total bytes:  64

Frequency Analysis:
  Period:       36.36 samples
  Full cycles:  0.44
```

### Audio Generation Algorithm
The tool uses high-precision floating-point math to generate sine waves:
- Phase accumulation with modulo wrapping to prevent discontinuities
- Proper handling of 24-bit signed audio (sign extension)
- Accurate duration calculation with rounding for non-integer sample rates

### Dependencies
- Standard library only (no external dependencies)

### For C/C++ Projects
1. Generate a C array:

   ```bash
   ./sine_generator -r 16000 -d 1 -o carray > sine_buffer.h
   ```

2. Include in your project:

   ```c
   #include "sine_buffer.h"
   
   // Use SINE_16000HZ_1MS_16BIT_2CH array
   ```

### For Rust Embedded Projects
1. Generate a Rust array:

   ```bash
   ./sine_generator -r 16000 -d 1 -o rustarray > src/sine_buffer.rs
   ```

2. Include in your project:

   ```rust
   mod sine_buffer;
   use sine_buffer::SINE_16000HZ_1MS_16BIT_2CH;
   ```

### Generate Test Patterns For Testing Audio:

   ```bash
   # Generate multiple test frequencies
   for freq in 440 1000 2000 4000; do
     ./sine_generator -f $freq -r 48000 -d 100 -o raw > test_${freq}hz.bin
   done
   ```

### Common Issues

 **Phase discontinuities in generated audio**
   - The code uses modulo phase wrapping to prevent discontinuities
   - If you still hear clicks, try generating longer buffers or using integer frequency multiples of sample rate

## License

This project is available for use under the MIT License.
