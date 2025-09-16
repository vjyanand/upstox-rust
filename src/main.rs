use std::{collections::HashMap, env, fs::File, path::Path};

use futures_util::{StreamExt, sink::SinkExt};
use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::{self, connect_async};
use tungstenite::protocol::Message as WsMessage;
use url::Url;

use crate::market_data::FeedResponse;

pub mod market_data {
    include!(concat!(
        env!("OUT_DIR"),
        "/com.upstox.marketdatafeederv3udapi.rpc.proto.rs"
    ));
}

#[derive(Serialize, Deserialize)]
struct AuthorizeResponse {
    data: AuthorizeData,
}

#[derive(Serialize, Deserialize)]
struct AuthorizeData {
    authorized_redirect_uri: String,
}

#[derive(Debug, serde::Deserialize)] // Requires Serde
struct Instrument {
    name1: String,
    count: u32,
    pvalue: f64,
    name2: String,
    key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = env::var("UPSTOX_ACCESS_TOKEN").expect("Set UPSTOX_ACCESS_TOKEN");

    let filename = "instrucments.csv";
    let file = File::open(Path::new(filename))?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut targets: HashMap<String, Instrument> = HashMap::new();

    for result in rdr.deserialize() {
        let record: Instrument = result?;
        targets.insert(record.key.clone(), record);
    }

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.upstox.com/v3/feed/market-data-feed/authorize")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("Auth failed: {}", response.status()).into());
    }

    let auth_resp: AuthorizeResponse = response.json().await?;
    let ws_url = auth_resp.data.authorized_redirect_uri;

    let (mut ws_stream, _) = connect_async(Url::parse(&ws_url)?).await?;
    println!("Connected to Upstox WS: {}", ws_url);

    let instrument_keys: Vec<String> = targets.keys().cloned().collect();

    let sub_msg = json!({
        "guid": "rust-alert-guid",
        "method": "sub",
        "data": {
            "mode": "ltpc",  // Minimal mode for LTP only; use "full" for more data
            "instrumentKeys": instrument_keys
        }
    });
    let sub_bytes = serde_json::to_string(&sub_msg)?;
    let payload = WsMessage::Binary(sub_bytes.into());
    ws_stream.send(payload).await?;
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(WsMessage::Binary(data)) => {
                let Ok(feed_resp) = FeedResponse::decode(&data[..]) else {
                    break;
                };
                for f in feed_resp.feeds.iter() {
                    let Some(k) = &f.1.feed_union else {
                        continue;
                    };
                    match k {
                        market_data::feed::FeedUnion::Ltpc(ltpc) => {
                            if ltpc.ltp <= 0.0 {
                                let Some(instrument) = targets.get(f.0) else {
                                    continue;
                                };
                                println!("{},{}", instrument.name1, instrument.key);
                            }
                        }
                        market_data::feed::FeedUnion::FullFeed(_) => {}
                        market_data::feed::FeedUnion::FirstLevelWithGreeks(_) => {}
                    }
                }
            }
            Ok(WsMessage::Pong(_)) => {
                println!("Ping");
            }
            Ok(WsMessage::Ping(_)) => {
                println!("Ping");
            }
            Ok(WsMessage::Text(_)) => {
                println!("Text");
            }
            Ok(WsMessage::Frame(_)) => {
                println!("Frame");
            }
            Ok(WsMessage::Close(_)) => {
                println!("Close");
                break;
            }
            Err(e) => {
                eprintln!("WS error: {}", e);
                break;
            }
        }
    }
    Ok(())
}
