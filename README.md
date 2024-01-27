# Nihongo Study Word Persisting Agent

The objective of this repo is to enable the persisting of words in a sentence to be used in Anki for later study. It's especially convenient for when I'm on my phone and reading material that I can't immediately save into Anki.

My other major objective was the save the various tenses of any word along with an example sentence. That way when reviewing the base form of a word I can see it's various conjugations in context.

#### Format of words stored in db
![Word Table](https://raw.githubusercontent.com/danwritecode/nihongo-study-gpt-agent/master/screenshots/2024-01-26_22-16.png)
![Word Tense table](https://raw.githubusercontent.com/danwritecode/nihongo-study-gpt-agent/master/screenshots/2024-01-26_22-17.png)

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
    "description": "Extracts all vocab words from a sentence excluding particles and saves them with their relevant tenses (past, future, current)",
    "version": "v1.0.0"
  },
  "servers": [
    {
      "url": "API GATEWAY BASE URL HERE"
    }
  ],
  "paths": {
    "/{endpoint path here}": {
      "post": {
        "description": "Extracts all vocab words from a sentence excluding particles and saves them with their relevant tenses (past, future, current)",
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
          "type": {
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
You are a conversational practice tool for users learning japanese. You are to keep your speech relatively simple and your objective is to simply hold a conversation. 

There is 1 phrase that can be said by the user that when heard, you will speak in English to explain the previous statement, other than that you will stay in Japanese only...this phrase is "Nihongo Explain".

Please note because you are conversational, you should not give long winded responses. Be short and concise, you are having a conversation, not explaining things endlessly.

You are going to post a body like this with your invoked action: 
{
  "words": [
    {
      "word": "",
      "definition": "",
      "sentence": "",
      "word_tenses": [
        {
          "word": "",
          "sentence": "",
          "tense_type": ""
        },
        {
          "word": "",
          "sentence": "",
          "tense_type": ""
        }
      ]
    }
  ]
}

What you should do is take each word, create it's definition, create a sentence for it, then take it's possible tenses and create sentences for those as well.
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
    created_at timestamp with time zone not null default now(),
    constraint nihongo_word_pkey primary key (id)
  ) tablespace pg_default;

create table
  public.nihongo_word_tense (
    id bigint generated by default as identity,
    word_id bigint not null,
    word text not null,
    sentence text not null,
    created_at timestamp with time zone not null default now(),
    tense_type text not null,
    constraint nihongo_word_tense_pkey primary key (id),
    constraint nihongo_word_tense_word_id_fkey foreign key (word_id) references nihongo_word (id) on update cascade on delete cascade
  ) tablespace pg_default;
```

# Next Steps
I need to create a service that will run to get words from the DB and add them into Anki. Will be working on that soon.
