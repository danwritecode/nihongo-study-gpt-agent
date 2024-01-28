use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordReq {
    pub words: Vec<NihongoWordReqWord>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordReqWord {
    pub word: String,
    pub definition: String,
    pub sentence: String,
    pub kanji_mnemonic: Option<String>,
    pub spoken_mnemonic: Option<String>,
    pub is_processed: bool,
    pub word_tenses: Vec<NihongoWordReqTense>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordReqTense {
    pub word: String,
    pub sentence: String,
    pub tense_type: String
}
