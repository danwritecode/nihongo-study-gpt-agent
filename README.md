# Nihongo Study Word Persisting Agent

The objective of this repo is to enable the persisting of words in a sentence to be used in Anki for later study. It's especially convenient for when I'm on my phone and reading material that I can't immediately save into Anki.

My other major objective was the save the various tenses of any word along with an example sentence. That way when reviewing the base form of a word I can see it's various conjugations in context.

#### Format of words stored in db
![Word Table](https://raw.githubusercontent.com/danwritecode/nihongo-study-gpt-agent/master/api/screenshots/2024-01-26_22-16.png)
![Word Tense table](https://raw.githubusercontent.com/danwritecode/nihongo-study-gpt-agent/master/api/screenshots/2024-01-26_22-17.png)

# Setting this up yourself

### Local env
See: https://www.cargo-lambda.info/guide/getting-started.html for details on using cargo lambda

1. Install: openssl, libssl-dev, and pkg-config (can be install on ubuntu with apt-get install)
2. Install cargo lambda (referenced above)
3. Configure your AWS credentials in `~.aws/credentials`
```
[default]
aws_access_key_id = 
aws_secret_access_key = 
region = us-east-2

```
4. Build with 
```
cargo lambda build --release
```
4. Deploy
```
cargo lambda deploy
```

Env Vars: 
```
DATABASE_URL="postgresql://{user}:{password}@{url}:5432/{db}"
RUST_LOG=info
```

### AWS Config
1. You need to create an API Gateway Endpoint that you integrate with your lambda function
2. Upload your environment variables to the Lambda function manually


### Configuring your GPT

1. Openapi spec
```json
{
  "openapi": "3.1.0",
  "info": {
    "title": "Nihongo Sentence Persisting",
    "description": "Takes a word and saves it with mnemonics, definition, sentences, and tenses",
    "version": "v1.0.0"
  },
  "servers": [
    {
      "url": "https://ws1mclo42d.execute-api.us-east-2.amazonaws.com"
    }
  ],
  "paths": {
    "/api/words": {
      "post": {
        "description": "Takes a word and saves it with mnemonics, definition, sentences, and tenses",
        "operationId": "ProcessNihongoSentenceWords",
        "requestBody": {
          "description": "Nihongo word extraction request",
          "required": true,
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/NihongoWordsReq"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Sentence words processed successfully"
          },
          "400": {
            "description": "Invalid request"
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "NihongoWordsReq": {
        "type": "object",
        "properties": {
          "words": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/WordItem"
            }
          }
        }
      },
      "WordItem": {
        "type": "object",
        "properties": {
          "word": {
            "type": "string"
          },
          "definition": {
            "type": "string"
          },
          "sentence": {
            "type": "string"
          },
          "kanji_mnemonic": {
            "type": "string"
          },
          "spoken_mnemonic": {
            "type": "string"
          },
          "word_tenses": {
            "type": "array",
            "items": {
              "$ref": "#/components/schemas/WordTenseItem"
            }
          }
        }
      },
      "WordTenseItem": {
        "type": "object",
        "properties": {
          "word": {
            "type": "string"
          },
          "sentence": {
            "type": "string"
          },
          "tense_type": {
            "type": "string"
          }
        }
      }
    }
  }
}
```

2. GPT Description
```
You are a tool to help users learn Japanese. You have a couple of modes: 

1. Learning Mode (your default mode): While in this mode you should stay in english and be brief with explanations.
2. Conversational Mode: In this mode you should be in only japanese unless told otherwise and be brief as this is a casual conversation.

Learning Mode Details:
In learning mode a user may invoke "Explanation Mode", by typing "explanation mode". When in this mode you are to:
1. Break down the sentences vocab and grammar structures
2. List each vocab word (excluding particles) in a numbered list

The user can then choose to save vocab words in a custom action invocation by writing something like: "save words 1,3,4" (listing the numbers of the vocab words you listed, include word tenses when applicable)

You are to then create the below object that you will post as part of your invoked action. 
For the word: please use it's base form, no conjugated forms
For word_tenses: Please include how the word can be used in present, past and future tenses (when applicable)
For Kanji Mnemonic: Only populate this if there is kanji, be clever and come up with things the kanji looks like to help remember it's meaning.
For Spoken Mnemonic: Do your best to come up with a clever way to remember the spoken words. It can loosely match the pronunciation, it's to guide the user.
```


### SQL Config
I used supabase because it's easy and free for hobby projects, here are the create table scripts that you need

```sql
create table
  public.nihongo_word (
    id bigint generated by default as identity,
    word text not null,
    definition text not null,
    sentence text not null,
    is_processed boolean not null default false,
    kanji_mnemonic text null default ''::text,
    spoken_mnemonic text null,
    created_at timestamp with time zone not null default now(),
    constraint nihongo_word_pkey primary key (id)
  ) tablespace pg_default;

create table
  public.nihongo_word_tense (
    id bigint generated by default as identity,
    word_id bigint not null,
    word text not null,
    sentence text not null,
    tense_type text not null,
    created_at timestamp with time zone not null default now(),
    constraint nihongo_word_tense_pkey primary key (id),
    constraint nihongo_word_tense_word_id_fkey foreign key (word_id) references nihongo_word (id) on update cascade on delete cascade
  ) tablespace pg_default;
```

# Next Steps
I need to create a service that will run to get words from the DB and add them into Anki. Will be working on that soon.
