// src/audio.rs

use std::io::Cursor;
use std::thread;
use rodio::{Decoder, Player};
// Import AtomicBool and Ordering for thread-safe global state
use std::sync::atomic::{AtomicBool, Ordering};

// 1. Create a global mutable boolean that defaults to false (unmuted)
static IS_MUTED: AtomicBool = AtomicBool::new(false);

/// Toggles the global mute state and returns the new status (true = muted, false = unmuted)
pub fn toggle_mute() -> bool {
    // fetch_xor flips true to false, and false to true atomically
    let previous = IS_MUTED.fetch_xor(true, Ordering::SeqCst);
    !previous // Return the current state after the flip
}

/// Helper to check if the game is currently muted
pub fn is_muted() -> bool {
    IS_MUTED.load(Ordering::SeqCst)
}

pub fn play_sound(audio_bytes: &'static [u8]) {
    if is_muted() {
        return;
    }

    thread::spawn(move || {
        if let Ok(mut handle) = rodio::DeviceSinkBuilder::open_default_sink() {
            handle.log_on_drop(false);
            
            let player = Player::connect_new(&handle.mixer());
            let cursor = Cursor::new(audio_bytes);
            
            if let Ok(source) = Decoder::try_from(cursor) {
                player.append(source);
                player.sleep_until_end();
            }
        }
    });
}