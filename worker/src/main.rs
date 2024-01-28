use std::io::Cursor;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use dotenv::dotenv;
use tempfile::{Builder, NamedTempFile};
use rodio::{Decoder, OutputStream, source::Source};


use std::fs::File;
use std::io::Write;
use std::path::Path;


use data::{
    models::requests::{NihongoWordReq, NihongoWordReqWord},
    services::data::get_unprocessed_words
};

const VOICE_ID: &str = "IKne3meq5aSn9XLyUdCD";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenv().ok();
    let eleven_labs_key = std::env::var("ELEVEN_LABS_KEY")?;
    // let bytes = generate_audio(eleven_labs_key).await?;

    // let path = format!("/home/dan/.local/share/Anki2/User 1/collection.media/[sound:lang_crack_audio_{}.mp3]");
    // let path = Path::new(&path);

    // save_mp3(bytes, path)?;
    
    loop {
        let up_words = get_unprocessed_words().await?;
        println!("words: {:?}", up_words);

        sleep(Duration::from_secs(120)).await;
    }

    // add_card_anki(path).await?;
}

async fn generate_audio(api_key: String) -> Result<Vec<u8>> {
    let target = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", VOICE_ID);
    let client = reqwest::Client::new();

    let bytes = client.post(target)
        .header("xi-api-key", api_key)
        .json(&serde_json::json!({
              "model_id": "eleven_multilingual_v2",
              "text": "私は",
              "voice_settings": {
                "similarity_boost": 0.7,
                "stability": 0.5
              }
        }))
        .send()
        .await?
        .bytes()
        .await?;

    Ok(bytes.to_vec())
}

fn save_mp3(bytes: Vec<u8>, path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

async fn add_card_anki(audio_path: &Path) -> Result<()> {
    let res: Value = Client::new()
        .post("http://localhost:8765")
        .json(&serde_json::json!({
            "action": "addNote",
            "version": 6,
            "params": {
                "note": {
                    "deckName": "Testing",
                    "modelName": "Danyo Custom",
                    "fields": {
                        "Expression": "Bar",
                        "Meaning": "Foo",
                        "Reading": "Foo",
                        "Sentence": "Foo",
                        "Notes": "Foo",
                        "Tenses": "Foo",
                        "Audio": audio_path,
                        "Kanji Mnemonic": "Foo",
                        "Spoken Mnemonic": "Foo"
                    },
                   "options": {
                        "allowDuplicate": false,
                        "duplicateScope": "deck",
                        "duplicateScopeOptions": {
                            "deckName": "Default",
                            "checkChildren": false,
                            "checkAllModels": false
                        }
                    },
                    "tags": [
                        "lang-crack"
                    ]
                } 
            }
        }))
        .send()
        .await?
        .json()
        .await?;

    println!("res: {:#?}", res);

    Ok(())
}
