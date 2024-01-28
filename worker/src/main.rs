use anyhow::Result;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use dotenv::dotenv;

use std::fs::File;
use std::io::Write;
use std::path::Path;


use data::{
    models::db::NihongoWordWithTenses,
    services::data::{get_unprocessed_words, update_word_status}
};

const VOICE_ID: &str = "IKne3meq5aSn9XLyUdCD";
const DECK_NAME: &str = "Dan's Nihongo Deck";
const DECK_FORMAT: &str = "Danyo Custom";
const BASE_ANKI_MEDIA_DIR: &str = "/home/dan/.local/share/Anki2/User 1/collection.media";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenv().ok();
    let eleven_labs_key = std::env::var("ELEVEN_LABS_KEY")?;
    
    loop {
        let up_words = get_unprocessed_words().await?;
        println!("words: {:?}", up_words);

        for w in &up_words {
            let anki_word_ref = format!("[sound:lang_crack_audio_{}.mp3]", w.word);
            let bytes = generate_audio(&eleven_labs_key, &w.sentence).await?;
            save_file(&w.word, bytes)?;
            add_card_anki(anki_word_ref, w).await?;

            // finally update the word status to processed = true
            update_word_status(w.id).await?;
        }

        sleep(Duration::from_secs(120)).await;
    }
}

fn save_file(path_word: &str, bytes: Vec<u8>) -> Result<()> {
    let path = format!("{}/lang_crack_audio_{}.mp3", BASE_ANKI_MEDIA_DIR, path_word);
    let path = Path::new(&path);
    save_mp3(bytes, path)?;

    Ok(())
}

async fn generate_audio(api_key: &str, text: &str) -> Result<Vec<u8>> {
    let target = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", VOICE_ID);
    let client = reqwest::Client::new();

    let bytes = client.post(target)
        .header("xi-api-key", api_key)
        .json(&serde_json::json!({
              "model_id": "eleven_multilingual_v2",
              "text": text,
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

async fn add_card_anki(
    audio_path: String,
    word: &NihongoWordWithTenses
) -> Result<()> {
    let res: Value = Client::new()
        .post("http://localhost:8765")
        .json(&serde_json::json!({
            "action": "addNote",
            "version": 6,
            "params": {
                "note": {
                    "deckName": DECK_NAME,
                    "modelName": DECK_FORMAT,
                    "fields": {
                        "Expression": word.word,
                        "Meaning": word.definition,
                        "Sentence": word.sentence,
                        "Audio": audio_path
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
