use crate::error::ErrorResponse;
use crate::logging::APP_LOGGING;

use reqwest::header::{REFERER, USER_AGENT};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

static BASE_URL: &'static str = "https://music.youtube.com/";
static CHROME_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/78.0.3904.108 Safari/537.36";
static MUSIC_REFERER: &'static str = "https://music.youtube.com/";

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct SearchContext {
    #[serde(rename(deserialize = "INNERTUBE_API_KEY"))]
    key: String,
    #[serde(rename(deserialize = "INNERTUBE_CLIENT_NAME"))]
    client_name: String,
    #[serde(rename(deserialize = "INNERTUBE_CONTEXT_CLIENT_VERSION"))]
    client_version: String,
}

async fn get_config_html(client: &reqwest::Client) -> Result<String, ErrorResponse> {
    Ok(client
        .get(BASE_URL)
        .header(USER_AGENT, CHROME_AGENT)
        .send()
        .await?
        .text()
        .await?)
}

async fn post_search(
    client: &reqwest::Client,
    context: SearchContext,
    q: String,
) -> Result<String, ErrorResponse> {
    let formated = &format!(
        "{}/youtubei/v1/search?alt=json&key={}",
        BASE_URL, context.key
    );

    let url = reqwest::Url::parse(formated).unwrap();
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

    Ok(client
        .post(url)
        .header(USER_AGENT, CHROME_AGENT)
        .header(REFERER, MUSIC_REFERER)
        .body(body.to_string())
        .send()
        .await?
        .text()
        .await?)
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
        let text = script.text().next().unwrap();
        if let Some(index) = text.find(&needle) {
            let sliced = text.get(index + needle.len()..text.len() - 2).unwrap_or("");
            let ctx: SearchContext = serde_json::from_str(sliced)?;
            return Ok(ctx);
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
    let client = reqwest::Client::new();

    let context_html = get_config_html(&client).await?;
    let context = scrape_context(context_html)?;
    let search_json = post_search(&client, context, q).await?;
    let id = scrape_search_json(search_json)?;

    Ok(SearchResponse { id: id })
}
