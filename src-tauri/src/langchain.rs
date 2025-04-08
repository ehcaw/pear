use langchain_rust::{
    chain::{builder::ConversationalChainBuilder, Chain, LLMChainBuilder},
    fmt_message, fmt_placeholder, fmt_template,
    language_models::{llm::LLM, options::CallOptions},
    llm::{OpenAI, OpenAIConfig},
    memory::SimpleMemory,
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::messages::Message,
    template_fstring,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::env;
use std::sync::OnceLock;

static GROQ_CONFIG: OnceLock<OpenAIConfig> = OnceLock::new();
pub fn get_groq_config() -> &'static OpenAIConfig {
    GROQ_CONFIG.get_or_init(|| {
        OpenAIConfig::new()
            .with_api_key(env::var("GROQ_API_KEY").unwrap())
            .with_api_base("https://api.groq.com/openai/v1".to_string())
    })
}

static GROQ_CLIENT: OnceLock<OpenAI<OpenAIConfig>> = OnceLock::new();
pub fn get_groq_client() -> &'static OpenAI<OpenAIConfig> {
    GROQ_CLIENT.get_or_init(|| {
        let groq_config = get_groq_config();
        OpenAI::new(groq_config.clone()).with_model("llama-3.3-70b-versatile".to_string())
    })
}

#[tauri::command]
pub async fn chain_test() -> String {
    let llm = get_groq_client();

    // let resp = llm.invoke("What is rust").await.unwrap();
    // resp
    let memory = SimpleMemory::new();
    let chain = ConversationalChainBuilder::new()
        .llm(llm)
        .memory(memory.into())
        .build()
        .expect("Error building chain");
}
