use lambda_http::{run, service_fn, Body, Request, Response};
use anyhow::Result;
use dotenv::dotenv;
use serde_json::Value;
use data::{
    models::requests::{NihongoWordReq, NihongoWordReqWord},
    services::data::{add_word, add_word_tense}
};

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

    match serde_json::from_slice::<NihongoWordReq>(body.as_ref()) {
        Ok(b) => {
            println!("Body: {:?}", b);
            for w in &b.words {
                add_to_table(w.clone()).await?;
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

async fn add_to_table(word: NihongoWordReqWord) -> Result<()> {
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
