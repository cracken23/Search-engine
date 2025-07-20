use lol_html::{element, rewrite_str, text, HtmlRewriter, RewriteStrSettings, Settings};
use reqwest::{
     header::{HeaderMap, HeaderValue, USER_AGENT}, Client
};


use std::{collections::{HashMap, HashSet}, str};
use tokenizers::tokenizer::{Result, Tokenizer};

use human_regex::{exactly, one_or_more, or, punctuation, whitespace, word_boundary};
use stop_words::{get as sget, LANGUAGE};
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
    output
}
fn extract_text(text: String, is_html: bool) -> Vec<String> {
    if is_html {
        let mut extracted_text = Vec::new();
        rewrite_str(
            &text,
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
fn process_corpus(document:String)->String{
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
    // Remove stop words
    let clean_text = regex_for_stop_words
        .to_regex()
        .replace_all(&text_without_punctuation, "");
    clean_text.to_string()

}
async fn process() {
    let tokenizer=create_tokenizer().unwrap();
    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let url = "https://courses.cs.washington.edu/courses/cse390c/24wi/lectures/moby.txt";

    let mut is_html = true;
    if url.ends_with(".txt") {
        is_html = false;
    }

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

    let output = clean_html(body);

    //println!("{:?}", output);

    let text = extract_text(output, is_html).join(" ");
    let cleaned_text=process_corpus(text);
    let tokens=cleaned_text.split_ascii_whitespace();
    let  Tf_score:HashMap<&str,i32>=tokens.fold(HashMap::new(), |mut acc,word|{
        acc.entry(word).and_modify(|x| *x+=1).or_insert(1);
        acc
    });
    println!("Hi");
    println!("{:?}",Tf_score["moby"]);

    // match tokenizer.encode(cleaned_text, true) {
    //     Ok(tokens)=>{
    //         println!("{:?}",&tokens.get_tokens()[..100]);
    //     },
    //     Err(e)=>{
    //         panic!("{}",e);
    //     }
    // }


    
    
    
}

#[tokio::main]
async fn main() {
    process().await;
}
