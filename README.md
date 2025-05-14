# WhatsApp Stats Parser

A Rust-based command-line tool to parse WhatsApp chat logs and compute per-user statistics such as message count, word count, and activity over time.

Supports multiline messages, yearly breakdowns, and customizable sorting with optional pretty-printed tables.

## Features

- Parses WhatsApp exports (including multiline messages)
- Aggregates per-user statistics
- Supports per-year grouping
- Sortable by message or word count
- Optional pretty-printed tables using `tabled`

## Installation

git clone https://github.com/SammyTbeile/whatsapp_stats.git
cd whatsapp_stats
cargo build --release

## Usage

cargo run -- path/to/chat.txt

### Flags

--year or -y  
Group stats by year

--pretty or -p  
Pretty print output using a table format

--sort [messages|words]  
Sort output by number of messages or words (default is messages)

### Example

cargo run -- chat.txt --year --pretty --sort words

## Input Format

The parser expects WhatsApp exports in this format:

[3/1/17, 11:36:03 PM] User 1: message 1
[3/1/17, 11:50:12 PM] User 2: message 2

Multiline messages are supported and will be grouped correctly.

## Dependencies

- chrono
- regex
- clap
- tabled (for --pretty)

Add them in your Cargo.toml:

[dependencies]
chrono = "0.4"
regex = "1"
clap = { version = "4", features = ["derive"] }
tabled = "0.14"

## License

MIT © 2025