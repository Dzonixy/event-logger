use env_logger::{Builder, WriteStyle};
use event_logger_sc::Milica;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, LevelFilter};
use serde_json::Value;
use solana_sdk::borsh0_10::try_from_slice_unchecked;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    Builder::new()
        .filter(
            None,
            filter_level
                .parse::<LevelFilter>()
                .unwrap_or(LevelFilter::Info),
        )
        .write_style(WriteStyle::Always)
        .init();

    let url = "ws://localhost:8900";
    let (stream, _) = connect_async(url).await?;
    let (mut write, mut read) = stream.split();

    let request = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "programSubscribe",
        "params": [
            "58cEU9rk6h7EDubCp4HA7tX2yWdNuC8NqU5nU2EDBqUW",
            {
                "encoding": "jsonParsed"
            }
        ]
    }"#;
    write.send(Message::Text(request.to_string())).await?;

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let v: Value = serde_json::from_str(text.as_str())?;
                let account = extract_account(&v);
                let _pubkey = extract_pubkey(&v);

                let string_data = &account["data"][0];

                if let Value::String(data) = &string_data {
                    let data_bytes = base64::decode(data)?;
                    match data_bytes[0..8] {
                        [152, 254, 7, 141, 166, 92, 84, 200] => {
                            let deserialized_data =
                                try_from_slice_unchecked::<Milica>(&data_bytes[8..]);

                            // TODO: Deposit to database
                            info!("{:#?}", deserialized_data)
                        }
                        _ => todo!(),
                    }
                }
            }
            Ok(Message::Binary(bin)) => info!("Received binary: {:?}", bin),
            Err(e) => error!("Error receiving message: {:?}", e),
            _ => {}
        }
    }
    Ok(())
}

pub fn extract_account(value: &Value) -> &Value {
    &value["params"]["result"]["value"]["account"]
}

pub fn extract_pubkey(value: &Value) -> &Value {
    &value["params"]["result"]["value"]["pubkey"]
}
