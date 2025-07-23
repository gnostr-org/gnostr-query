use serde_json::{json, Map};

pub fn build_gnostr_query(
    authors: Option<String>,
    ids: Option<String>,
    limit: Option<i32>,
    generic: Option<(&str, &str)>,
    hashtag: Option<&str>,
    mentions: Option<&str>,
    references: Option<&str>,
    kinds: Option<Vec<i64>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut filt = Map::new();

    if let Some(authors) = authors {
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    }

    let authors = filt.get("authors");
    println!("{:?}", authors);
    if let Some(ids) = ids {
        filt.insert(
            "ids".to_string(),
            json!(ids.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(limit) = limit {
        filt.insert("limit".to_string(), json!(limit));
    }

    if let Some((tag, val)) = generic {
        let tag_with_hash = format!("#{}", tag);
        filt.insert(tag_with_hash, json!(val.split(',').collect::<Vec<&str>>()));
    }

    if let Some(hashtag) = hashtag {
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(mentions) = mentions {
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(references) = references {
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(kinds) = kinds {
        let kind_ints: Result<Vec<i64>, Box<dyn std::error::Error>> = Ok(kinds);
        match kind_ints {
            Ok(kind_ints) => {
                filt.insert("kinds".to_string(), json!(kind_ints));
            }
            Err(_) => {
                return Err("Error parsing kinds. Ensure they are integers.".into());
            }
        }
    }

    for filter in filt.clone() {
        println!("filter:{:?}", filter.clone());
    }
    let q = json!(["REQ", "gnostr-query", filt]);
    println!("{}", serde_json::to_string(&q)?);
    Ok(serde_json::to_string(&q)?)
}
