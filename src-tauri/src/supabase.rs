use reqwest; // Add reqwest
use serde_json::{json, Value};
use std::env;
use std::sync::OnceLock;
// Removed unused imports: Cursor, Read, Write, str, SystemTime, UNIX_EPOCH

use postgrest::Postgrest;

struct Document {
    id: String,
    name: String,
    content: String,
    similarity: Option<f64>,
}

// --- Correct Postgrest Client Initialization ---
static SUPABASE_CLIENT: OnceLock<Postgrest> = OnceLock::new();

fn get_client() -> &'static Postgrest {
    SUPABASE_CLIENT.get_or_init(|| {
        // 1. Get BASE URL (e.g., https://<ref>.supabase.co)
        let supabase_url = "http://127.0.0.1:54321/rest/v1";
        println!("Initializing Postgrest client for URL: {}", supabase_url);

        // 2. Get ANON KEY
        let supabase_admin_key =
            env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY env var not set");

        // 3. Initialize Postgrest client with BASE URL
        //    The library handles appending /rest/v1/
        println!("Initializing Postgrest client for URL: {}", supabase_url);
        Postgrest::new(supabase_url) // DO NOT append /rest/v1 here
            .insert_header("apikey", supabase_admin_key.clone()) // Clone key if needed elsewhere
            .insert_header("Authorization", format!("Bearer {}", supabase_admin_key))
    })
}

// --- REMOVE the old synchronous call_embedding_function ---
// pub fn call_embedding_function(content_to_embed: &str) -> Result<String, String> { ... }

// --- NEW Async Edge Function Call using Reqwest ---
// Helper to potentially reuse the client (optional)
// static REQWEST_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
// fn get_reqwest_client() -> &'static reqwest::Client {
//     REQWEST_CLIENT.get_or_init(|| reqwest::Client::new())
// }

// Renamed to indicate it's async
pub async fn call_embedding_function_async(content_to_embed: &str) -> Result<String, String> {
    let supabase_anon_key =
        env::var("SUPABASE_ANON_KEY").map_err(|e| format!("SUPABASE_ANON_KEY not found: {}", e))?;

    // Construct the FULL function URL (e.g., http://localhost:54321/functions/v1/embed)
    // Use SUPABASE_URL if functions run relative to it, or use a specific local URL.
    // For local testing, often it's directly localhost. Check your Supabase CLI setup.
    let function_url = "http://localhost:54321/functions/v1/embed"; // Hardcoded for local dev based on prior examples
                                                                    // let base_url = env::var("SUPABASE_URL").map_err(|e| format!("SUPABASE_URL not found: {}", e))?;
                                                                    // let function_url = format!("{}/functions/v1/embed", base_url.trim_end_matches('/'));

    println!("Calling async edge function URL: {}", function_url);

    let payload = json!({ "input": content_to_embed });

    // Create a client for this request (or reuse one via OnceLock)
    let client = reqwest::Client::new();
    // let client = get_reqwest_client(); // If using OnceLock

    let response = client
        .post(function_url) // Use the function URL
        .header("Authorization", format!("Bearer {}", supabase_anon_key))
        .header("Content-Type", "application/json")
        .json(&payload) // Handles serialization and Content-Type (though explicit header is fine)
        .send()
        .await // Async send
        .map_err(|e| format!("Reqwest request failed: {}", e))?;

    let status = response.status();
    let response_text = response
        .text()
        .await // Async read body
        .map_err(|e| format!("Failed to read edge function response body: {}", e))?;

    if status.is_success() {
        // Check for empty response - this might be the source of the "" error
        if response_text.is_empty() {
            // Log or return a specific error if an empty body is unexpected
            eprintln!("Warning: Edge function returned successful status ({}) but empty body for input: {:.50}...", status, content_to_embed);
            // Decide how to handle: return error or empty string? Returning error is safer.
            return Err(format!(
                "Edge function returned success ({}) but empty body",
                status
            ));
        }
        Ok(response_text)
    } else {
        Err(format!(
            "Edge function HTTP Error {}: {}",
            status,
            response_text // Include error body from function
        ))
    }
}

// --- insert_embedding function remains the same ---
pub async fn insert_embedding(
    file_path: String,
    content: String,
    embedding_json_string: String,
) -> Result<(), String> {
    let static_client_ref = get_client();
    let client = static_client_ref.clone();

    // Basic check: Ensure embedding string looks like a vector
    if !embedding_json_string.trim().starts_with('[')
        || !embedding_json_string.trim().ends_with(']')
    {
        return Err(format!("Invalid embedding format received before DB insert for {}: does not look like a JSON array string", file_path));
    }
    // Add more validation if needed (e.g., parse and check length)

    let record_data = json!({
        "id": file_path,
        "name": file_path.split('/').last().unwrap_or(&file_path).to_string(),
        "content": content,
        "embedding": embedding_json_string // Insert the raw JSON array string
    });

    let record_data_string = serde_json::to_string(&record_data)
        .map_err(|e| format!("Failed to serialize record data: {}", e))?;

    println!("Executing insert for ID: {}", file_path);

    let response = client
        .from("files")
        .upsert(format!("[{}]", record_data_string)) // Wrap object in array string
        .execute()
        .await
        .map_err(|e| format!("Postgrest insert execute error: {}", e))?;

    let status = response.status();
    println!("Insert response status for ID {}: {}", file_path, status);

    if status.is_success() {
        Ok(())
    } else {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "<Failed to read error body>".to_string());
        Err(format!(
            "Postgrest insert failed for ID {} with status {}: {}",
            file_path, status, error_body
        ))
    }
}

#[tauri::command]
pub async fn query_document(query: String, directory: String, num: u8) -> Result<Value, String> {
    let client = get_client();
    match call_embedding_function_async(&query).await {
        Ok(response) => {
            // Parse the response string into a serde_json::Value
            let response_json: Value = serde_json::from_str(&response)
                .map_err(|e| format!("Failed to parse embedding response as JSON: {}", e))?;

            // Extract the embedding array
            let embedding = response_json
                .get("embedding")
                .ok_or_else(|| "Missing 'embedding' field in response".to_string())?;

            // Convert the embedding to a string for the RPC call
            let embedding_str = serde_json::to_string(embedding)
                .map_err(|e| format!("Failed to serialize embedding: {}", e))?;

            let result = client
                .rpc(
                    "query_files",
                    format!(
                        "{{\"file_directory\": \"{}\", \"query_embedding\": {}, \"num\": {}}}",
                        directory, embedding_str, num
                    ),
                )
                .execute()
                .await
                .map_err(|e| format!("Database query failed: {}", e))?;

            // Get the response body as text
            let body = result
                .text()
                .await
                .map_err(|e| format!("Failed to get response text: {}", e))?;

            // Parse the text into a Vec<Value>
            let results: Value = serde_json::from_str(&body)
                .map_err(|e| format!("Failed to parse query results as JSON array: {}", e))?;

            Ok(results)
        }
        Err(e) => Err(format!("Embedding function call failed: {}", e)),
    }
}
