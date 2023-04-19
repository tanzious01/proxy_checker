use reqwest::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use futures::future::{ok, join_all};
use serde::*;
use std::result::Result;
use std::iter::Cycle;
use std::time::Instant;
use rayon::{prelude::*, range};
use std::path::Path;
use std::io::{copy, Write};
use std::fs;
use tokio::task;
use std::str::FromStr;
use reqwest::{Client, Proxy};
use std::fs::read_to_string;
use tokio::fs::File as AsyncFile;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt};
use tokio::io::AsyncWriteExt;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::collections::HashSet;
use std::fs::OpenOptions;
#[derive(Debug, Serialize, Deserialize)]
pub struct Welcome {
    ip: String,
}






async fn download_http_proxies() -> Result<(), reqwest::Error>{
    let url1 = "https://raw.githubusercontent.com/MuRongPIG/Proxy-Master/main/http.txt";
    let url2 = "https://api.proxyscrape.com/v2/?request=getproxies&amp;protocol=http&amp;timeout=10000&amp;country=all&amp;ssl=all&amp;anonymity=all";
    let url3 = "https://raw.githubusercontent.com/hookzof/socks5_list/master/proxy.txt";
    let url4 = "https://raw.githubusercontent.com/ShiftyTR/Proxy-List/master/http.txt";
    let url5 = "https://raw.githubusercontent.com/ShiftyTR/Proxy-List/master/https.txt";
    let url6 = "https://raw.githubusercontent.com/saisuiu/Lionkings-Http-Proxys-Proxies/main/awkpro2.txt";
    let url7 ="https://raw.githubusercontent.com/ALIILAPRO/Proxy/main/http.txt";
 

    let url_vec = vec![url1,url2,url3,url4,url5,url6,url7];
    let output_file = "proxies_raw_http.txt";
    let output_path = Path::new(output_file);
    let mut out_file = AsyncFile::create(output_path).await.unwrap();
    for i in url_vec{
        let response = reqwest::get(i).await?;
        let body = response.text().await?;
        out_file.write_all(body.as_bytes()).await;
    }

        
        Ok(())
    }
    


async fn make_vec(filename:String)->Result<Vec<String>,reqwest::Error>{
    let file = tokio::fs::File::open(filename).await;
    let reader = tokio::io::BufReader::new(file.unwrap());
    let mut lines = reader.lines();
    let mut set = HashSet::new();
    while let Some(line) = lines.next_line().await.transpose(){
        if !set.contains(line.as_ref().unwrap()){
            set.insert(line.unwrap());
        }

    }
    let vec: Vec<String> = set.into_iter().collect();
    println!("{:?}",vec.len());
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
                    file.write(format!("{}\n",print_proxy).as_bytes());
                },
                Err(e) => println!("{:?}",e),
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
    let chunk_size = (my_vec.len())/3; // round up to the nearest integer
    let chunks = my_vec.chunks(chunk_size);
    let mut vec1 = Vec::new();
    let mut vec2 = Vec::new();
    let mut vec3 = Vec::new();
    let mut vec4 = Vec::new();
    for (i, chunk) in chunks.enumerate() {
        match i {
            0 => vec1 = chunk.to_vec(),
            1 => vec2 = chunk.to_vec(),
            2 => vec3 = chunk.to_vec(),
            3 => vec4 = chunk.to_vec(),
            _ => unreachable!(),
        }
    }
  tokio::join!(
      final_req(vec1),
      final_req(vec2),
      final_req(vec3),
      final_req(vec4),
      );
  let end = Instant::now();
  println!("Took: {:?} seconds", end-start);
 }

