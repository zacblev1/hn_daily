# Hacker News Daily

A simple utility that creates a daily digest of top Hacker News stories in a reader-friendly format, including full article content with an interactive index.

## Features

- Fetches top 30 stories from Hacker News
- Retrieves full article content from linked pages with paywall detection
- Uses readability algorithms to extract clean article text
- Generates HTML with an interactive index for easy navigation between articles
- Shows complete article content for each story
- Creates a plain text version for easy reading
- Optionally creates a PDF if wkhtmltopdf is installed
- Files are saved to `~/hn_daily/YYYY-MM-DD.{html,txt,pdf}`

## Installation

```
git clone https://github.com/zacblev1/hn_daily.git
cd hn_daily
cargo build --release
```

## Usage

Run manually:
```
./target/release/hn_daily
```

Or set up a cron job to run it daily at 8:00 AM:
```
0 8 * * * /path/to/hn_daily/target/release/hn_daily
```

This will generate the following files in your home directory under `~/hn_daily/`:
- `YYYY-MM-DD.html` - HTML version of the digest
- `YYYY-MM-DD.txt` - Plain text version for easy reading
- `YYYY-MM-DD.pdf` - PDF version (if wkhtmltopdf is installed)

## Requirements

- Rust 2021 edition or newer
- `wkhtmltopdf` (optional, for PDF generation)

## License

This project is licensed under the MIT License - see the LICENSE file for details.