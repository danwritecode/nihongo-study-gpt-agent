use std::str::FromStr;

use lambda_http::{run, service_fn, Body, Request, Response, RequestExt};
use anyhow::{Result, anyhow};
use dotenv::dotenv;
use serde_json::Value;
use data::{
    models::{requests::{NihongoWordReqChatgpt, NihongoWordReq}, oai::{Prompt, ModelProvider, NihongoWordOpenAiRes}, db::NihongoWordInsert},
    services::data::{add_word, add_word_tense}
};
use services::oai::ChatAsync;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    dotenv().ok();
    tracing_subscriber::fmt().json()
        .with_max_level(tracing::Level::INFO)
        .with_current_span(false)
        .with_target(false)
        .init();

    run(service_fn(function_handler)).await
}

async fn function_handler(event: Request) -> Result<Response<Body>, lambda_http::Error> {
    match event.method().as_str() {
        "POST" => {
            let resp = post_handler(event).await?;
            Ok(resp)
        }
        _ => {
            let resp = Response::builder()
                .status(405)
                .header("content-type", "text/plain")
                .body("Method Not Allowed".into())
                .map_err(Box::new)?;
            Ok(resp)
        }
    }
}

async fn post_handler(event: Request) -> Result<Response<Body>, lambda_http::Error> {
    let body = event.body();
    let params = event.query_string_parameters();

    let req_type_param = match params.all("req_type") {
        Some(v) => v,
        None => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/plain")
                .body("Invalid or missing req_type".into())
                .map_err(Box::new)?;

            return Ok(resp);
        }
    };

    let req_type: PostWordType = match req_type_param.first().ok_or_else(|| anyhow!("Missing req_type"))?.parse() {
        Ok(v) => v,
        Err(_) => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/plain")
                .body("Invalid or missing req_type".into())
                .map_err(Box::new)?;

            return Ok(resp);
        }
    };

    match req_type {
        PostWordType::ChatGpt => handle_chatgpt_req(body).await,
        PostWordType::SingularWord => handle_singular_req(body).await
    }
}

async fn handle_chatgpt_req(body: &Body) -> Result<Response<Body>, lambda_http::Error> {
    match serde_json::from_slice::<NihongoWordReqChatgpt>(body.as_ref()) {
        Ok(b) => {
            println!("Body: {:?}", b);
            for w in &b.words {
                add_to_table(w.clone().into()).await?;
            }

            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/plain")
                .body("".into())
                .map_err(Box::new)?;

            Ok(resp)
        },
        Err(e) => {
            let body: Value = serde_json::from_slice(body.as_ref())?;
            println!("Failed to deserialize error: {} | request body: {}", e, body);

            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/plain")
                .body("".into())
                .map_err(Box::new)?;

            Ok(resp)
        }
    }
}

async fn handle_singular_req(body: &Body) -> Result<Response<Body>, lambda_http::Error> {
    match serde_json::from_slice::<NihongoWordReq>(body.as_ref()) {
        Ok(w) => {
            let system_prompt = get_system_prompt();
            let user_prompt = get_user_prompt(&w.word);

            let prompt = Prompt {
                system_prompt,
                user_prompt,
                model: "mistral-medium".to_string(),
                provider: ModelProvider::Mistral
            };

            let res: NihongoWordOpenAiRes = ChatAsync::new(prompt).chat_json().await?;
            add_to_table(res.into()).await?;

            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/plain")
                .body("".into())
                .map_err(Box::new)?;

            Ok(resp)
        },
        Err(e) => {
            let body: Value = serde_json::from_slice(body.as_ref())?;
            println!("Failed to deserialize error: {} | request body: {}", e, body);

            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/plain")
                .body("".into())
                .map_err(Box::new)?;

            Ok(resp)
        }
    }
}

fn get_system_prompt() -> String {
    format!("
        YOU RESPOND WITH JSON ONLY NO OTHER WORDS AT ALL BESIDES FOR JSON.
        You are a tool to help users learn Japanese.

        You will be provided with a Japanese word, your job is to do the following:
        
        1. Create a definition of the word
        2. Word reading = Hiragana version of the word
        3. Create an example sentence using the word (using kanji version of the word and fully in japanese)
        4. Create an translation of that sentence
        5. Create a kanji mnemonic for the word (in english)
        6. Create a spoken mnemonic for the word (in english)
        7. Create word tenses 
        
        If word tenses are not needed, return an empty array.
        
        Please respond with the below JSON only, NO OTHER WORDS EXCEPT THIS JSON:
        {{
          'word': <String>,
          'is_kanji': <Boolean>,
          'word_reading': <String>,
          'definition': <String>,
          'sentence': <String>,
          'sentence_translation': <String>,
          'kanji_mnemonic': <String>,
          'spoken_mnemonic': <String>,
          'word_tenses': [
            {{
              'word': <String>,
              'sentence': <String>,
              'tense_type': <String>
            }}
            ...
          ]
        }}
    ")
}

fn get_user_prompt(word: &str) -> String {
    format!("
        Word: {}
    ",  word)
}

async fn add_to_table(word: NihongoWordInsert) -> Result<()> {
    match add_word(&word).await? {
        Some(id) => {
            add_word_tense(id, word.word_tenses).await?;
        },
        None => {
            println!("Word: {} already exists in DB", word.word);
        }
    };

    Ok(())
}


#[derive(Debug)]
enum PostWordType {
    ChatGpt,
    SingularWord
}


#[derive(Debug)]
struct ParseWordTypeError;

impl FromStr for PostWordType {
    type Err = ParseWordTypeError;
 
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "chatgpt" => Ok(PostWordType::ChatGpt),
            "singular_word" => Ok(PostWordType::SingularWord),
            _ => Err(ParseWordTypeError)
        }
    }
}
