mod utils;
use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client,
};

use std::{
    collections::{HashMap, HashSet}, fs::File, io::Write, path::Path, str
};
use tokenizers::tokenizer::{Result, Tokenizer};

use human_regex::{exactly, one_or_more, or, punctuation, whitespace, word_boundary};
use stop_words::{get as sget, LANGUAGE};
use utils::{is_binary_extension, is_text_content};

use crate::utils::Store;
fn clean_html(body: String) -> String {
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
    //println!("{}",&output[..100]);
    output
}
fn extract_text(text: String) -> Vec<String> {
    let text =text.trim();
    if text.ends_with("</html>") {
        let mut extracted_text = Vec::new();
        rewrite_str(
            text,
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
        extracted_text
    } else {
        text.lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect()
    }
}
fn create_tokenizer() -> Result<Tokenizer> {
    let tokenizer = Tokenizer::from_pretrained("bert-large-cased", None)?;
    Ok(tokenizer)
}
fn process_corpus(document: String) -> String {
    let words = sget(LANGUAGE::English);

    // Remove punctuation and lowercase the text to make parsing easier
    let lowercase_doc = document.to_ascii_lowercase();
    let regex_for_punctuation = one_or_more(punctuation());
    let text_without_punctuation = regex_for_punctuation
        .to_regex()
        .replace_all(&lowercase_doc, "");
    // Make a regex to match stopwords with trailing spaces and punctuation
    let regex_for_stop_words =
        word_boundary() + exactly(1, or(&words)) + word_boundary() + one_or_more(whitespace());
    //println!("{}", regex_for_stop_words.to_regex());
    // Remove stop words
    let clean_text = regex_for_stop_words
        .to_regex()
        .replace_all(&text_without_punctuation, "");
    clean_text.to_string()
}

async fn get_data(client: &Client, url: &str) -> String {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let res = client.get(url).headers(headers).send().await;
    let body: String;
    match res {
        Ok(val) => {
            body = val.text().await.unwrap();
        }
        Err(e) => {
            panic!("{}", e);
        }
    }
    body
}

async fn check_content_type(client: &Client, url: &str) -> bool {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let head_response = client.head(url).headers(headers.clone()).send().await;
    match head_response {
        Ok(response) => {
            if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
                let content_type_str = content_type.to_str().unwrap_or("");
                if !is_text_content(content_type_str) {
                    println!(
                        "Skipping non-text content: {} (Content-Type: {})",
                        url, content_type_str
                    );
                    return false;
                }
            }

            // Check content length to avoid very large files
            if let Some(content_length) = response.headers().get("content-length") {
                if let Ok(length_str) = content_length.to_str() {
                    if let Ok(length) = length_str.parse::<u64>() {
                        const MAX_SIZE: u64 = 10 * 1024 * 1024; // 10MB limit
                        if length > MAX_SIZE {
                            println!("Skipping large file: {} ({} bytes)", url, length);
                            return false;
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to get HEAD response: {}", e);
            return false;
        }
    }
    true
}

async fn process(url:& str) ->Result<Store> {
    let tokenizer = create_tokenizer().unwrap();

    //let url = "https://courses.cs.washington.edu/courses/cse390c/24wi/lectures/moby.txt";

    let client = Client::new();

    if is_binary_extension(url) || !check_content_type(&client, url).await {
        return Err("Check Failed".into());
    }

    let output = clean_html(get_data(&client, url).await);

    let text = extract_text(output).join(" ");
    let cleaned_text = process_corpus(text);
    let tokens = cleaned_text.split_ascii_whitespace();
    let tf_score: HashMap<String, i32> = tokens.fold(HashMap::new(), |mut acc, word| {
        acc.entry(word.to_string()).and_modify(|x| *x += 1).or_insert(1);
        acc
    });

    //println!("{:?}", tf_score["moby"]);

    Ok(Store::new(url, tf_score))
}

#[tokio::main]
async fn main()->Result<()> {
    let urls=["https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html","https://courses.cs.washington.edu/courses/cse390c/24wi/lectures/moby.txt","https://www.popsci.com/technology/ai-math-competition/?utm_source=firefox-newtab-en-intl"];
    let mut stores:Vec<Store>=Vec::new();
    for url in urls{
        stores.push(process(url).await?);
    }
    let mut global_count:HashMap<&str,i32>=HashMap::new();
    for store in &stores{
        for word in store.tf_score.keys(){
            global_count.entry(word).and_modify(|x| *x+=1).or_insert(1);
        }

    }
    let mut idf:HashMap<&str,f32>=HashMap::new();
    let doc_num=urls.len() as f32 +1.0;
    let mut mn=f32::MAX;
    let mut mx=f32::MIN;
    for (&word,count) in global_count.iter(){
        idf.entry(word).or_insert(f32::log10(doc_num/(*count as f32 +1.0))+1.0);
        mn=mn.min(f32::log10(doc_num/(*count as f32 +1.0))+1.0);
        mx=mx.max(f32::log10(doc_num/(*count as f32 +1.0))+1.0);
    }
    println!("max:{},min:{}",mx,mn);
    let jsn=serde_json::to_string(&idf).unwrap();

    let path=Path::new("./out.json");
    let mut file=match File::create(&path){
        Ok(file)=>{
            file
        },Err(e)=>{
            panic!("File creation failed:{}",e);
        }
    };

    match  file.write(&jsn.as_bytes()) {
        Ok(_)=>{
            print!("File creation success")
        },
        Err(_)=>{panic!("file failed")
    }
    };
    Ok(())
}
