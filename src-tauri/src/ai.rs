use directories::ProjectDirs;
use hound::{SampleFormat, WavSpec, WavWriter};
use serde_json::{json, Value};
use std::cmp::{max, min};
use std::env;
use std::f32::consts::PI;
use std::fs;
use std::io::Cursor;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use tempfile::Builder; // Added missing import

use curl::easy::Easy;
use futures::Future;
use tokio_core::reactor::Core;
use tokio_curl::Session;

use tauri::async_runtime;
use tauri::{command, AppHandle, Runtime, Window};

use groq_api_rust::{
    ChatCompletionMessage, ChatCompletionRequest, ChatCompletionRoles, GroqClient,
    SpeechToTextRequest, TextToSpeechRequest,
};

fn get_app_data_dir() -> Result<PathBuf, String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ehcaw", "pear") {
        // Get the data direc
        // tory path
        let data_dir = proj_dirs.data_dir();

        fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        Ok(data_dir.to_path_buf())
    } else {
        Err("Could not determine app data directory".to_string())
    }
}

// fn create_temporary_audio_file(audio: Vec<i16>) -> Result<PathBuf, Box<dyn std::error::Error>> {
//     let app_directory = get_app_data_dir();
//     let temp_file = Builder::new().suffix(".wav").tempfile_in(app_directory);
//     let temp_path = temp_file.path().to_path_buf();
//     let spec = WavSpec {
//         channels: 1,
//         sample_rate: 32000,
//         bits_per_sample: 16,
//         sample_format: SampleFormat::Int,
//     };

//     let mut writer = WavWriter::create(temp_path.clone(), spec);

//     for &sample in audio {
//         writer.write_sample(sample)?;
//     }
//     writer.finalize()?;
//     Ok(temp_path);
// }

//const api_key = std::env::var("GROQ_API_KEY");
//const client = GroqClient::new(api_key);
static GROQ_CLIENT: OnceLock<GroqClient> = OnceLock::new();
fn get_client() -> &'static GroqClient {
    GROQ_CLIENT.get_or_init(|| {
        let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY env variable not set");
        GroqClient::new(api_key.to_string(), None)
    })
}

fn process_audio(audio: Vec<f32>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Create a WAV file in memory
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000, // Use 16kHz which is standard for speech recognition
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    // Create an in-memory buffer for the WAV file
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = hound::WavWriter::new(cursor, spec)?;

        // Convert float samples [-1.0, 1.0] to i16 [-32768, 32767]
        for &sample in &audio {
            let sample_i16 = (sample * 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }

        writer.finalize()?;
    }

    Ok(buffer)
}

fn transcribe(audio: Vec<f32>) -> Result<String, String> {
    // No need for a separate runtime or block_on

    println!("Processing audio..."); // Log progress
                                     // Process the audio to create WAV data in memory
    let wav_data = process_audio(audio).map_err(|e| format!("Failed to process audio: {}", e))?;
    println!("Audio processed, WAV size: {} bytes", wav_data.len()); // Log size

    // Create the speech-to-text request
    let request = SpeechToTextRequest::new(wav_data)
        .temperature(0.7) // Optional: configure as needed
        .language("en") // Optional: configure as needed
        .model("whisper-large-v3"); // Ensure this model is supported by Groq STT

    println!("Getting Groq client..."); // Log progress
                                        // Get the shared Groq client instance
    let client = get_client();

    println!("Sending request to Groq API..."); // Log progress
                                                // Execute the SYNCHRONOUS API request
                                                // This will block the current Tauri command thread, which is acceptable.
    let result = client.speech_to_text(request);

    // Handle the Result
    match result {
        Ok(response) => {
            println!("Groq API Success. Transcription: {}", response.text); // Log success and result
            Ok(response.text)
        }
        Err(e) => {
            eprintln!("Groq API Error: {:?}", e); // Log the full error
                                                  // Try to provide a more specific error message if possible
                                                  // For example, if e has a method to get status code or message:
                                                  // Err(format!("Failed to get response from Groq: {} - {}", e.status(), e.message()))
            Err(format!("Failed to get response from Groq: {}", e))
        }
    }
}

fn get_llm_response(transcription: &String) -> Result<String, String> {
    let messages = vec![ChatCompletionMessage {
        role: ChatCompletionRoles::User,
        content: transcription.to_string(),
        name: None,
    }];
    let request = ChatCompletionRequest::new("meta-llama/llama-4-scout-17b-16e-instruct", messages);
    let client = get_client();
    let response = client.chat_completion(request);
    match response {
        Ok(response) => {
            // Safely get the first choice
            if let Some(choice) = response.choices.get(0) {
                // Return the content, cloning it to return an owned String
                Ok(choice.message.content.clone())
            } else {
                // Handle the unlikely case where there are no choices in a success response
                Err("LLM response contained no choices.".to_string())
            }
        }
        Err(e) => {
            // Handle the error from the API call properly
            Err(format!("Error getting llm response: {}", e))
        }
    }
}
fn text_to_speech(text: &String) -> Result<String, String> {
    // Get client
    let client = get_client();

    // Create request
    let request = TextToSpeechRequest::new(
        Some("playai-tts".to_string()),
        &text,
        Some("Chip-PlayAI".to_string()),
        Some(1.0),
    );

    // Send request to Groq API
    let response = client.text_to_speech(request);

    match response {
        Ok(response) => {
            // Get app data directory
            let app_data_dir = get_app_data_dir()
                .map_err(|e| format!("Failed to get app data directory: {}", e))?;

            // Create a unique filename based on timestamp
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("Time error: {}", e))?
                .as_millis();

            // Create audio file path with .mp3 extension (adjust if Groq uses a different format)
            let file_path = app_data_dir.join(format!("tts_{}.mp3", timestamp));

            // Write the audio data to the file
            fs::write(&file_path, response.audio_data)
                .map_err(|e| format!("Failed to write audio file: {}", e))?;

            // Return the path as a string
            Ok(file_path.to_string_lossy().to_string())
        }
        Err(e) => {
            // Handle the error from the API call properly
            Err(format!("Error getting text-to-speech response: {}", e))
        }
    }
}

#[tauri::command]
pub fn transcribe_generate_play(audio: Vec<f32>) -> Result<String, String> {
    let transcription = transcribe(audio).unwrap();
    let llm_response = get_llm_response(&transcription).unwrap();
    let tts_audio_path = text_to_speech(&llm_response).unwrap();
    let result = json!({
        "audio_path": tts_audio_path,
        "transcription": transcription,
        "llm_response": llm_response
    });
    Ok(result.to_string())
}

#[tauri::command]
pub fn delete_audio_file(path: String) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| format!("Failed to delete audio file: {}", e))
}
