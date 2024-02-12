#[derive(Clone, Debug)]
pub struct Prompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub model: String,
    pub provider: ModelProvider 
}

#[derive(Clone, Debug)]
pub enum ModelProvider {
    OpenAi,
    Mistral
}
