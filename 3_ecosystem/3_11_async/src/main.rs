use clap::Parser;
use std::{fs, sync::Arc, time::Instant};
use tokio::sync::Semaphore;

/// Downloads web pages from a list of URLs concurrently.
#[derive(Parser)]
struct Cli {
    /// Maximum number of simultaneously running threads
    #[arg(long, default_value_t = num_cpus())]
    max_threads: usize,

    /// File containing URLs (one per line)
    file: String,
}

// the number of cores on this computer
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Sanitize a URL into a valid filename: replace special chars with '_'
fn url_to_filename(url: &str) -> String {
    let name = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
        .collect();
    format!("{sanitized}.html")
}

async fn download(client: reqwest::Client, url: String) {
    match client.get(&url).send().await {
        Err(e) => eprintln!("ERROR {url}: {e}"),
        Ok(resp) => match resp.text().await {
            Err(e) => eprintln!("ERROR reading {url}: {e}"),
            Ok(body) => {
                let filename = url_to_filename(&url);
                match fs::write(&filename, &body) {
                    Ok(_) => println!("OK {url} -> {filename}"),
                    Err(e) => eprintln!("ERROR writing {filename}: {e}"),
                }
            }
        },
    }
}

#[tokio::main]
async fn main() {
    let num = num_cpus();
    let now = Instant::now();
    println!("{}", num);
    let cli = Cli::parse();

    let content = tokio::fs::read_to_string(&cli.file).await
        .unwrap_or_else(|e| panic!("Cannot read '{}': {e}", cli.file));

    let urls: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    let semaphore = Arc::new(Semaphore::new(cli.max_threads));
    let client = reqwest::Client::new();

    let mut handles = Vec::with_capacity(urls.len());

    for url in urls {
        let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let now = Instant::now();
            download(client, url).await;
            drop(permit); // release slot when done
            println!("{:?}", now.elapsed());
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }
    println!("all time: {:?}", now.elapsed());
}
