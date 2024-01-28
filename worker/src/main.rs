use anyhow::Result;
use dotenv::dotenv;
use tokio::time::{sleep, Duration};

use data::{
    models::requests::{NihongoWordReq, NihongoWordReqWord},
    services::data::get_unprocessed_words
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    
    loop {
        let unproc_words = get_unprocessed_words().await?;
        sleep(Duration::from_secs(120)).await;
    }

    Ok(())
}
