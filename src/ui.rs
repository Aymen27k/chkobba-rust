// src/ui.rs

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use crate::card::Card;

/// Generic arrow-navigation helper for terminal selections
pub fn select_item_with_arrows<T>(
    options: &[T],
    get_prompt: impl Fn() -> String,
    format_option: impl Fn(&T) -> String,
) -> usize {
    let mut selected_index = 0;

    enable_raw_mode().unwrap();

    let result = loop {
        // Print dynamic prompt (evaluated every render frame)
        print!("\r{} ", get_prompt());

        for (i, item) in options.iter().enumerate() {
            let formatted_item = format_option(item);
            if i == selected_index {
                print!("> {}  ", formatted_item);
            } else {
                print!("  {}  ", formatted_item);
            }
        }

        // Clear trailing characters from line length shifts
        print!("\x1B[K");
        io::stdout().flush().unwrap();

        if let Event::Key(key_event) = event::read().unwrap() {
            if key_event.kind == event::KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Left | KeyCode::Up => {
                        if selected_index > 0 {
                            selected_index -= 1;
                        } else {
                            selected_index = options.len() - 1;
                        }
                    }
                    KeyCode::Right | KeyCode::Down => {
                        if selected_index < options.len() - 1 {
                            selected_index += 1;
                        } else {
                            selected_index = 0;
                        }
                    }
                    // Handle mute globally for all arrow selectors!
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        crate::audio::toggle_mute();
                    }
                    KeyCode::Enter => {
                        break selected_index;
                    }
                    KeyCode::Esc => {
                        disable_raw_mode().unwrap();
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
    };

    disable_raw_mode().unwrap();
    println!();
    result
}

/// Helper wrapper for selecting a card to drop from hand
pub fn select_card_with_arrows(hand: &[Card]) -> usize {
    select_item_with_arrows(
        hand,
        || {
            let mute_status = if crate::audio::is_muted() { " [MUTED 🔇]" } else { "" };
            format!("Which card to drop?{}", mute_status)
        },
        |card| card.format_card(),
    )
}

/// Helper wrapper for selecting between multiple capture targets/combos
pub fn select_capture_with_arrows(captures: &[Vec<usize>], table: &[Card], dropped_card: &Card) -> usize {
    let prompt_label = if captures.iter().all(|combo| combo.len() == 1) {
        "Select target:"
    } else {
        "Choose combo:"
    };

    select_item_with_arrows(
        captures,
        || {
            let mute_status = if crate::audio::is_muted() { " [MUTED 🔇]" } else { "" };
            format!(
                "Multiple captures for {}! {}{}",
                dropped_card.format_card(),
                prompt_label,
                mute_status
            )
        },
        |combo| {
            let combo_str = combo
                .iter()
                .map(|&idx| table[idx].format_card())
                .collect::<Vec<_>>()
                .join(" + ");
            format!("[ {} ]", combo_str)
        },
    )
}