# Hacker News Daily

A simple utility that creates a daily digest of top Hacker News stories in a reader-friendly format. It includes full article content with a convenient fixed sidebar index for easy navigation.

## Features

- Fetches top 30 stories from Hacker News
- Retrieves full article content from linked pages with paywall detection
- Uses readability algorithms to extract clean, readable article text
- Generates HTML with a fixed sidebar index for easy navigation between articles
- Provides responsive layout that works well on desktop and mobile devices
- Highlights the current article in the sidebar navigation
- Shows complete article content with proper formatting for images, code, and tables
- Prevents horizontal scrolling for comfortable reading
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
- `YYYY-MM-DD.html` - HTML version of the digest with interactive sidebar
- `YYYY-MM-DD.txt` - Plain text version for easy reading
- `YYYY-MM-DD.pdf` - PDF version (if wkhtmltopdf is installed)

### Reading the Digest

The HTML digest provides several features for easy reading:

1. **Fixed Sidebar Navigation**:
   - The sidebar stays visible as you scroll through articles
   - Click on any article title to jump directly to it
   - The currently visible article is highlighted in the sidebar

2. **Full Article Content**:
   - Complete article text with proper formatting
   - Images, code blocks, and tables are displayed properly
   - Content is formatted to prevent horizontal scrolling

3. **Responsive Design**:
   - Works well on desktop, tablet, and mobile devices
   - Adapts layout for different screen sizes
   - Sidebar collapses to top navigation on smaller screens

## Requirements

- Rust 2021 edition or newer
- `wkhtmltopdf` (optional, for PDF generation)

## License

This project is licensed under the MIT License - see the LICENSE file for details.