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
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;

use crate::supabase::query_document;

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
static GROQ_SIMPLE_CLIENT: OnceLock<OpenAI<OpenAIConfig>> = OnceLock::new();
pub fn get_simple_groq_client() -> &'static OpenAI<OpenAIConfig> {
    GROQ_SIMPLE_CLIENT.get_or_init(|| {
        let groq_config = get_groq_config();
        OpenAI::new(groq_config.clone()).with_model("llama-3.1-8b-instant".to_string())
    })
}

#[tauri::command]
pub async fn chain_test() {
    let llm = get_groq_client().clone();
    // let resp = llm.invoke("What is rust").await.unwrap();
    // resp
    let memory = SimpleMemory::new();
    let chain = ConversationalChainBuilder::new()
        .llm(llm)
                .prompt(message_formatter![
                    fmt_message!(Message::new_system_message("You are a helpful assistant")),
                    fmt_template!(HumanMessagePromptTemplate::new(
                    template_fstring!("
        The following is a friendly conversation between a human and an AI. The AI is talkative and provides lots of specific details from its context. If the AI does not know the answer to a question, it truthfully says it does not know.

        Current conversation:
        {history}
        Human: {input}
        AI:
        ",
        "input","history")))
        ])
        .memory(memory.into())
        .build()
        .expect("Error building ConversationalChain");
}

pub async fn determine_intricacy(
    query: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let llm = get_simple_groq_client().clone();

    // Create a system message explaining the task
    let system_message = Message::new_system_message(
        "You are an assistant that analyzes programming questions and determines how much documentation is needed."
    );

    // Create a human message with the query
    let human_message = Message::new_human_message(format!(
        "Based on this programming query: '{}', respond with EXACTLY ONE of these words: none, light, medium, or heavy. \
        This indicates how much documentation is needed to answer the question effectively.",
        query
    ));

    // Call the LLM directly
    let result = llm.generate(&[system_message, human_message]).await?;

    // Extract just the word from the response
    let response = result.generation.trim().to_lowercase();

    // Ensure we get one of the expected values
    match response.as_str() {
        "none" | "light" | "medium" | "heavy" => Ok(response),
        _ => {
            // If the response isn't one of the expected values, default to "medium"
            eprintln!(
                "Unexpected intricacy level: {}, defaulting to medium",
                response
            );
            Ok("medium".to_string())
        }
    }
}

// Function to retrieve documents
pub async fn retrieve_documents(
    query: &str,
    intricacy_level: &str,
    directory_string: &str,
) -> String {
    // This would connect to your vector DB - simplified here
    let num_documents: u8 = match intricacy_level {
        "none" => 0,
        "light" => 3,
        "medium" => 5,
        "heavy" => 10,
        _ => 0,
    };

    // Check if we need to retrieve any documents
    if num_documents == 0 {
        return "".to_string(); // Return empty string if no docs needed
    }

    // Get documents from Supabase
    let documents_value = match query_document(
        query.to_string(),
        directory_string.to_string(),
        num_documents,
    )
    .await
    {
        Ok(docs) => docs,
        Err(e) => {
            eprintln!("Error retrieving documents: {:?}", e);
            return format!("Error retrieving documents: {}", e);
        }
    };

    // Format documents into a single string context
    if !documents_value.is_array() || documents_value.as_array().unwrap().is_empty() {
        return "No relevant documents found.".to_string();
    }

    // Process the JSON array into a formatted string
    let documents = documents_value.as_array().unwrap();
    let formatted_docs = documents
        .iter()
        .map(|doc| {
            let name = doc
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let content = doc.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let similarity = doc
                .get("similarity")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            format!(
                "=== Document: {} (Relevance: {:.2}) ===\n{}\n",
                name, similarity, content
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    formatted_docs
}

// Function to generate the final response
pub async fn generate_response(
    query: &str,
    intricacy_level: &str,
    context: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let llm = get_groq_client().clone();

    // Create a system message
    let system_message = Message::new_system_message(
        "You are a helpful AI pair programmer. Provide clear, concise code and explanations.",
    );

    // Create a human message with all the context
    let human_message = Message::new_human_message(format!(
        "Intricacy level: {}\n\nContext: {}\n\nUser query: {}\n\nRespond as a helpful pair programmer.",
        intricacy_level, context, query
    ));

    // Call the LLM
    let result = llm.generate(&[system_message, human_message]).await?;
    Ok(result.generation)
}

#[tauri::command]
pub async fn ai_pair_programmer(query: &str, current_directory: &str) -> Result<String, String> {
    // Step 1: Determine intricacy
    let intricacy_level = determine_intricacy(&query)
        .await
        .map_err(|e| format!("Error determining intricacy: {}", e))?;

    // Step 2: Retrieve documents
    let context = if current_directory.is_empty() {
        String::new()
    } else {
        retrieve_documents(&query, &intricacy_level, &current_directory).await
    };

    // Step 3: Generate final response
    let response = generate_response(&query, &intricacy_level, &context)
        .await
        .map_err(|e| format!("Error generating response: {}", e))?;

    Ok(response)
}
