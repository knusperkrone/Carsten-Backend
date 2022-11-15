use std::sync::RwLock;

use crate::error::ErrorResponse;
use crate::logging::APP_LOGGING;

use actix_web::client::Client;
use chrono::{DateTime, NaiveDateTime, Utc};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

static USER_AGENT: &'static str = "User-Agent";
static REFERER: &'static str = "Referer";

static BASE_URL: &'static str = "https://music.youtube.com/";
static CHROME_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.108 Safari/537.36";
static MUSIC_REFERER: &'static str = "https://music.youtube.com/";

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    id: String,
}

#[derive(Debug, Default, Deserialize)]
struct SearchContext {
    #[serde(rename(deserialize = "INNERTUBE_API_KEY"))]
    key: String,
    #[serde(rename(deserialize = "INNERTUBE_CLIENT_NAME"))]
    client_name: String,
    #[serde(rename(deserialize = "INNERTUBE_CONTEXT_CLIENT_VERSION"))]
    client_version: String,
}

static CACHED_CONTEXT: Lazy<RwLock<(DateTime<Utc>, SearchContext)>> = Lazy::new(|| {
    RwLock::new((
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
        SearchContext::default(),
    ))
});

async fn refresh_context(client: &Client) -> Result<(), ErrorResponse> {
    let now = Utc::now();
    let read_guard = CACHED_CONTEXT.read().unwrap();
    if (now - read_guard.0).num_hours() > 1 {
        drop(read_guard);
        let mut write_guard = CACHED_CONTEXT.write().unwrap();
        // Double-Check-Idiom
        if (now - write_guard.0).num_hours() > 1 {
            let context_html = fetch_config_html(&client).await?;
            let context = scrape_context(context_html)?;
            *write_guard = (Utc::now(), context);
        }
    }
    Ok(())
}

async fn fetch_config_html(client: &Client) -> Result<String, ErrorResponse> {
    let body = client
        .get(BASE_URL)
        .header(USER_AGENT, CHROME_AGENT)
        .send()
        .await?
        .body()
        .limit(1024 * 1024)
        .await?;
    Ok(std::str::from_utf8(&body).unwrap().to_owned())
}

async fn post_search(
    client: &Client,
    context: &SearchContext,
    q: String,
) -> Result<String, ErrorResponse> {
    let url = &format!(
        "{}/youtubei/v1/search?alt=json&key={}",
        BASE_URL, context.key
    );

    let body = serde_json::json!({
        "params": "Eg-KAQwIABABGAAgACgAMABqChAKEAQQAxAFEAk%3D", // prefer video
        "query": q,
        "context": {
            "client": {
                "clientName": context.client_name,
                "clientVersion": context.client_version,
            },
        },
    });

    let resp_body = client
        .post(url)
        .header(USER_AGENT, CHROME_AGENT)
        .header(REFERER, MUSIC_REFERER)
        .send_body(body.to_string())
        .await?
        .body()
        .limit(1024 * 1024)
        .await?;
    Ok(std::str::from_utf8(&resp_body).unwrap().to_owned())
}

fn scrape_search_json(json: String) -> Result<String, ErrorResponse> {
    let needle = "\"videoId\":";
    for line in json.lines() {
        if let Some(index) = line.find(needle) {
            let sliced = line
                .get(index + needle.len() + 2..line.len() - 2)
                .unwrap_or("QryoOF5jEbc");
            return Ok(sliced.into());
        }
    }
    Err(ErrorResponse::new(
        String::from("Scrape Error"),
        String::from("Couln't find scrape JSON"),
    ))
}

fn scrape_context(html: String) -> Result<SearchContext, ErrorResponse> {
    let script_sel: Selector = Selector::parse("script").unwrap();
    let needle = "ytcfg.set(";

    let doc = Html::parse_document(&html);
    let scripts = doc.select(&script_sel);
    for script in scripts {
        for text in script.text() {
            if let Some(index) = text.find(&needle) {
                // Slice begin 'ytcfg.set('
                let mut sliced = text.get(index + needle.len()..text.len() - 2).unwrap_or("");
                // Slice end ');'
                let last_index = sliced.rfind(')').unwrap_or(sliced.len());
                sliced = &sliced[0..last_index];

                eprintln!("{}", sliced);
                let ctx: SearchContext = serde_json::from_str(sliced)?;
                return Ok(ctx);
            }
        }
    }
    Err(ErrorResponse::new(
        String::from("Scrape Error"),
        String::from("Couln't find scrape JS context"),
    ))
}

// Returns video id
pub async fn search(q: String) -> Result<SearchResponse, ErrorResponse> {
    info!(&APP_LOGGING, "Searching track: {}", q);
    let client = Client::new();

    refresh_context(&client).await?;
    let search_json = post_search(&client, &CACHED_CONTEXT.read().unwrap().1, q).await?;
    let id = scrape_search_json(search_json)?;

    Ok(SearchResponse { id: id })
}

#[cfg(test)]
 mod test {
    use super::*;

    #[actix_rt::test]
    async fn test_search() {
        let actual = search("feel good inc - gorillaz".to_owned()).await;
        assert!(actual.is_ok());
        assert_eq!("HyHNuVaZJ-k", actual.unwrap().id);
    }
}
