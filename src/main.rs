use anyhow::{Context, Result, anyhow};
use chrono::Local;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Duration};
use url::Url;
use readability::extractor;
use scraper::Html;

const TOP_URL: &str = "https://hacker-news.firebaseio.com/v0/topstories.json";
const ITEM_URL: &str = "https://hacker-news.firebaseio.com/v0/item/";
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36";
const CONTENT_FETCH_TIMEOUT: u64 = 10; // 10 seconds timeout
// No longer needed since we're showing full content instead of previews

#[derive(Deserialize)]
struct Item {
    #[allow(dead_code)]
    id: u64,
    by: Option<String>,
    score: Option<u32>,
    #[allow(dead_code)]
    time: Option<u64>,
    title: Option<String>,
    url: Option<String>,
    descendants: Option<u32>,
}

#[derive(Debug)]
struct ScrapedContent {
    #[allow(dead_code)]
    title: String,
    // Original plaintext content
    #[allow(dead_code)]
    content: String,
    // HTML content for rendering
    content_html: String,
    is_paywall: bool,
    domain: String,
}

fn main() -> Result<()> {
    let stories = fetch_front_page(30)?;
    let out_dir = dirs::home_dir().unwrap_or(PathBuf::from(".")).join("hn_daily");
    fs::create_dir_all(&out_dir)?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    
    println!("Fetching article content (this may take a minute)...");
    let stories_with_content = fetch_article_content(&stories)?;
    
    // Generate regular HTML with content
    let html = render_html(&stories, &stories_with_content)?;
    let html_path = out_dir.join(format!("{}.html", &date));
    fs::write(&html_path, &html)?;

    // Generate text version
    let text = html2text::from_read(html.as_bytes(), 80);
    let text_path = out_dir.join(format!("{}.txt", &date));
    fs::write(&text_path, text)?;
    
    // optional: create PDF if wkhtmltopdf is present
    if which::which("wkhtmltopdf").is_ok() {
        let pdf_path = out_dir.join(format!("{}.pdf", &date));
        std::process::Command::new("wkhtmltopdf")
            .arg("--quiet")
            .arg(&html_path)
            .arg(&pdf_path)
            .status()
            .ok();
    }

    println!("Files generated in {}", out_dir.display());
    println!("- {}.html - HTML digest", date);
    println!("- {}.txt - Plain text digest", date);
    if which::which("wkhtmltopdf").is_ok() {
        println!("- {}.pdf - PDF digest", date);
    }

    Ok(())
}

fn fetch_front_page(limit: usize) -> Result<Vec<Item>> {
    let client = Client::builder()
        .user_agent("hn_daily/0.1")
        .timeout(Duration::from_secs(10))
        .build()?;
        
    let ids: Vec<u64> = client
        .get(TOP_URL)
        .send()?
        .json()
        .context("topstories JSON")?;

    let mut items = Vec::with_capacity(limit);
    for id in ids.into_iter().take(limit) {
        let item: Item = client
            .get(format!("{id_url}{id}.json", id_url = ITEM_URL))
            .send()?
            .json()
            .with_context(|| format!("item {id}"))?;
        items.push(item);
    }
    Ok(items)
}

fn fetch_article_content(items: &[Item]) -> Result<Vec<Option<ScrapedContent>>> {
    // Create browser-like headers to help with some paywalls
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(BROWSER_UA));
    
    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(CONTENT_FETCH_TIMEOUT))
        .build()?;
    
    // Process each URL
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        let url = match &item.url {
            Some(url) if !url.is_empty() => url,
            _ => {
                // Skip items without URLs (e.g., "Ask HN" posts)
                results.push(None);
                continue;
            }
        };
        
        println!("Fetching: {}", url);
        
        match fetch_and_process(&client, url) {
            Ok(content) => results.push(Some(content)),
            Err(e) => {
                println!("Failed to fetch {}: {}", url, e);
                results.push(None);
            }
        }
    }
    
    Ok(results)
}

fn fetch_and_process(client: &Client, url: &str) -> Result<ScrapedContent> {
    let response = match client.get(url).send() {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err(anyhow!("Failed with status: {}", resp.status()));
            }
            resp
        }
        Err(e) => return Err(anyhow!("Request failed: {}", e)),
    };
    
    let is_paywall = detect_paywall(&response);
    let domain = extract_domain(url)?;
    
    // Get page HTML
    let html = response.text()?;
    
    // Process with Readability
    let parsed_url = Url::parse(url)?;
    let mut html_bytes = html.as_bytes();
    let article = extractor::extract(&mut html_bytes, &parsed_url)?;
    let content_html = article.content;
    let content = clean_content(&content_html);
    
    Ok(ScrapedContent {
        title: article.title,
        content,
        content_html,
        is_paywall,
        domain,
    })
}

fn detect_paywall(response: &Response) -> bool {
    // Simple heuristic - more advanced detection would need specific site handling
    let content_type = response.headers().get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if !content_type.contains("text/html") {
        return true; // Not HTML content might be a redirect or paywall
    }
    
    // We could add more sophisticated detection based on cookies, redirects, etc.
    false
}

