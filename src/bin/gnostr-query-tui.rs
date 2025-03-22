use clap::{Arg, Command};
use futures::{SinkExt, StreamExt};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use serde::de::Error as SerdeError;
use serde_json::{Result, Value};
use shatter::parser::Parser;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

//fn bytes_to_ascii(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
//fn bytes_to_ascii(bytes: &[u8]) -> Result<String, serde_json::Error> {
fn bytes_to_ascii(bytes: &[u8]) -> String {
    // to_vec is needed to create an owned vec
    String::from_utf8(bytes.to_vec()).expect("")
}

fn extract_elements(json_str: &str, keys: &[&str]) -> Result<Value> {
    let json: Value = serde_json::from_str(json_str)?;

    match json {
        Value::Object(map) => {
            let mut extracted = serde_json::Map::new();
            for key in keys {
                if let Some(value) = map.get(*key) {
                    extracted.insert(key.to_string(), value.clone());
                }
            }
            Ok(Value::Object(extracted))
        }
        _ => Err(serde_json::Error::custom("Input is not a JSON object")),
    }
}

fn shatter_test() {
    //             v alien  v
    // 00000000: 20f0 9f91 bd23 6861 7368 7461 670a       _....#hashtag.
    let s = " #hashtag ";
    let mut parser = Parser::from_str(s);
    //access
    //println!("{:?}", parser);
    //println!("{:?}", parser.data());
    //println!("parser.data>>{:?}", bytes_to_ascii(parser.data()));

    let mut res = parser.parse_until_char('#');
    //result
    assert_eq!(res, Ok(()));
    //position
    assert_eq!(parser.pos(), 1);

    res = parser.parse_until_char('t');
    //result
    assert_eq!(res, Ok(()));
    //position
    assert_eq!(parser.pos(), 6);
}

pub fn paragraph_from_json_colon_split(json_string: &str) -> Paragraph {
    let text = json_colon_split_to_text(json_string);
    Paragraph::new(text)
}

fn json_colon_split_to_text(json_string: &str) -> Text {
    let mut parser = Parser::from_str(json_string);
    //println!("{:?}", parser);
    //println!("parser.len()={:?}", parser.len());
    //println!(
    //    "bytes_to_ascii:parser.data:{:?} {}",
    //    bytes_to_ascii(parser.data()),
    //    parser.len()
    //);
    let mut spans = Vec::new();
    let mut current_key = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    for char in json_string.chars() {
        //
        //print!("{}", char);
        //
        //std::process::exit(0);
        match char {
            ':' if !in_quotes => {
                spans.push(Span::styled(
                    current_key.trim().to_string(),
                    Style::default().fg(Color::Red),
                ));
                spans.push(Span::raw(": "));
                current_key.clear();
            }
            '"' if !escape_next => {
                in_quotes = !in_quotes;
                current_key.push(char);
            }
            '\\' if in_quotes => {
                escape_next = true;
                current_key.push(char);
            }
            _ => {
                //current_key.push(char);
                escape_next = false;
            }
        }
    }

    if !current_key.is_empty() {
        spans.push(Span::raw(current_key.trim().to_string()));
    }

    Text::from(vec![Line::from(spans)])
}

