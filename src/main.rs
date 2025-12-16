use bencodex::json::{BinaryEncoding, JsonEncodeOptions, from_json, to_json_with_options};
use bencodex::{Decode, Encode};
use clap::Parser;
use std::io::{Read, Write};
use std::process::ExitCode;

/// A program to encode and decode between Bencodex and JSON.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Encode Bencodex Binary by base64 string.
    /// If not given, it will encode as hexadecimal string.
    #[arg(short, long)]
    base64: bool,

    /// Decode to Bencodex from JSON.
    #[arg(short, long)]
    decode: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if !args.decode {
        encode(&args)
    } else {
        decode()
    }
}

fn decode() -> ExitCode {
    let mut buf = Vec::new();
    if let Err(err) = std::io::stdin().read_to_end(&mut buf) {
        eprintln!("Failed to read from stdin: {:?}", err);
        return ExitCode::FAILURE;
    }

    let json = match serde_json::from_slice(&buf) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Failed to parse JSON: {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    let bencodex_value = match from_json(&json) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Failed to decode JSON to Bencodex: {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    buf = Vec::new();
    if let Err(err) = bencodex_value.encode(&mut buf) {
        eprintln!("Failed to encode Bencodex to binary: {:?}", err);
        return ExitCode::FAILURE;
    }

    if let Err(err) = std::io::stdout().write_all(&buf) {
        eprintln!("Failed to write to stdout: {:?}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn encode(args: &Args) -> ExitCode {
    let mut buf = Vec::new();
    if let Err(err) = std::io::stdin().read_to_end(&mut buf) {
        eprintln!("Failed to read from stdin: {:?}", err);
        return ExitCode::FAILURE;
    }

    let decoded = match buf.decode() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to decode to Bencodex: {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    let json_encode_options = JsonEncodeOptions {
        binary_encoding: if args.base64 {
            BinaryEncoding::Base64
        } else {
            BinaryEncoding::Hex
        },
    };

    let json_str = match to_json_with_options(&decoded, json_encode_options) {
        Ok(json_str) => json_str,
        Err(err) => {
            eprintln!("Failed to encode Bencodex to JSON: {:?}", err);
            return ExitCode::FAILURE;
        }
    };

    println!("{}", json_str);

    ExitCode::SUCCESS
}
