use clap::Parser;
use serde::{Deserialize, Serialize};
use stac::{Collection, Item, Link};

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

// replace with stac-rs struct when links is made public
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ItemCollection {
    /// The type field.
    ///
    /// Must be set to "FeatureCollection".
    r#type: String,

    /// The list of [Items](Item).
    ///
    /// The attribute is actually "features", but we rename to "items".
    #[serde(rename = "features")]
    items: Vec<Item>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    links: Vec<Link>,

    #[serde(skip)]
    href: Option<String>,
}

struct SearchResultItems {
    url: Option<String>,
    items: <Vec<Item> as IntoIterator>::IntoIter,
    client: reqwest::blocking::Client,
}

impl SearchResultItems {
    fn of(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(SearchResultItems {
            url: Some(url.to_owned()),
            items: vec![].into_iter(),
            client: reqwest::blocking::Client::new(),
        })
    }

    fn try_next(&mut self) -> Result<Option<Item>, Box<dyn std::error::Error>> {
        if let Some(item) = self.items.next() {
            return Ok(Some(item));
        }

        if let Some(s) = &self.url {
            let item_collection: ItemCollection = self.client.get(s).send()?.json()?;
            if item_collection.items.len() > 0 {
                self.items = item_collection.items.into_iter();
                self.url = item_collection
                    .links
                    .iter()
                    .find(|&x| x.rel == "next")
                    .map(|x| x.href.clone());
                // println!("item: {:?}", self.url);
            } else {
                self.url = None;
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        Ok(self.items.next())
    }
}

impl Iterator for SearchResultItems {
    type Item = Result<Item, Box<dyn std::error::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(dep)) => Some(Ok(dep)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

pub fn run(collection_url: &str, src: &str) -> Result<(), Box<dyn std::error::Error>> {

    let (collections_url, collection_id) = match collection_url.rsplit_once('/') {
        Some((a, b)) => (a, b),
        _ => panic!("Could not parse dst collection URI for collection ID"),
    };

    let client = reqwest::blocking::Client::new();

    let res = client
        .post(collections_url)
        .json(&Collection::new(collection_id, collection_id))
        .send();

    match res {
        // todo: handle 409
        Err(e) => panic!(
            "Could not create dst collection: {:?}",
            e
        ),
        _ => (),
    }

    let collection_items_url = format!("{collection_url}/items");

    for result in SearchResultItems::of(&src)? {
        match result {
            Ok(item) => {
                match client.post(&collection_items_url).json(&item).send() {
                    Err(e) => panic!("Failure creating new item: {:?}", e),
                    _ => println!("Success creating item: {}", item.id),
                }
            }
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