#[tokio::main]
//async fn main() -> Result<(), Box<dyn std::error::Error>> {
async fn main() -> Result<()> {
    shatter_test();
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
                .help("Limit the number of results"),
        )
        .arg(
            Arg::new("relay")
                .short('r')
                .long("relay")
                .required(false)
                .default_value("wss://relay.damus.io"),
        )
        .get_matches();

    let authors = matches.get_one::<String>("authors").map(|s| s.as_str());
    let ids = matches.get_one::<String>("ids").map(|s| s.as_str());
    let limit = matches.get_one::<i32>("limit").copied();
    let generic = matches.get_many::<String>("generic").and_then(|values| {
        let vec: Vec<&String> = values.collect();
        if vec.len() == 2 {
            Some((vec[0].as_str(), vec[1].as_str()))
        } else {
            None
        }
    });
    let hashtag = matches.get_one::<String>("hashtag").map(|s| s.as_str());
    let mentions = matches.get_one::<String>("mentions").map(|s| s.as_str());
    let references = matches.get_one::<String>("references").map(|s| s.as_str());
    let kinds = matches.get_one::<String>("kinds").map(|s| s.as_str());

    let query_string = gnostr_query::build_gnostr_query(
        authors, ids, limit, generic, hashtag, mentions, references, kinds,
    )
    .expect("");

    let relay_url_str = matches.get_one::<String>("relay").unwrap();
    let relay_url = Url::parse(relay_url_str).expect("");
    let (ws_stream, _) = connect_async(relay_url).await.expect("");

    let (mut write, mut read) = ws_stream.split();

    //query send
    write
        .send(Message::Text(query_string.clone()))
        .await
        .expect("");

    // Ratatui setup
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend).expect("");
    terminal.clear().expect("");

    let (tx, mut rx) = mpsc::channel(100);

    //
    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            if let Ok(data) = message {
                if let Message::Text(text) = data {
                    //This is the entry point to wrangle json data
                    //println!("text={}", text);
                    //let elements = extract_elements(&text,&[&"", &""]);
                    //println!("{:?}", elements);

                    if tx.send(text).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    let mut messages: Vec<String> = Vec::new();
    let mut list_state = ListState::default();

    loop {
        terminal
            .draw(|f| {
                #[allow(deprecated)]
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.size());

                let items: Vec<ListItem> = messages
                    .iter()
                    .map(|msg| {
                        //TODO handle EOSE
                        if msg == "[\"EOSE\",\"gnostr-query\"]" {
                            //let mut paragraph = paragraph_from_json_colon_split(&msg);

                            //f.render_widget(paragraph, chunks[0]); // chunks is an array of Rects
                            ListItem::new("TODO: handle EOSE")
                        } else {
                            //ListItem::new("{\"string\":\"test\"}")
                            //ListItem::new(String::from("\n\n\nrender Paragraph here?") + &msg.clone())
                            //ListItem::new(String::from("\n\n\n") + &msg.clone() + &String::from("\n\n\n"))
                            ListItem::new(String::from(&msg.clone()))
                        }
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().title("Messages").borders(Borders::ALL))
                    .highlight_symbol(">>--->> ");
                f.render_stateful_widget(list, chunks[0], &mut list_state);

                let query_paragraph = Paragraph::new(query_string.clone())
                    .block(Block::default().title("Query").borders(Borders::ALL));
                f.render_widget(query_paragraph, chunks[1]);
            })
            .expect("draw loop:render_widget query_paragraph");

        if let Ok(msg) = rx.try_recv() {
            let msg_json: String = serde_json::to_string(&msg.clone())?;

            let added_string_1 = "added_string_1";
            let added_int = 30;
            let json_str = &format!(
                r#"{{"added_string_1": "{}", "added_int": {}, "address":{{"street":"1234 street"}},"EVENT": {}}}"#,
                //r#"{{"added_string_1": "{}", "added_int": {}, "address":{{"street":"1234 street"}},"": {}}}"#,
                added_string_1,
                added_int,
                msg.clone() //msg_json.clone()
            );

            let keys_to_extract = &[
                "EVENT",
                "content",
                "string",
                "added_string_1",
                "added_int",
                "city",
                "address",
            ];

            //let keys_to_extract = &["content", "string", "added_string_1", "added_int", "city", "address", "street", "EVENT"];

            match extract_elements(json_str, keys_to_extract) {
                Ok(extracted_json) => {
                    //println!("\n\n\n\n\n\n\n{}\n\n\n\n\n\n\n", extracted_json);

                    let extracted_json: String = serde_json::to_string(&extracted_json)?;

                    messages.push(extracted_json);
                    list_state.select(Some(messages.len() - 1));
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }

            //messages.push(msg);
            //list_state.select(Some(messages.len() - 1));
        }
    }
}
