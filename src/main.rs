
use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::task::spawn;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::{fs::File as AsyncFile, io::AsyncBufReadExt, io::AsyncWriteExt, task, sync::Semaphore};


async fn download_http_proxies() -> Result<(), reqwest::Error> {
  const URLS: &[&str] = &[
    "https://raw.githubusercontent.com/MuRongPIG/Proxy-Master/main/http.txt",
    "https://api.proxyscrape.com/v2/?request=getproxies&amp;protocol=http&amp;timeout=10000&amp;country=all&amp;ssl=all&amp;anonymity=all",
    "https://raw.githubusercontent.com/hookzof/socks5_list/master/proxy.txt",
    "https://raw.githubusercontent.com/ShiftyTR/Proxy-List/master/http.txt",
    "https://raw.githubusercontent.com/ShiftyTR/Proxy-List/master/https.txt",
    "https://raw.githubusercontent.com/saisuiu/Lionkings-Http-Proxys-Proxies/main/awkpro2.txt",
    "https://raw.githubusercontent.com/ALIILAPRO/Proxy/main/http.txt",
];    
    let output_file = "proxies_raw_http.txt";
    let output_path = Path::new(output_file);
    let mut out_file = AsyncFile::create(output_path).await.unwrap();
    for url in URLS {
        let response = reqwest::get(url.to_string()).await?;
        let body = response.text().await?;
        out_file.write_all(body.as_bytes()).await;
    }

    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let local_ip = reqwest::get("https://ipinfo.io/json")
        .await?
        .json::<Value>()
        .await?
        .get("ip")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();

    let unvalidated_proxies = open_text("proxies_raw_http.txt");
    let unique_proxies: HashSet<String> = unvalidated_proxies.into_iter().collect();
    let valid_proxies = Arc::new(Mutex::new(Vec::new()));
    let mut tasks = Vec::new();
    let concurrent_limit = Arc::new(Semaphore::new(300)); // Limit concurrent tasks to 100

    for proxy in unique_proxies {
        let valid_proxies = Arc::clone(&valid_proxies);
        let local_ip = local_ip.clone();
        let concurrent_limit = Arc::clone(&concurrent_limit);
        let task = tokio::spawn(async move {
            let _permit = concurrent_limit.acquire().await;
            if let Some(valid_proxy) = validate_proxy(proxy, &local_ip).await {
                valid_proxies.lock().unwrap().push(valid_proxy);
            }
        });
        tasks.push(task);
    }

    futures::future::join_all(tasks).await;

    let valid_proxies = valid_proxies.lock().unwrap();
    println!("Proxies found: {}", valid_proxies.len());
    let mut file = File::create("working_proxies.txt")?;
    for proxy in &*valid_proxies {
        writeln!(file, "{}", proxy)?;
    }
    println!("{:?}",start.elapsed());
    Ok(())
}
fn open_text(filename: &str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    reader.lines().map(|line| line.unwrap()).collect()
}




async fn validate_proxy(proxy: String, local_ip: &str) -> Option<String> {
    let client = Client::builder()
        .proxy(reqwest::Proxy::all(&proxy).ok()?)
        .build()
        .ok()?;

    let result = client
        .get("https://ipinfo.io/json")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await;

    match result {
        Ok(response) => match response.status() {
            StatusCode::OK => {
                let json_response: Value = response.json().await.ok()?;
                if local_ip == json_response.get("ip")?.as_str()? {
                    println!("Leaking");
                    None
                } else {
                    println!("{:?}", json_response);
                    Some(proxy)
                }
            }
            StatusCode::TOO_MANY_REQUESTS => {
                println!("429 get lit");
                None
            }
            _ => None,
        },
        Err(_) => None,
    }
}


