use clap::{Arg, Command};
use futures::{SinkExt, StreamExt};
use gnostr_query::build_gnostr_query;
use log::debug;
use serde_json::{json, to_string, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("gnostr-query")
        .about("Construct nostr queries and send them over a websocket")
        .arg(
            Arg::new("authors")
                .short('a')
                .long("authors")
                .help("Comma-separated list of authors"),
        )
        .arg(
            Arg::new("mentions")
                .short('p')
                .long("mentions")
                .help("Comma-separated list of mentions"),
        )
        .arg(
            Arg::new("references")
                .short('e')
                .long("references")
                .help("Comma-separated list of references"),
        )
        .arg(
            Arg::new("hashtag")
                .short('t')
                .long("hashtag")
                .help("Comma-separated list of hashtags"),
        )
        .arg(
            Arg::new("ids")
                .short('i')
                .long("ids")
                .help("Comma-separated list of ids"),
        )
        .arg(
            Arg::new("kinds")
                .short('k')
                .long("kinds")
                .help("Comma-separated list of kinds (integers)"),
        )
        .arg(
            Arg::new("generic")
                .short('g')
                .long("generic")
                .value_names(["tag", "value"])
                .number_of_values(2)
                .help("Generic tag query: #<tag>: value"),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("limit")
                .value_parser(clap::value_parser!(i32))
                .default_value("500")
                .help("Limit the number of results"),
        )
        .arg(
            Arg::new("relay")
                .short('r')
                .long("relay")
                .required(false)
                //.help("-r wss://relay.damus.io")
                .default_value("wss://relay.damus.io"),
        )
        .get_matches();

    let mut filt = serde_json::Map::new();

    if let Some(authors) = matches.get_one::<String>("authors") {
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    } else {
    }

    let mut limit_check: i32 = 0;
    if let Some(ids) = matches.get_one::<String>("ids") {
        filt.insert(
            "ids".to_string(),
            json!(ids.split(',').collect::<Vec<&str>>()),
        );
        filt.insert("limit".to_string(), json!(1_i32 /*+ 1*/));
        limit_check = 1_i32;
    } else {
        if let Some(limit) = matches.get_one::<i32>("limit") {
            // ["EOSE","gnostr-query"] counts as a message!      + 1
            filt.insert("limit".to_string(), json!(limit.clone() /*+ 1*/));
            limit_check = *limit;
        }
    }

    if let Some(generic) = matches.get_many::<String>("generic") {
        let generic_vec: Vec<&String> = generic.collect();
        if generic_vec.len() == 2 {
            let tag = format!("#{}", generic_vec[0]);
            let val = generic_vec[1].split(',').collect::<Vec<&str>>();
            filt.insert(tag, json!(val));
        }
    }

    if let Some(hashtag) = matches.get_one::<String>("hashtag") {
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(mentions) = matches.get_one::<String>("mentions") {
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(references) = matches.get_one::<String>("references") {
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
    }

    //grab the values from matches
    let mut vec_kind_ints: Vec<i64> = vec![];
    if let Some(kinds) = matches.get_one::<String>("kinds") {
        if let Ok(kind_ints) = kinds
            .split(',')
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<i64>, _>>()
        {
            filt.insert("kinds".to_string(), json!(kind_ints));
            debug!("136:kind_ints={:?}", kind_ints);
            for element in kind_ints.clone() {
                debug!("138:vec_kind_ints element={:?}", element);
                vec_kind_ints.push(element);
            }
        } else {
            eprintln!("Error parsing kinds. Ensure they are integers.");
            std::process::exit(1);
        }
    }

    if let Some(kinds) = filt.get("kinds") {
        debug!("kinds: {}", kinds);
    } else {
        debug!("'kinds' key not found.");
    }

    //pub fn build_gnostr_query(
    //    authors: Option<&str>,
    //    ids: Option<&str>,
    //    limit: Option<i32>,
    //    generic: Option<(&str, &str)>,
    //    hashtag: Option<&str>,
    //    mentions: Option<&str>,
    //    references: Option<&str>,
    //    kinds: Option<Vec<i64>>,
    //) -> Result<String, Box<dyn std::error::Error>>

    let gnostr_query = build_gnostr_query(
        Some(filt.get("authors").unwrap().to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        Some(vec_kind_ints.clone()),
    );
    let v: Value = serde_json::from_str(&gnostr_query.unwrap().clone())?;

    println!("176:{:?}", v);

    println!("v={:?}", v["kinds"]);
    let v: Value = serde_json::from_str(&format!("{:?}", vec_kind_ints.clone()))?;
    println!("v={:?}", v.get("kinds"));

    let q = json!(["REQ", "gnostr-query", filt]);
    let query_string = to_string(&q)?;
    let relay_url_str = matches.get_one::<String>("relay").unwrap();
    let relay_url = Url::parse(relay_url_str)?;
    let (ws_stream, _) = connect_async(relay_url).await?;
    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Text(query_string)).await?;

    let mut count: i32 = 0;
    while let Some(message) = read.next().await {
        let data = message?;
        if count >= limit_check {
            std::process::exit(0);
        }
        if let Message::Text(text) = data {
            print!("{}", text);
            count += 1;
        }
    }

    Ok(())
}
