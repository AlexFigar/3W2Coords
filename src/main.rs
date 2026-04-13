use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Deserialize, Debug)]
struct ApiResponse {
    coordinates: Coordinates,
}

#[derive(Deserialize, Debug)]
struct Coordinates {
    lat: f64,
    lng: f64,
}

fn lookup_words(client: &Client, words: &str) -> (f64, f64) {
    let url = format!(
        "https://mapapi.what3words.com/api/convert-to-coordinates?words={}&format=json",
        words
    );
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .expect("Failed to send request");
    let body = response.text().expect("Failed to get response body");
    let api_response: ApiResponse =
        serde_json::from_str(&body).expect("Failed to parse JSON response");
    (api_response.coordinates.lat, api_response.coordinates.lng)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (input, output_to_file, output_path) = match args.len() {
        2 => (args[1].clone(), false, String::new()),
        3 => (args[1].clone(), true, args[2].clone()),
        _ => {
            eprintln!("Usage: {} <input> [output_file]", args[0]);
            eprintln!("  <input> can be:");
            eprintln!("    - A file path (one what3words per line)");
            eprintln!("    - A single set of three words (e.g. prices.slippery.traps)");
            eprintln!("  [output_file] is optional - if omitted, output goes to stdout");
            std::process::exit(1);
        }
    };

    let client = Client::new();

    let words_list: Vec<String> =
        if input.contains('.') && !input.contains('/') && !input.contains('\\') {
            vec![input.clone()]
        } else {
            let input_file = File::open(&input).expect("Failed to open input file");
            let reader = BufReader::new(input_file);
            reader
                .lines()
                .map(|l| l.unwrap().trim().to_string())
                .filter(|w| !w.is_empty())
                .collect()
        };

    let mut output_file =
        output_to_file.then(|| File::create(&output_path).expect("Failed to create output file"));

    for words in words_list {
        let (lat, lng) = lookup_words(&client, &words);
        let output_line = format!("{},{},{}", words, lat, lng);

        if let Some(ref mut file) = output_file {
            file.write_all(format!("{}\n", output_line).as_bytes())
                .expect("Failed to write to output file");
        } else {
            println!("{}", output_line);
        }
    }

    if output_to_file {
        println!("Output written to {}", output_path);
    }
}
