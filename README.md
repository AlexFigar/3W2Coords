# Three Words to Coordinates

CLI tool that converts what3words addresses to latitude/longitude coordinates and vice versa.

## Installation

```bash
./build.sh
```

Or manually:
```bash
cargo build --release
```

The compiled binary will be at: `target/release/3W2Coords`

## Usage

```bash
# Single what3words to coordinates
./3W2Coords prices.slippery.traps

# Coordinates to what3words (reverse lookup)
./3W2Coords "48.858358,2.294473" --reverse

# File with multiple locations
./3W2Coords test.txt

# Specify output file
./3W2Coords test.txt output.txt

# Output format options
./3W2Coords prices.slippery.traps -f text   # default: words,lat,lng
./3W2Coords prices.slippery.traps -f csv    # comma-separated
./3W2Coords prices.slippery.traps -f json   # JSON array
```

## Options

```
-f, --format <FORMAT>  Output format: text, csv, json [default: text]
-r, --reverse       Coordinate to 3 words (reverse lookup)
-h, --help         Print help
-V, --version      Print version
```

## Output Format

**text/csv** (default):
```
words,lat,lng
prices.slippery.traps,48.858358,2.294473
```

**json**:
```json
[{"lat":48.858358,"lng":2.294473,"words":"prices.slippery.traps"}]
```

## API

Uses the free what3words API:
- Forward: `https://mapapi.what3words.com/api/convert-to-coordinates`
- Reverse: `https://mapapi.what3words.com/api/convert-to-3wa`

No API key required.