fn clean_content(html: &str) -> String {
    let document = Html::parse_document(html);
    
    // Extract text from the HTML document
    let text = document.root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ");
    
    // Clean up whitespace
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_domain(url: &str) -> Result<String> {
    let parsed = Url::parse(url)?;
    parsed.host_str()
        .map(|h| h.to_string())
        .ok_or_else(|| anyhow!("No host in URL"))
}

fn render_html(items: &[Item], contents: &[Option<ScrapedContent>]) -> Result<String> {
    let today = Local::now().format("%B %e, %Y").to_string();
    
    // Build index
    let mut index = String::new();
    for (i, it) in items.iter().enumerate() {
        let title = it.title.as_deref().unwrap_or("[no title]");
        index.push_str(&format!(
            "<li><a href=\"#article-{}\">{}</a></li>",
            i, title
        ));
    }
    
    // Build article content
    let mut articles = String::new();
    for (i, it) in items.iter().enumerate() {
        let url = it.url.as_deref().unwrap_or("#");
        let title = it.title.as_deref().unwrap_or("[no title]");
        let score = it.score.unwrap_or(0);
        let by = it.by.as_deref().unwrap_or("unknown");
        let comments = it.descendants.unwrap_or(0);
        
        // Article content section
        let content_html = match &contents.get(i) {
            Some(Some(content)) => {
                // Use full content instead of preview
                let paywall_warning = if content.is_paywall {
                    "<div class=\"paywall-warning\">Content may be behind a paywall</div>"
                } else {
                    ""
                };
                
                format!(
                    "<div class=\"content\">\
                    <div class=\"domain\">{}</div>\
                    {}\
                    <div class=\"full-content\">{}</div>\
                    </div>",
                    content.domain,
                    paywall_warning,
                    content.content_html
                )
            },
            _ => "<div class=\"content\">\
                  <em>Could not retrieve content</em>\
                  </div>".to_string(),
        };
        
        articles.push_str(&format!(
            "<article id=\"article-{}\" class=\"story\">\
            <h2><a href=\"{}\">{}</a></h2>\
            <p class=\"meta\">{} points • by {} • {} comments</p>\
            {}\
            </article>\
            <hr>",
            i,
            url,
            title,
            score,
            by,
            comments,
            content_html
        ));
    }

    Ok(format!(
        "<!DOCTYPE html>\
<html lang=\"en\">\
<head>\
<meta charset=\"utf-8\">\
<title>Hacker News Daily – {}</title>\
<style>\
body{{font-family:Georgia,serif;margin:0;padding:0;display:flex;flex-direction:column;}}\
h1{{text-align:center;margin:0;padding:20px 0 10px 0;}}\
.date{{text-align:center;margin:0 0 20px 0;}}\
.main-container{{display:flex;flex:1;}}\
.sidebar{{position:sticky;top:0;width:280px;height:100vh;overflow-y:auto;background:#f8f8f8;padding:15px;box-sizing:border-box;border-right:1px solid #ddd;}}\
.sidebar h2{{text-align:center;margin-top:0;}}\
.story-index{{padding-left:20px;}}\
.story-index li{{margin-bottom:0.8em;font-size:0.9em;}}\
.articles{{flex:1;padding:20px 40px;overflow-y:auto;max-width:800px;margin:0 auto;}}\
.story{{margin-bottom:1.5em;}}\
.story h2{{font-size:1.3em;margin:1em 0 .1em 0;}}\
.meta{{font-size:.8em;color:#555;margin:0 0 .5em 0;}}\
.content{{font-size:0.85em;margin-top:0.5em;}}\
.domain{{color:#888;font-size:0.9em;margin-bottom:0.3em;}}\
.full-content{{line-height:1.5;margin-top:1em;}}\
.full-content p{{margin:0.7em 0;}}\
.paywall-warning{{color:#aa3300;font-style:italic;margin-bottom:0.3em;}}\
.back-to-top{{display:none;}}\
hr{{border:0;border-top:1px solid #ddd;margin:2em 0;}}\
a{{color:#000;text-decoration:none;}}\
a:hover{{text-decoration:underline;}}\
a.active{{font-weight:bold;color:#ff6600;}}\
@media print{{.sidebar{{display:none;}} .articles{{margin:0;max-width:none;}} a{{color:#000}}}}\
@media (max-width: 800px) {{.main-container{{flex-direction:column;}} .sidebar{{position:static;width:100%;height:auto;}} .articles{{max-width:none;}}}}\
</style>\
<script>\
function highlightActive() {{\
  const articles = document.querySelectorAll('.story');\
  const links = document.querySelectorAll('.story-index a');\
  const observer = new IntersectionObserver((entries) => {{\
    entries.forEach(entry => {{\
      if (entry.isIntersecting) {{\
        const id = entry.target.id;\
        links.forEach(link => {{\
          if (link.getAttribute('href') === '#' + id) {{\
            link.classList.add('active');\
          }} else {{\
            link.classList.remove('active');\
          }}\
        }});\
      }}\
    }});\
  }}, {{ threshold: 0.5 }});\
  \
  articles.forEach(article => {{\
    observer.observe(article);\
  }});\
}}\
\
document.addEventListener('DOMContentLoaded', highlightActive);\
</script>\
</head>\
<body>\
<h1>Hacker News Daily</h1>\
<p class=\"date\">{}</p>\
\
<div class=\"main-container\">\
  <div class=\"sidebar\">\
    <h2>Article Index</h2>\
    <ol class=\"story-index\">\
      {}\
    </ol>\
  </div>\
\
  <div class=\"articles\">\
  {}\
  </div>\
</div>\
\
</body></html>",
        today,
        today,
        index,
        articles
    ))
}