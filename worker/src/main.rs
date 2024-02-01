use anyhow::{Result, bail};
use serde_json::Value;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use dotenv::dotenv;
use rand::Rng;

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;


use data::{
    models::db::NihongoWordWithTenses,
    services::data::{get_unprocessed_words, update_word_status}
};

const VOICE_ID: &str = "IKne3meq5aSn9XLyUdCD";
const DECK_NAME: &str = "Dan's Nihongo Deck";
const DECK_FORMAT: &str = "JP1Kv3";
const BASE_ANKI_MEDIA_DIR: &str = "/home/dan/.local/share/Anki2/User 1/collection.media";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    dotenv().ok();
    let eleven_labs_key = std::env::var("ELEVEN_LABS_KEY")?;
    
    loop {
        let up_words = get_unprocessed_words().await?;
        let words = group_rows(up_words);
        
        println!("words: {:?}", words);

        for w in &words {
            generate_and_save_audio_files(&w.word, &w.word_reading, &w.sentence, &eleven_labs_key).await?;
            add_card_anki(w).await?;

            // finally update the word status to processed = true
            update_word_status(w.id).await?;
        }

        sleep(Duration::from_secs(120)).await;
    }
}

fn group_rows(words: Vec<NihongoWordWithTenses>) -> Vec<NihongoWordsGrouped> {
    let mut word_map: HashMap<i64, NihongoWordsGrouped> = HashMap::new();

    for w in &words {
        word_map.entry(w.id)
            .or_insert(NihongoWordsGrouped { 
                id: w.id, 
                word: w.word.clone(), 
                is_kanji: w.is_kanji.clone(), 
                definition: w.definition.clone(), 
                sentence: w.sentence.clone(), 
                kanji_mnemonic: w.kanji_mnemonic.clone(),
                spoken_mnemonic: w.spoken_mnemonic.clone(), 
                word_reading: w.word_reading.clone(), 
                sentence_translation: w.sentence_translation.clone(), 
                tenses: vec![] 
            })
            .tenses.push(NihongoWordTense { tense_word: w.tense_word.clone(), tense_sentence: w.tense_sentence.clone(), tense_type: w.tense_type.clone() })
    }

    let words = word_map.values().cloned().collect::<Vec<NihongoWordsGrouped>>();

    words
}

async fn generate_and_save_audio_files(
    word: &str, 
    word_reading: &str, 
    sentence: &str,
    eleven_labs_key: &str
) -> Result<()> {
    let sentence_audio = generate_audio(eleven_labs_key, sentence).await?;
    save_file(word, word_reading, sentence_audio, "sentence")?;

    let word_audio = generate_audio(eleven_labs_key, word_reading).await?;
    save_file(word, word_reading, word_audio, "word")?;

    Ok(())
}

/// file_type is really just 'word' or 'sentence' but I didn't feel like enum'ing it
fn save_file(word: &str, word_reading: &str, bytes: Vec<u8>, file_type: &str) -> Result<()> {
    let path = format!("{}/lang_crack_audio_{}_{}_{}.mp3", BASE_ANKI_MEDIA_DIR, file_type, word, word_reading);
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
    word: &NihongoWordsGrouped
) -> Result<()> {
    let anki_word_ref = format!("[sound:lang_crack_audio_word_{}_{}.mp3]", word.word, word.word_reading);
    let anki_sentence_ref = format!("[sound:lang_crack_audio_sentence_{}_{}.mp3]", word.word, word.word_reading);
    let mut rng = rand::thread_rng();
    let range = rng.gen_range(1001..20000);
    let is_kanji = match word.is_kanji {
        true => "True",
        false => "False"
    };

    let mut tenses = "".to_string();
    
    if word.tenses.len() > 0 {
        for t in &word.tenses {
            tenses.push_str(&format!("
                {:?}: {:?} | {:?} \n
            ", 
                t.tense_type.clone().map_or("".to_string(), |v| v), 
                t.tense_word.clone().map_or("".to_string(), |v| v), 
                t.tense_sentence.clone().map_or("".to_string(), |v| v)
            ));
        }
    }

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
                        "Index": format!("{}", range),
                        "Word": word.word,
                        "Word With Reading": word.word_reading,
                        "Definition": word.definition,
                        "Example Sentence": word.sentence,
                        "Sentence Translation": word.sentence_translation,
                        "word_audio": anki_word_ref,
                        "sentence_audio": anki_sentence_ref,
                        "Kanji": is_kanji,
                        "kanji_mnemonic": word.kanji_mnemonic,
                        "spoken_mnemonic": word.spoken_mnemonic,
                        "tenses": tenses
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

    if !res["error"].is_null() {
        bail!("Response from anki contained error(s) | error(s): {}", res["error"]) 
    }

    Ok(())
}


#[derive(Debug, Clone, PartialEq)]
struct NihongoWordsGrouped {
    pub id: i64,
    pub word: String,
    pub is_kanji: bool,
    pub definition: String,
    pub sentence: String,
    pub kanji_mnemonic: Option<String>,
    pub spoken_mnemonic: Option<String>,
    pub word_reading: String,
    pub sentence_translation: String,
    pub tenses: Vec<NihongoWordTense>
}

#[derive(Debug, Clone, PartialEq)]
struct NihongoWordTense {
    pub tense_word: Option<String>,
    pub tense_sentence: Option<String>,
    pub tense_type: Option<String>
}
