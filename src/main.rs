use reqwest::{Client, Proxy};
use serde::*;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Duration;
use std::time::Instant;
use tokio::{fs::File as AsyncFile, io::AsyncBufReadExt, io::AsyncWriteExt, task};
#[derive(Debug, Serialize, Deserialize)]
pub struct Welcome {
    ip: String,
}

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

async fn make_vec(filename: String) -> Result<Vec<String>, reqwest::Error> {
    let file = tokio::fs::File::open(filename).await;
    let reader = tokio::io::BufReader::new(file.unwrap());
    let mut lines = reader.lines();
    let mut set = HashSet::new();
    while let Some(line) = lines.next_line().await.transpose() {
        if !set.contains(line.as_ref().unwrap()) {
            set.insert(line.unwrap());
        }
    }
    let vec: Vec<String> = set.into_iter().collect();
    println!("{:?}", vec.len());
    Ok(vec)
}

async fn final_req(proxy_list: Vec<String>) -> Result<(), reqwest::Error> {
    let url = "https://ipconfig.io/json";
    let file = OpenOptions::new()
        .append(true) // open the file in append mode
        .create(true) // create the file if it doesn't exist
        .open("working_proxies.txt")
        .expect("could not open file");
    let file = std::sync::Arc::new(std::sync::Mutex::new(file));
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(300)); // limit to 2000 open files at once
    let pool = tokio::task::LocalSet::new();
    for proxy in proxy_list {
        let file_clone = file.clone();
        let print_proxy = proxy.clone();
        let semaphore_clone = semaphore.clone();
        pool.spawn_local(async move {
            let permit = semaphore_clone.acquire().await.unwrap();
            let proxy = Proxy::https(proxy);
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .pool_idle_timeout(Duration::from_secs(2))
                .pool_max_idle_per_host(5)
                .proxy(proxy.unwrap())
                .build();
            let response = client
                .expect("joe")
                .get(url)
                .header("Accept", "text/plain")
                .timeout(Duration::from_secs(35))
                .send()
                .await
                .expect("joe2")
                .json::<Welcome>()
                .await;
            match response {
                Ok(response) => {
                    let mut file = file_clone.lock().unwrap();
                    file.write(format!("{}\n", print_proxy).as_bytes());
                }
                Err(e) => println!("{:?}", e),
            }
            drop(permit); // release the permit when done with the file
        });
    }
    pool.await;
    Ok(())
}
#[tokio::main]
async fn main() {
    let start = Instant::now();
    download_http_proxies().await.unwrap();
    let my_vec = make_vec("proxies_raw_http.txt".to_string()).await.unwrap();
    let chunk_size = (my_vec.len()) / 3; // round up to the nearest integer
    let mut iter = my_vec.chunks(chunk_size);
    let [vec1, vec2, vec3, vec4] = std::array::from_fn(|_| iter.next().unwrap_or(&[]).to_vec());
   tokio::join!(
        finaly_req(vec1),
        finaly_req(vec2),
        finaly_req(vec3),
        finaly_req(vec4),
    );
    check_duplicates().await;
    let end = Instant::now();
    println!("Took: {:?} seconds", end - start);
}



async fn check_duplicates() -> Result<(), reqwest::Error> {
    let input_file = tokio::fs::File::open("curated_proxy_list.txt").await;
    let mut output_file = OpenOptions::new()
        .append(true) // open the file in append mode
        .create(true) // create the file if it doesn't exist
        .open("working_proxies.txt")
        .expect("could not open file");
    let reader = tokio::io::BufReader::new(input_file.unwrap());
    let mut lines = reader.lines();
    let mut set = HashSet::new();
    while let Some(line) = lines.next_line().await.transpose() {
        if !set.contains(line.as_ref().unwrap()){
            let dupe = line.as_ref().unwrap().clone();
            set.insert(line.unwrap());
            output_file.write(format!("{}\n",dupe).as_bytes());
        }
    }
  
    Ok(())
}








async fn finaly_req(proxy_list: Vec<String>) -> Result<(), reqwest::Error> {
    let url = "https://ipconfig.io/json";
    let mut tasks = vec![];
    let file = OpenOptions::new()
        .append(true) // open the file in append mode
        .create(true) // create the file if it doesn't exist
        .open("curated_proxy_list.txt")
        .expect("could not open file");
    let file = std::sync::Arc::new(std::sync::Mutex::new(file));
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(300)); // limit to 2000 open files at onc
    for proxy in proxy_list {
        let file_clone = file.clone();
        let print_proxy = proxy.clone();
        let semaphore_clone = semaphore.clone();
        let task = tokio::spawn(async move {
            let permit = semaphore_clone.acquire().await.unwrap();
            let proxy = Proxy::https(proxy);
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .pool_idle_timeout(Duration::from_secs(2))
                .pool_max_idle_per_host(5)
                .proxy(proxy.unwrap())
                .build();
            let response = client
                .expect("error")
                .get(url)
                .header("Accept", "text/plain")
                .timeout(Duration::from_secs(35))
                .send()
                .await
                .expect("joe2")
                .json::<Welcome>()
                .await;
            match response {
                Ok(response) => {
                    let mut file = file_clone.lock().unwrap();
                    file.write(format!("{}\n", print_proxy).as_bytes());
                }
                Err(e) => println!("{:?}", e),
            }
            drop(permit); // release the permit when done with the file
        });
        tasks.push(task);
    }
    futures::future::join_all(tasks).await;
    Ok(())
}
