use std::{io, str::FromStr};
use std::f32::consts::PI;
use std::i16;
use hound;

use clap::Parser;

const WINDOWS_RESERVED_KEYWORDS : [&str; 22] = [
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const WINDOWS_INVALID_CHARACTERS : [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// A simple program to generate linear chirp and ave it into .wav file
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Initial chirp frequency in Hz
    #[arg(short, long)]
    initial_freq: Option<f32>,

    /// Ending chirp frequency in Hz
    #[arg(short, long)]
    ending_freq: Option<f32>,

    /// Duration of generated .wav file in seconds
    #[arg(short, long)]
    duration : Option<u32>,

    /// Name for output .wav file
    #[arg(short, long)]
    output_name : Option<String>,
}

fn main() {
    let args = Args::parse();

    const SAMPLE_RATE : u32 = 44100;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let initial_freq: f32 = args.initial_freq.unwrap_or_else(|| input_value("Please input initial frequency (Hz):"));
    let ending_freq: f32 = args.ending_freq.unwrap_or_else(|| input_value("Please input ending frequency (Hz):"));
    let duration: u32 = args.duration.unwrap_or_else(|| input_value("Please input file duration in seconds:"));
    let output: String  = args.output_name
        .as_ref()
        .and_then(|name| match validate_file_name(name) {
            Ok(_) => Some(name.trim().to_string()),
            Err(e) => {
                println!("{e}. Defaulting to manual input.");
                None
            }
        })
        .unwrap_or_else(|| {
            let input_name = input_file_name();
            input_name.trim().to_string()
        });

    let mut writer = hound::WavWriter::create(&format!("{output}.wav"), spec).unwrap();
   
    let samples_total = SAMPLE_RATE * duration;
    
    for t in (0 .. (samples_total)).map(|x| x as f32 / SAMPLE_RATE as f32) {
        let sample = (2.0 * PI * (((ending_freq - initial_freq)/(2.0 * duration as f32)) * t * t + initial_freq * t)).sin();
        let amplitude = i16::MAX as f32;
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }
}

fn validate_file_name(file_name: &str) -> Result<&str, String> {
    if WINDOWS_RESERVED_KEYWORDS.iter().any(|&res| res.eq_ignore_ascii_case(file_name)) {
        return Err(format!("The file name '{}' is a reserved name on Windows.", file_name));
    }

    if let Some(invalid_char) = file_name.chars().find(|&c| WINDOWS_INVALID_CHARACTERS.contains(&c)) {
        return Err(format!(
            "The file name '{}' contains an invalid character: '{}'.",
            file_name, invalid_char
        ));
    }

    if file_name.ends_with('.') {
        return Err(format!(
            "The file name '{}' cannot end with a period.",
            file_name
        ));
    }

    if file_name.len() > 255 {
        return Err(format!(
            "The file name '{}' exceeds the maximum length of 255 characters.",
            file_name
        ));
    }

    Ok(file_name)
}

fn input_value<T>(prompt : &str) -> T where T : std::str::FromStr, <T as FromStr>::Err: std::fmt::Display {
    loop {
        println!("{prompt}");
    
        let mut x = String::new();

        io::stdin()
        .read_line(&mut x)
        .expect("Failed to read line");

        match x.trim().parse() {
            Ok(v) => break v,
            Err(e) => {
                println!("{e}. Try again!");
                continue;
            }
        };
    }
}

fn input_file_name() -> String {
    loop {
        let file_name: String = input_value("Please input output file name:");
        match validate_file_name(&file_name){
            Ok(_) => return file_name,
            Err(e) => println!("{e}. Try again!")
        }
    }
}