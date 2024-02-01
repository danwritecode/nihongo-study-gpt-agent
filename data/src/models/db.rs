use sqlx::types::chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct NihongoWordWithTensesStructured {
    pub id: i64,
    pub word: String,
    pub definition: String,
    pub sentence: String,
    pub kanji_mnemonic: Option<String>,
    pub spoken_mnemonic: Option<String>,
    pub is_processed: bool,
    pub word_tenses: Vec<NihongoWordWithTensesStructuredTenses>,
    pub created_at: DateTime<Utc>
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct NihongoWordWithTensesStructuredTenses {
    pub id: i64,
    pub word_id: i64,
    pub word: String,
    pub sentence: String,
    pub tense_type: String,
    pub created_at: DateTime<Utc>
}


#[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct NihongoWordWithTenses {
    pub id: i64,
    pub word: String,
    pub is_kanji: bool,
    pub definition: String,
    pub sentence: String,
    pub kanji_mnemonic: Option<String>,
    pub spoken_mnemonic: Option<String>,
    pub word_reading: String,
    pub sentence_translation: String,
    pub word_id: Option<i64>,
    pub tense_word: Option<String>,
    pub tense_sentence: Option<String>,
    pub tense_type: Option<String>
}

