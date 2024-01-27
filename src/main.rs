use lambda_http::{run, service_fn, Body, Request, Response};
use anyhow::Result;
use serde::{Serialize, Deserialize};
use dotenv::dotenv;

mod services;

use crate::services::data::{add_word, add_word_tense};

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

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
    let body: NihongoWordReq = serde_json::from_slice(body.as_ref())?;

    tracing::info!("Body: {:?}", body); 

    for w in &body.words {
        add_to_table(w.clone()).await?;
    }

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("".into())
        .map_err(Box::new)?;

    Ok(resp)
}

async fn add_to_table(word: NihongoWordReqWord) -> Result<()> {
    let id = add_word(&word).await?;
    add_word_tense(id, word.word_tenses).await?;

    Ok(())
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordReq {
    pub words: Vec<NihongoWordReqWord>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordReqWord {
    pub word: String,
    pub definition: String,
    pub sentence: String,
    pub word_tenses: Vec<NihongoWordReqTense>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordReqTense {
    pub word: String,
    pub sentence: String,
    pub tense_type: String
}
