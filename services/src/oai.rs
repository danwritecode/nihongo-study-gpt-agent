use anyhow::{Result, anyhow, bail};
use data::models::oai::{Prompt, ModelProvider};
use serde::de::DeserializeOwned;
use dotenv::dotenv;

use async_openai::{
    types::{
        CreateChatCompletionRequestArgs, ChatCompletionResponseFormat, ChatCompletionResponseFormatType, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    },
    Client, config::{Config, OpenAIConfig},
};


#[derive(Clone, Debug)]
pub struct ChatAsync {
    messages: Vec<ChatCompletionRequestMessage>,
    model: String,
    client: Client<OpenAIConfig>,
    provider: ModelProvider
}

impl ChatAsync {
    pub fn new(prompt: Prompt) -> Self {
        dotenv().ok();
        
        let messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt.system_prompt)
                .build().expect("Failed to build system request")
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt.user_prompt)
                .build().expect("Failed to build user request")
                .into(),
        ]; 
        
        let client = match prompt.provider {
            ModelProvider::OpenAi => {
                let key = std::env::var("OPENAI_API_KEY").expect("Could not find OPENAI_API_KEY");
                let config = OpenAIConfig::new()
                    .with_api_key(key);

                let client = Client::with_config(config);
                client
            },
            ModelProvider::Mistral => {
                let key = std::env::var("MISTRAL_API_KEY").expect("Could not find MISTRAL_API_KEY");
                let config = OpenAIConfig::new()
                    .with_api_key(key)
                    .with_api_base("https://api.mistral.ai/v1");

                let client = Client::with_config(config);
                client
            },
        };

        ChatAsync {
            messages,
            model: prompt.model.to_string(),
            client,
            provider: prompt.provider
        }
    }
    

    pub async fn chat_json<P>(&mut self) -> Result<P> 
    where 
        P: DeserializeOwned
    {
        let mut retry_count = 0;
        let max_retries = 5;

        while retry_count < max_retries {
            match self.inner_chat_json().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("JSON Chat failure or deserialization error | error: {}. Retrying... ({}/{})", e, retry_count + 1, max_retries);

                    // only want to system modify on first failure and not always
                    if retry_count == 0 {
                        let system_prompt = &self.messages.first()
                            .ok_or_else(|| anyhow!("Failed to find system prompt when generating chat completion in error retry"))?
                            .clone();

                        let new_system_prompt = format!("
                            You are being invoked again as a result of a JSON deserilazation failure in a previous attempt.
                            Please pay careful attention to the JSON format described and adhere perfectly to this.
                            
                            {:?}
                        ", system_prompt);

                        let new_system_prompt = ChatCompletionRequestSystemMessage {
                            content: new_system_prompt,
                            role: async_openai::types::Role::System,
                            name: None
                        };

                        let message = ChatCompletionRequestMessage::System(new_system_prompt);

                        self.messages[0] = message;
                    }
                    retry_count += 1;
                }
            }
        }

        bail!("Failed to generate chat completion, error. Retried: {} times, giving up", retry_count);
    }

    async fn inner_chat_json<P>(&self) -> Result<P> 
    where 
        P: DeserializeOwned
    {
        let client = &self.client;

        let request = match &self.provider {
            ModelProvider::OpenAi => {
                let res_format = ChatCompletionResponseFormat {
                    r#type: ChatCompletionResponseFormatType::JsonObject
                };
                
                CreateChatCompletionRequestArgs::default()
                    .stream(false)
                    .model(&self.model)
                    .response_format(res_format)
                    .messages(self.messages.clone())
                    .build()?
            },
            ModelProvider::Mistral => {
                CreateChatCompletionRequestArgs::default()
                    .stream(false)
                    .model(&self.model)
                    .messages(self.messages.clone())
                    .build()?
            }
        };

        let returned_message = client.chat().create(request).await?
            .choices
            .first()
            .ok_or_else(|| anyhow!("First option missing from OAI prompt return"))?
            .message
            .content
            .clone()
            .ok_or_else(|| anyhow!("Content missing from OAI prompt message"))?;
        
        let returned_message = returned_message.replace("\\", "");

        match serde_json::from_str::<P>(returned_message.as_str()) {
            Ok(des) => Ok(des),
            Err(e) => {
                println!("JSON String: {}", returned_message);
                bail!("Failed to deserialize JSON, error: {}", e)
            }
        }
    }

    pub async fn chat_raw(&mut self) -> Result<String> {
        let mut retry_count = 0;
        let max_retries = 5;

        while retry_count < max_retries {
            match self.inner_chat_raw().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    eprintln!("Raw Chat failure | error: {}. Retrying... ({}/{})", e, retry_count + 1, max_retries);

                    // only want to system modify on first failure and not always
                    if retry_count == 0 {
                        let system_prompt = &self.messages.first()
                            .ok_or_else(|| anyhow!("Failed to find system prompt when generating chat completion in error retry"))?
                            .clone();

                        let new_system_prompt = format!("
                            You are being invoked as a result of a previous inference failure. Please review the system prompt carefully and response accurately.
                            
                            {:?}
                        ", system_prompt);

                        let new_system_prompt = ChatCompletionRequestSystemMessage {
                            content: new_system_prompt,
                            role: async_openai::types::Role::System,
                            name: None
                        };

                        let message = ChatCompletionRequestMessage::System(new_system_prompt);

                        self.messages[0] = message;
                    }
                    retry_count += 1;
                }
            }
        }

        bail!("Failed to generate chat completion, error. Retried: {} times, giving up", retry_count);
    }

    async fn inner_chat_raw(&self) -> Result<String> {
        let client = &self.client;
        let request = CreateChatCompletionRequestArgs::default()
            .stream(false)
            .model(&self.model)
            .temperature(0.2)
            .messages(self.messages.clone())
            .build()?;

        Ok(client.chat().create(request).await?
            .choices
            .first()
            .ok_or_else(|| anyhow!("First option missing from OAI prompt return"))?
            .message
            .content
            .clone()
            .ok_or_else(|| anyhow!("Content missing from OAI prompt message"))?)
    }

}


