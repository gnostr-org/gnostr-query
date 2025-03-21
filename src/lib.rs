use serde_json::{json, Map};

pub fn build_gnostr_query(
    authors: Option<&str>,
    ids: Option<&str>,
    limit: Option<i32>,
    generic: Option<(&str, &str)>,
    hashtag: Option<&str>,
    mentions: Option<&str>,
    references: Option<&str>,
    kinds: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut filt = Map::new();

    if let Some(authors) = authors {
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    }

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
        let kind_ints: Result<Vec<i64>, _> = kinds.split(',').map(|s| s.parse::<i64>()).collect();
        match kind_ints {
            Ok(kind_ints) => {
                filt.insert("kinds".to_string(), json!(kind_ints));
            }
            Err(_) => {
                return Err("Error parsing kinds. Ensure they are integers.".into());
            }
        }
    }

    let q = json!(["REQ", "gnostr-query", filt]);
    Ok(serde_json::to_string(&q)?)
}
