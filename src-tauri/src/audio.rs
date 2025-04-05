use lazy_static::lazy_static;
use rodio::{Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use tauri::command;

// Store active audio players to allow control (stop, pause)
lazy_static! {
    static ref AUDIO_PLAYERS: Arc<Mutex<HashMap<String, Arc<Mutex<Sink>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Play an audio file from a given path
/// Returns a unique ID that can be used to control the playback
#[command]
pub fn play_audio_file(path: String) -> Result<String, String> {
    // Create a unique ID for this playback instance
    let playback_id = format!(
        "audio_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis()
    );

    // Start playback in a separate thread to not block the main thread
    let playback_id_clone = playback_id.clone();

    std::thread::spawn(
        move || match play_audio_internal(&path, playback_id_clone) {
            Ok(_) => (),
            Err(e) => eprintln!("Error playing audio: {}", e),
        },
    );

    Ok(playback_id)
}

fn play_audio_internal(path: &str, playback_id: String) -> Result<(), String> {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| format!("Failed to get audio output stream: {}", e))?;

    // Create a sink to control playback
    let sink =
        Sink::try_new(&stream_handle).map_err(|e| format!("Failed to create audio sink: {}", e))?;

    // Open the audio file
    let file =
        File::open(path).map_err(|e| format!("Failed to open audio file {}: {}", path, e))?;

    // Decode the audio file
    let source = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode audio file: {}", e))?;

    // Add the source to the sink
    sink.append(source);

    // Store the sink so it can be controlled later
    let sink_arc = Arc::new(Mutex::new(sink));
    AUDIO_PLAYERS
        .lock()
        .unwrap()
        .insert(playback_id.clone(), sink_arc.clone());

    // Wait for playback to finish
    sink_arc.lock().unwrap().sleep_until_end();

    // Clean up by removing the sink from the HashMap
    AUDIO_PLAYERS.lock().unwrap().remove(&playback_id);

    Ok(())
}

/// Stop audio playback for a given ID
#[command]
pub fn stop_audio(playback_id: String) -> Result<(), String> {
    let players = AUDIO_PLAYERS.lock().unwrap();
    if let Some(sink) = players.get(&playback_id) {
        sink.lock().unwrap().stop();
        Ok(())
    } else {
        Err(format!("No audio player found with ID: {}", playback_id))
    }
}

/// Pause audio playback
#[command]
pub fn pause_audio(playback_id: String) -> Result<(), String> {
    let players = AUDIO_PLAYERS.lock().unwrap();
    if let Some(sink) = players.get(&playback_id) {
        sink.lock().unwrap().pause();
        Ok(())
    } else {
        Err(format!("No audio player found with ID: {}", playback_id))
    }
}

/// Resume audio playback
#[command]
pub fn resume_audio(playback_id: String) -> Result<(), String> {
    let players = AUDIO_PLAYERS.lock().unwrap();
    if let Some(sink) = players.get(&playback_id) {
        sink.lock().unwrap().play();
        Ok(())
    } else {
        Err(format!("No audio player found with ID: {}", playback_id))
    }
}

/// Check if audio is still playing
#[command]
pub fn is_audio_playing(playback_id: String) -> Result<bool, String> {
    let players = AUDIO_PLAYERS.lock().unwrap();
    if let Some(sink) = players.get(&playback_id) {
        Ok(!sink.lock().unwrap().empty())
    } else {
        Ok(false) // If the player doesn't exist, it's not playing
    }
}
