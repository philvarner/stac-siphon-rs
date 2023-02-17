#![deny(unused_extern_crates)]

use clap::Parser;
use futures_util::stream::StreamExt;
use stac::Collection;
use stac_async::{ApiClient, Client};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub src: String,

    #[arg(short, long)]
    pub dst: String,

    #[arg(short, long, default_value_t = true)]
    bulk: bool,
}

pub async fn run(collection_url: &str, src: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (collections_url, collection_id) = match collection_url.rsplit_once('/') {
        Some((a, b)) => (a, b),
        _ => panic!("Could not parse dst collection URI for collection ID"),
    };

    let client = Client::new();
    let _: Option<()> = client
        .post(
            collections_url,
            &Collection::new(collection_id, collection_id),
        )
        .await?;

    let collection_items_url = format!("{collections_url}/items");
    let api_client = ApiClient::new(src)?;
    let items = api_client.items(collection_id, None).await?;
    tokio::pin!(items);
    while let Some(result) = items.next().await {
        match result {
            Ok(item) => match client
                .post::<_, ()>(collection_items_url.clone(), &item)
                .await
            {
                Err(e) => panic!("Failure creating new item: {:?}", e),
                _ => println!("Success creating item: {}", item["id"]),
            },
            Err(_) => println!("error"),
        }
    }

    Ok(())
}

// use assert_cmd::Command;

// #[test]
// fn runs() {
//     let mut cmd = Command::cargo_bin("stac-siphon-rs").unwrap();
//     cmd.assert().success();
// }
