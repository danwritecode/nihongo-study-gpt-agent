use dotenv::dotenv;
use sqlx::{postgres::PgConnection, Connection};
use anyhow::{Result, bail};
use crate::models::{
    db::NihongoWordWithTenses,
    requests::{NihongoWordReqWord, NihongoWordReqTense}
};

pub async fn add_word(word: &NihongoWordReqWord) -> Result<Option<i64>> {
    // Fine because we aren't hitting this a lot, don't need a pool, we're just hacking
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let mut connection = PgConnection::connect(db_url.as_str()).await?;

    println!("Word from db crate: {:?}", word);

    // wildly inefficient but I don't care, we're just hacking
    let rec = sqlx::query!(
            r#"
                INSERT INTO nihongo_word ( word, definition, sentence, kanji_mnemonic, spoken_mnemonic )
                VALUES ( $1, $2, $3, $4, $5 )
                RETURNING id
            "#,
            word.word,
            word.definition,
            word.sentence,
            word.kanji_mnemonic,
            word.spoken_mnemonic
        )
        .fetch_one(&mut connection)
        .await;

    match rec {
        Ok(r) => Ok(Some(r.id)),
        Err(e) => {
            if let Some(dbe) = e.as_database_error() {
                if let Some(code) = dbe.code() {
                    if code == "23505" {
                        return Ok(None);
                    }
                }
            }
            bail!(e)
        }
    }
}

pub async fn add_word_tense(id: i64, words: Vec<NihongoWordReqTense>) -> Result<()> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let mut connection = PgConnection::connect(db_url.as_str()).await?;

    // wildly inefficient but I don't care, we're just hacking
    for w in &words {
        sqlx::query!(
                r#"
                    INSERT INTO nihongo_word_tense ( word_id, word, sentence, tense_type )
                    VALUES ( $1, $2, $3, $4 )
                "#,
                id,
                w.word,
                w.sentence,
                w.tense_type
            )
            .execute(&mut connection)
            .await?;
    }

    Ok(())
}

pub async fn get_unprocessed_words() -> Result<Vec<NihongoWordWithTenses>> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let mut connection = PgConnection::connect(db_url.as_str()).await?;

    let res = sqlx::query_as!(
        NihongoWordWithTenses,
        r"
            SELECT
                nw.id,
                nw.word,
                nw.definition,
                nw.sentence,
                nw.kanji_mnemonic,
                nw.spoken_mnemonic
            FROM nihongo_word AS nw
            LEFT JOIN nihongo_word_tense AS nwt ON nw.id = nwt.word_id
            WHERE nw.is_processed = false
            ORDER BY id desc
        "
    )
    .fetch_all(&mut connection)
    .await?;

    Ok(res)
}
