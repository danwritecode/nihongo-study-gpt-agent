use lambda_http::{run, service_fn, Body, Request, Response};

use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{Client, types::AttributeValue};

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::Utc;


const TABLE_NAME: String = "NihongoYo".to_string();

async fn function_handler(event: Request) -> Result<Response<Body>, lambda_http::Error> {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-2");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let client = Client::new(&config);

    match event.method().as_str() {
        "POST" => {
            let resp = post_handler(event, client).await?;
            Ok(resp)
        },
        "GET" => {
            let resp = get_handler(event, client).await?;
            Ok(resp)
        },
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


async fn post_handler(event: Request, client: aws_sdk_dynamodb::Client) -> Result<Response<Body>, lambda_http::Error> {
    let body = event.body();
    let body: NihongoSaveTextReq  = serde_json::from_slice(body.as_ref())?;

    add_to_table(body.text, client).await?;

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("".into())
        .map_err(Box::new)?;

    Ok(resp)
}

async fn add_to_table(text: String, client: aws_sdk_dynamodb::Client) -> Result<(), aws_sdk_dynamodb::Error> {
    let id = Uuid::new_v4();
    let id = AttributeValue::S(id.to_string());
    let text = AttributeValue::S(text);
    let status = AttributeValue::S("unreviewed".to_string());
    let created_on = AttributeValue::S(Utc::now().to_string());

    let request = client
        .put_item()
        .table_name(TABLE_NAME)
        .item("id", id)
        .item("text", text)
        .item("created_on", created_on)
        .item("status", status);

    request.send().await?;

    Ok(())
}

async fn get_handler(event: Request, client: aws_sdk_dynamodb::Client) -> Result<Response<Body>, lambda_http::Error> {
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("".into())
        .map_err(Box::new)?;

    Ok(resp)
}

pub async fn get_rand_record(client: aws_sdk_dynamodb::Client) -> Result<(), aws_sdk_dynamodb::Error> {
    let results = client
        .query()
        .table_name(TABLE_NAME)
        .key_condition_expression("#st = :status_val")
        .expression_attribute_names("#st", "status")
        .expression_attribute_values(":status_val", AttributeValue::S("unreviewed".to_string()))
        .send()
        .await?;

    if let Some(items) = results.items {
        let movies = items.iter().map(|v| v.into()).collect();
        Ok(())
    } else {
        Ok(())
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NihongoSaveTextReq {
    pub text: String,
}
