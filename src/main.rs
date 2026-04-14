use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input: a file path, single what3words address, or coordinates (lat,lng)
    input: String,

    /// Output file path (optional - defaults to stdout)
    output: Option<String>,

    /// Output format: text, csv, or json
    #[arg(short, long, help = "Output format", default_value = "text")]
    format: OutputFormat,

    /// Perform reverse lookup (coordinates to what3words)
    #[arg(short, long, help = "Coordinate to 3 words (reverse lookup)")]
    reverse: bool,
}

/// Supported output formats for results
#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum OutputFormat {
    Text,
    Csv,
    Json,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    coordinates: Coordinates,
}

#[derive(Deserialize, Debug)]
struct ReverseApiResponse {
    words: String,
}

#[derive(Deserialize, Debug)]
struct Coordinates {
    lat: f64,
    lng: f64,
}

/// Returns None if input is invalid.
fn parse_coordinates(input: &str) -> Option<(f64, f64)> {
    let coords: Vec<&str> = input.split(',').collect();
    if coords.len() == 2 {
        let lat = coords[0].trim().parse::<f64>().ok()?;
        let lng = coords[1].trim().parse::<f64>().ok()?;
        Some((lat, lng))
    } else {
        None
    }
}

/// Words --> Coords: `https://mapapi.what3words.com/api/convert-to-coordinates`
fn lookup_words(client: &Client, words: &str) -> Result<(f64, f64), String> {
    let url = format!(
        "https://mapapi.what3words.com/api/convert-to-coordinates?words={}&format=json",
        words
    );

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let body = response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if body.contains("\"error\"") {
        return Err("Invalid what3words address".to_string());
    }

    let api_response: ApiResponse =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok((api_response.coordinates.lat, api_response.coordinates.lng))
}

/// Coords --> Words: `https://mapapi.what3words.com/api/convert-to-3wa`
fn reverse_lookup(client: &Client, lat: f64, lng: f64) -> Result<String, String> {
    let url = format!(
        "https://mapapi.what3words.com/api/convert-to-3wa?coordinates={}%2C{}&language=en&format=json",
        lat, lng
    );

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let body = response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if body.contains("\"error\"") {
        return Err("Invalid coordinates".to_string());
    }

    let api_response: ReverseApiResponse =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(api_response.words)
}

fn format_output(words: &str, lat: f64, lng: f64, format: &OutputFormat) -> String {
    match format {
        // Text and CSV use the same format: words,lat,lng
        OutputFormat::Text => format!("{},{},{}", words, lat, lng),
        OutputFormat::Csv => format!("{},{},{}", words, lat, lng),
        OutputFormat::Json => {
            let obj = json!({
                "words": words,
                "lat": lat,
                "lng": lng
            });
            obj.to_string()
        }
    }
}

//Where it begins
fn main() {
    let cli = Cli::parse();
    let client = Client::new();

    let is_coords = cli.input.contains(',') && parse_coordinates(&cli.input).is_some();
    let is_file = Path::new(&cli.input).exists();

    // Collect all items to process based on input type
    let (items, is_reverse): (Vec<String>, bool) = if is_coords {
        // Input is coordinate pair - use reverse lookup mode
        (vec![cli.input.clone()], true)
    } else if is_file {
        // Input is a file - read all what3words addresses from file
        let input_file = File::open(&cli.input).expect("Failed to open input file");
        let reader = BufReader::new(input_file);
        let items: Vec<String> = reader
            .lines()
            .map(|l| l.unwrap().trim().to_string())
            .filter(|w| !w.is_empty())
            .collect();
        (items, cli.reverse)
    } else {
        // Input is a single what3words address
        (vec![cli.input.clone()], cli.reverse)
    };

    // Set up progress bar for bulk operations (more than 1 item)
    let count = items.len();
    let show_progress: bool = count > 1;

    let pb = if show_progress {
        let pb = ProgressBar::new(count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len}")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let mut results: Vec<(String, String)> = Vec::new();

    // Process each item
    for item in items {
        if is_reverse || cli.reverse {
            // Reverse lookup: coordinates -> what3words
            let coords = parse_coordinates(&item)
                .unwrap_or_else(|| parse_coordinates(&cli.input).unwrap_or((0.0, 0.0)));
            let (lat, lng) = coords;
            match reverse_lookup(&client, lat, lng) {
                Ok(words) => {
                    results.push((format_output(&words, lat, lng, &cli.format), String::new()))
                }
                Err(e) => results.push((e, format!("Error: {},{}", lat, lng))),
            }
        } else {
            // Forward lookup: what3words -> coordinates
            let words = item;
            match lookup_words(&client, &words) {
                Ok((lat, lng)) => {
                    results.push((format_output(&words, lat, lng, &cli.format), String::new()))
                }
                Err(e) => results.push((e, words)),
            }
        }

        // Update progress bar
        if let Some(ref pb) = pb {
            pb.inc(1);
        }
    }

    // Finish progress bar
    if let Some(pb) = pb {
        pb.finish();
    }

    // Format final output based on requested format
    let output: String = if cli.format == OutputFormat::Json {
        // Wrap individual JSON objects into a JSON array
        let arr: Vec<_> = results.iter().map(|(r, _)| r).collect();
        json!(arr).to_string()
    } else {
        // Join all results with newlines
        results
            .iter()
            .map(|(r, _)| r.clone())
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Write output to file or stdout
    if let Some(output_path) = &cli.output {
        let mut file = File::create(output_path).expect("Failed to create output file");
        file.write_all(output.as_bytes())
            .expect("Failed to write output");
        println!("Output written to {}", output_path);
    } else {
        println!("{}", output);
    }
}
