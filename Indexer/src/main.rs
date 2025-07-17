use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use reqwest::Client;
use std::str;

async fn process() {
    let client = Client::new();
    let res = client
        .get("https://www.reddit.com/r/rust/comments/1i6v2z1/comparing_13_rust_crates_for_extracting_text_from/")
        .send()
        .await;
    let body: String;
    match res {
        Ok(val) => {
            body = val.text().await.unwrap();
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    let mut extracted_text = Vec::new();
    let mut output = String::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("script", |el| {
                    el.remove();
                    Ok(())
                }),
                element!("svg", |el| {
                    el.remove();
                    Ok(())
                }),
                element!("style", |el| {
                    el.remove();
                    Ok(())
                }),
            ],
            ..Settings::new()
        },
        |out: &[u8]| output.extend(str::from_utf8(out)),
    );
    rewriter.write(body.as_bytes()).unwrap();
    rewriter.end().unwrap();
    //println!("{:?}", output);
    rewrite_str(
        &output,
        RewriteStrSettings {
            element_content_handlers: vec![text!("*", |t| {
                let text_content = t.as_str().trim();
                if !text_content.is_empty() {
                    extracted_text.push(text_content.to_string());
                }
                Ok(())
            })],
            ..RewriteStrSettings::new()
        },
    )
    .unwrap();
    println!("{:?}", extracted_text);
}
#[tokio::main]
async fn main() {
    process().await;
}
