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
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub fn paragraph_from_json_colon_split(json_string: &str) -> Paragraph {
    let text = json_colon_split_to_text(json_string);
    Paragraph::new(text)
}

fn json_colon_split_to_text(json_string: &str) -> Text {
    let mut spans = Vec::new();
    let mut current_key = String::new();
    let mut in_quotes = false;
    let mut escape_next = false;

    for char in json_string.chars() {
        print!("{}", char);
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
                current_key.push(char);
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
    )?;

    let relay_url_str = matches.get_one::<String>("relay").unwrap();
    let relay_url = Url::parse(relay_url_str)?;
    let (ws_stream, _) = connect_async(relay_url).await?;
    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Text(query_string.clone())).await?;

    // Ratatui setup
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (tx, mut rx) = mpsc::channel(100);
    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            if let Ok(data) = message {
                if let Message::Text(text) = data {
                    //This is the entry point to wrangle json data
                    //println!("text={}", text);
                    let paragraph = paragraph_from_json_colon_split(&text);
                    //f.render_widget(paragraph, chunks[0]); // chunks is an array of Rects

                    //

                    //

                    //
                    //
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
        terminal.draw(|f| {
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
                        ListItem::new("TODO: handle EOSE")
                    } else {
                        let mut paragraph = paragraph_from_json_colon_split(&msg);

                        f.render_widget(paragraph, chunks[0]); // chunks is an array of Rects
                        ListItem::new("render Paragraph here?")
                        //ListItem::new(msg.clone())
                    }
                })
                .collect();
            let list = List::new(items)
                .block(Block::default().title("Messages").borders(Borders::ALL))
                .highlight_symbol(">> ");
            //f.render_stateful_widget(list, chunks[0], &mut list_state);

            let query_paragraph = Paragraph::new(query_string.clone())
                .block(Block::default().title("Query").borders(Borders::ALL));
            f.render_widget(query_paragraph, chunks[1]);
        })?;
        if let Ok(msg) = rx.try_recv() {
            let paragraph = paragraph_from_json_colon_split(&msg);
            //f.render_widget(paragraph, chunks[0]); // chunks is an array of Rects

            messages.push(msg);
            list_state.select(Some(messages.len() - 1));
        }
    }
}