#[deprecated]
pub async fn chat_json<P>(mut messages: Vec<ChatCompletionRequestMessage>, model: &str) -> Result<P> 
where 
    P: DeserializeOwned
{
    let mut retry_count = 0;
    let max_retries = 5;

    while retry_count < max_retries {
        match inner_chat_json(&messages, model).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                eprintln!("JSON Chat failure or deserialization error | error: {}. Retrying... ({}/{})", e, retry_count + 1, max_retries);

                // only want to system modify on first failure and not always
                if retry_count == 0 {
                    let system_prompt = messages.first()
                        .ok_or_else(|| anyhow!("Failed to find system prompt when generating chat completion in error retry"))?
                        .clone();

                    let new_system_prompt = format!("
                        You are being invoked again as a result of a JSON deserilazation failure in a previous attempt.
                        Please pay careful attention to the JSON format described and adhere perfectly to this.
                        
                        {:?}
                    ", system_prompt);

                    let new_system_prompt = ChatCompletionRequestSystemMessage {
                        content: new_system_prompt,
                        role: async_openai::types::Role::System,
                        name: None
                    };

                    let message = ChatCompletionRequestMessage::System(new_system_prompt);

                    messages[0] = message;
                }
                retry_count += 1;
            }
        }
    }

    bail!("Failed to generate chat completion, error. Retried: {} times, giving up", retry_count);
}

async fn inner_chat_json<P>(messages: &Vec<ChatCompletionRequestMessage>, model: &str) -> Result<P> 
where 
    P: DeserializeOwned
{
    let client = Client::new();
    let res_format = ChatCompletionResponseFormat {
        r#type: ChatCompletionResponseFormatType::JsonObject
    };

    let request = CreateChatCompletionRequestArgs::default()
        .stream(false)
        .model(model)
        // .temperature(0.2)
        .response_format(res_format)
        .messages(messages.clone())
        .build()?;

    let returned_message = client.chat().create(request).await?
        .choices
        .first()
        .ok_or_else(|| anyhow!("First option missing from OAI prompt return"))?
        .message
        .content
        .clone()
        .ok_or_else(|| anyhow!("Content missing from OAI prompt message"))?;

    match serde_json::from_str::<P>(returned_message.as_str()) {
        Ok(des) => Ok(des),
        Err(e) => {
            bail!("Failed to deserialize JSON, error: {}", e)
        }
    }
}


#[deprecated]
pub async fn chat_raw(mut messages: Vec<ChatCompletionRequestMessage>, model: &str) -> Result<String> {
    let mut retry_count = 0;
    let max_retries = 5;

    while retry_count < max_retries {
        match inner_chat_raw(&messages, model).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                eprintln!("Raw Chat failure | error: {}. Retrying... ({}/{})", e, retry_count + 1, max_retries);

                // only want to system modify on first failure and not always
                if retry_count == 0 {
                    let system_prompt = messages.first()
                        .ok_or_else(|| anyhow!("Failed to find system prompt when generating chat completion in error retry"))?
                        .clone();

                    let new_system_prompt = format!("
                        You are being invoked as a result of a previous inference failure. Please review the system prompt carefully and response accurately.
                        
                        {:?}
                    ", system_prompt);

                    let new_system_prompt = ChatCompletionRequestSystemMessage {
                        content: new_system_prompt,
                        role: async_openai::types::Role::System,
                        name: None
                    };

                    let message = ChatCompletionRequestMessage::System(new_system_prompt);

                    messages[0] = message;
                }
                retry_count += 1;
            }
        }
    }

    bail!("Failed to generate chat completion, error. Retried: {} times, giving up", retry_count);
}

async fn inner_chat_raw(messages: &Vec<ChatCompletionRequestMessage>, model: &str) -> Result<String> {
    let client = Client::new();
    let request = CreateChatCompletionRequestArgs::default()
        .stream(false)
        .model(model)
        .temperature(0.2)
        .messages(messages.clone())
        .build()?;

    Ok(client.chat().create(request).await?
        .choices
        .first()
        .ok_or_else(|| anyhow!("First option missing from OAI prompt return"))?
        .message
        .content
        .clone()
        .ok_or_else(|| anyhow!("Content missing from OAI prompt message"))?)
}
