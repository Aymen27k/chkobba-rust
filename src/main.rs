mod card;
mod cpu;
mod stats;
mod audio;
use card::{Rank, Suit, Card};
use stats::{GameStats, save_stats, load_stats};
use strum::IntoEnumIterator;
use rand::{self, Rng, seq::SliceRandom};
use rand::prelude::*;
use colored::*;
use audio::play_sound;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{self, Write};
use itertools::Itertools;



fn draw_cards(deck: &mut Vec<Card>, num_card: usize ) -> Vec<Card> {
    let mut disposal_cards: Vec<Card> = vec![];
    for _ in 0..num_card {
        if let Some(card) = deck.pop() {
            disposal_cards.push(card);
        }
    }
    disposal_cards
}

fn validate_input(player_hand: &[Card]) -> usize {

    let card_index: usize = loop {
    println!("Which card to drop? (1-{})", player_hand.len());
    
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line from terminal");

    // 1. Try to parse the trimmed string into a number
    match input.trim().parse::<usize>() {
        // 2. If it successfully parsed into a number...
        Ok(num) => {
            // 3. Check if the number actually matches a card in their hand
            if num >= 1 && num <= player_hand.len() {
                break num; // Success! Break the loop and return the number
            } else {
                println!("Invalid choice! Please pick a number between 1 and {}.", player_hand.len());
            }
        }
        // 4. If they typed a letter or typo...
        Err(_) => {
            println!("That's not a number! Please enter a valid number.");
        }
    }
    // Loop repeats automatically if we didn't hit the 'break' statement!
};

// Now you can safely do your index adjustment out here!
let real_index = card_index - 1;
real_index
}


// New function to calculate matches
fn find_combination_match(target_value: usize, table: &[Card], combo_size: usize) -> Option<([usize; 3], usize)> {
    // 1. itertools generates index patterns lazily
    for combo_indices in (0..table.len()).combinations(combo_size) {
        let sum: usize = combo_indices.iter()
            .map(|&idx| table[idx].rank.value())
            .sum();
            
        // 2. We found a match!
        if sum == target_value {
            // Allocate a 3-slot array entirely on the STACK, filled with 0s
            let mut stack_array = [0; 3];
            
            // 3. Copy the indices from the temporary iterator into our stack array
            for (i, &idx) in combo_indices.iter().enumerate() {
                stack_array[i] = idx;
            }
            
            // 4. Return the array AND the actual number of cards used (combo_size)
            return Some((stack_array, combo_size));
        }
    }
    None
}

fn process_turn(dropped_card: Card, table: &mut Vec<Card>, won_cards: &mut Vec<Card>, player_name: &str) -> bool {
    let target = dropped_card.rank.value();

    // ==========================================
    // 1. Try single card match (combo_size = 1)
    // ==========================================
    if let Some((mut indices_array, len)) = find_combination_match(target, table, 1) {
        // Slice out the valid section of our stack array
        let valid_indices = &mut indices_array[0..len];
        
        let captured_card = table.remove(valid_indices[0]);
        println!("Boom! A direct match for {}! You captured: {} -> {}", player_name, dropped_card.format_card(), captured_card.format_card());
        
        won_cards.push(captured_card);
        won_cards.push(dropped_card);
        println!("{} has won: {} Card(s)", player_name, won_cards.len());
        return true;
    } 
    
    // ==========================================
    // 2. Try pair combo match (combo_size = 2)
    // ==========================================
    if let Some((mut indices_array, len)) = find_combination_match(target, table, 2) {
        // Slice out the valid section of our stack array
        let valid_indices = &mut indices_array[0..len];
        
        // CRITICAL SYSTEM DESIGN TRICK: Sort indices highest to lowest 
        // so removing valid_indices[0] doesn't shift valid_indices[1] in the vector!
        valid_indices.sort_by(|a, b| b.cmp(a)); 
        
        let card2 = table.remove(valid_indices[0]); // Highest index
        let card1 = table.remove(valid_indices[1]); // Lowest index
        
        println!("Wow! {} found a combo match! {} -> {} + {}", player_name, dropped_card.format_card(), card1.format_card(), card2.format_card());
        won_cards.push(card1);
        won_cards.push(card2);
        won_cards.push(dropped_card);
        println!("{} has won: {} Card(s)", player_name, won_cards.len());
        return true;
    } 
    
    // ==========================================
    // 3. Try three-card combo match (combo_size = 3)
    // ==========================================
    if let Some((mut indices_array, len)) = find_combination_match(target, table, 3) {
        // Slice out the valid section of our stack array
        let valid_indices = &mut indices_array[0..len];
        
        // Sort highest to lowest again for seamless removal
        valid_indices.sort_by(|a, b| b.cmp(a)); 
        
        let captured3 = table.remove(valid_indices[0]); // Highest
        let captured2 = table.remove(valid_indices[1]);
        let captured1 = table.remove(valid_indices[2]); // Lowest

        println!("Insane! {} got a triple combo match!\n{} -> {} + {} + {}", player_name, dropped_card.format_card(), captured1.format_card(), captured2.format_card(), captured3.format_card());
        
        won_cards.push(dropped_card);
        won_cards.push(captured1);
        won_cards.push(captured2);
        won_cards.push(captured3);
        println!("{} has won: {} Card(s)", player_name, won_cards.len());
        return true;
    }

    // 4. No match at all - Card stays on table
    println!("No match for {}. The card stays on the table.", player_name);
    table.push(dropped_card);
    false
}
fn pick_initial_dealer() -> GameEntity {
    let mut rng = rand::rng();
    // Generates a boolean (50/50 chance) to choose the starting dealer
    if rng.random_bool(0.5) {
        GameEntity::Player
    } else {
        GameEntity::Cpu
    }
}
fn select_card_with_arrows(hand: &[Card]) -> usize {
    let mut selected_index = 0;
    
    enable_raw_mode().unwrap();
    
    let result = loop {
        // Dynamic prompt showing current volume status
        let mute_status = if crate::audio::is_muted() { " [MUTED 🔇]" } else { "" };
        print!("\rWhich card to drop?{} ", mute_status);
        
        for (i, card) in hand.iter().enumerate() {
            if i == selected_index {
                print!("> {}  ", card.format_card());
            } else {
                print!("  {}  ", card.format_card());
            }
        }
        // Clear trailing characters if the prompt string changes length
        print!("\x1B[K"); 
        io::stdout().flush().unwrap();

        if let Event::Key(key_event) = event::read().unwrap() {
            if key_event.kind == event::KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Left | KeyCode::Up => {
                        if selected_index > 0 { selected_index -= 1; } 
                        else { selected_index = hand.len() - 1; }
                    }
                    KeyCode::Right | KeyCode::Down => {
                        if selected_index < hand.len() - 1 { selected_index += 1; } 
                        else { selected_index = 0; }
                    }
                    // INTERCEPT THE 'M' KEY FOR MUTING!
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

struct PileStats {
    total_cards: usize,
    diamond_count: usize,
    has_haya: bool,
    sevens_count: usize,
    sixes_count: usize,
}
fn calculate_pile_stats(won_cards: &[Card]) -> PileStats {
    // 1. Count Total Cards
    let total_cards = won_cards.len();

    // 2. Count Diamonds (Dīnārī)
    let diamond_count = won_cards.iter()
        .filter(|card| card.suit == Suit::Diamonds)
        .count();

    // 3. Find the 7 of Diamonds (Sabʿa l-ḥayya)
    let has_haya = won_cards.iter()
        .any(|card| card.suit == Suit::Diamonds && card.rank == Rank::Seven);

    // 4. Count all 7s (For El Barmīla)
    let sevens_count = won_cards.iter()
        .filter(|card| card.rank == Rank::Seven)
        .count();

    // 5. Count all 6s (The Tie-Breaker for El Barmīla)
    let sixes_count = won_cards.iter()
        .filter(|card| card.rank == Rank::Six)
        .count();

    // Return the struct bundling all our chained iterator results!
    PileStats {
        total_cards,
        diamond_count,
        has_haya,
        sevens_count,
        sixes_count,
    }
}
#[derive(Clone, Copy, PartialEq)]
enum GameEntity {
    Player,
    Cpu
}
fn main() {
    let mut player_match_wins = 0;
    let mut cpu_match_wins = 0;
    let mut total_matches_played = 0;
    let mut keep_playing = true;
    let mut current_dealer = pick_initial_dealer();
    /* Sound Backing */
    const SHUFFLE_SOUND: &[u8] = include_bytes!("../assets/shuffle.mp3");
    const NICE_CHKOBBA_SOUND: &[u8] = include_bytes!("../assets/nice_chkobba.mp3");
    const SWEEP_SOUND: &[u8] = include_bytes!("../assets/sweep.mp3");
    const VICTORY_SOUND: &[u8] = include_bytes!("../assets/victory_clap.mp3");
    const LOSS_SOUND: &[u8] = include_bytes!("../assets/lost_game.mp3");
    const TABLE_SLAM_SOUND: &[u8] = include_bytes!("../assets/table_slam.mp3");
    
    while keep_playing {

    let mut deck = vec![];
    let mut rng = rand::rng();
    let mut table: Vec<Card> = vec![];
    let mut player_hand: Vec<Card> = vec![];
    let mut cpu_hand: Vec<Card> = vec![];
    let mut player_won_cards: Vec<Card> = vec![];
    let mut cpu_won_cards: Vec<Card> = vec![];
    let mut game_over = false;
    let mut last_capture: Option<GameEntity> = None;
    let mut player_chkobbas = 0;
    let mut cpu_chkobbas = 0;
    let mut round_number = 0;
    let mut game_stats = load_stats();
    let win_rate = game_stats.win_percentage();

    
    /*/ --- TEST LAB: HARDCODED CARDS ---
    // Manually build a table to test specific scenarios
    table = vec![
        Card { suit: Suit::Spades, rank: Rank::Five },

    ];

    // Manually build your player hand
    player_hand = vec![
        Card { suit: Suit::Clubs, rank: Rank::Five },
        Card { suit: Suit::Diamonds, rank: Rank::Six },
        Card { suit: Suit::Spades, rank: Rank::Seven }
    ];
    // ---------------------------------*/
    println!("{}", "=================================".red());
    println!("{}", "       WELCOME TO CHKOBBA!       ".yellow().bold().on_red());
    println!("{}", "=================================".red());

    // Load historical stats from JSON file
    if game_stats.total_games > 0 {
        println!("💾 Historical Stats Loaded! Total Matches Played: {}, Player Wins: {}, CPU Wins: {}", 
                 game_stats.total_games, game_stats.player_wins, game_stats.cpu_wins);
        println!("🏆 Player Win Percentage: {:.1}%", win_rate);
        if win_rate >= 60.0 {
            println!("{}", "🔥 Status: Absolute Chkobba Legend! The CPU fears you. 🔥".green());
        } else if win_rate >= 50.0 {
            println!("{}", "📈 Status: Holding your ground! You're beating the machine.".cyan());
        } else {
            println!("{}", "📉 Status: The CPU is dominating you. Time to lock in! 🤖".red());
        }
    } else {
        println!("💾 No historical stats found. Starting fresh!");
    }


    //  Dealer printing
    if total_matches_played == 0 {
        println!(
            "\n🎲 The RNG gods have spoken! {} will deal first!", 
            match current_dealer {
                GameEntity::Player => "👤 You (Player)",
                GameEntity::Cpu => "🤖 CPU"
            }
        );
    } else {
        println!(
            "\n👑 Match {}: {} is the Dealer!", 
            total_matches_played + 1, // +1 because 1 game played means we are starting match 2!
            match current_dealer {
                GameEntity::Player => "👤 Player",
                GameEntity::Cpu => "🤖 CPU"
            }
        );
    }
        
    
    // Creating the deck from both Suits and Ranks
    for s in Suit::iter() {
        for r in Rank::iter() {
            let new_card = Card {
                suit: s,
                rank: r,
            };
            deck.push(new_card);
        }
        


    }
    // In your initialization section
    play_sound(SHUFFLE_SOUND, 2000);
    deck.shuffle(&mut rng);
    while !game_over {
        let mut round_over = false;
        println!("Deck : {:?}", &deck.len());
        // Initial table setup - Draw 1 card for the cut
        if round_number == 0 {
            let mut drawn_list = draw_cards(&mut deck, 1);
            let first_card = drawn_list.remove(0); 
            let mut keep_card = false;

                match current_dealer {
                    GameEntity::Cpu => {
                        // Player gets to choose, so the player HAS to see it!
                        println!("The cut card drawn is: {}", first_card.format_card());
                        loop {
                            println!("Do you want to keep this card in your hand? (y/n)");
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input).unwrap();
                            let trimmed = input.trim().to_lowercase();

                            if trimmed == "y" {
                                keep_card = true;
                                break;
                            } else if trimmed == "n" {
                                keep_card = false;
                                break;
                            } else {
                                println!("❌ Invalid input. Please enter exactly 'y' or 'n'.");
                            }
                        }
                    }
                    GameEntity::Player => {
                        // CPU gets to choose! Keep the card identity hidden from the player log.
                        if cpu::evaluate_blind_cut_card(&first_card) {
                            println!("🤖 CPU cuts a card, smiles, and slips it secretly into its hand!");
                            keep_card = true;
                        } else {
                            println!("🤖 CPU decides: No thanks, put it on the table.");
                            keep_card = false;
                        }
                    }
                }
            

            // Execute the decision cleanly based on the boolean state
            if keep_card {
                if current_dealer == GameEntity::Cpu {
                    player_hand.push(first_card);
                } else {
                    cpu_hand.push(first_card);
                }
                table = draw_cards(&mut deck, 4);
            } else {
                println!("The card goes face up on the table: {}", first_card.format_card());
                table.push(first_card);
                table.extend(draw_cards(&mut deck, 3));
            }
        }
        // Setting up hands and table cards:
        if player_hand.is_empty() {
             player_hand = draw_cards(&mut deck, 3);
        } else {
            let extra_cards = draw_cards(&mut deck, 2);
            player_hand.extend(extra_cards);
        }
        if cpu_hand.is_empty() {
            cpu_hand = draw_cards(&mut deck, 3);
        } else {
            let extra_cards = draw_cards(&mut deck, 2);
            cpu_hand.extend(extra_cards);      
        }    
        
        round_number += 1;
        // Inside your main game loop when a new round starts
        println!("\n{}", format!("--- ROUND {} ---", round_number).blue().bold());

/* ------------------ Game Loop ------------------ */
    // If Player dealt, CPU goes first. If CPU dealt, Player goes first.
    let mut current_turn = if current_dealer == GameEntity::Player {
        GameEntity::Cpu
    } else {
        GameEntity::Player
    };

    while !round_over {        
        let display_table = table   
            .iter()
            .map(|card| card.format_card())
            .join(" ");
        println!("We have on the table: {}", display_table);
        println!("{}","=======================================================".dimmed());
        
        // 🎯 EXECUTE DYNAMIC TURN BRANCH
        match current_turn {
            GameEntity::Player => {
            if !player_hand.is_empty() {
                println!("{}", "--- PLAYER TURN ---".green().bold());
                
                /* 💡 COMMENTED OUT OLD METHOD FOR BACKUP:
                let display_player_hand = player_hand
                    .iter()
                    .map(|card| card.format_card())
                    .join(" ");
                println!("We have on the player_hand: {}", display_player_hand);
                */

                let real_index = if player_hand.len() == 1 && table.is_empty(){
                    println!("\n[Auto-Play] Table is empty! Automatically dropping your last card.");
                    0
                } else {
                    // 🎯 NEW INTERACTIVE MENU
                    select_card_with_arrows(&player_hand)
                    
                    // 💡 COMMENTED OUT OLD VALIDATION FALLBACK:
                    // validate_input(&player_hand)
                };

                let dropped_card = player_hand.remove(real_index);
                let captured = process_turn(dropped_card, &mut table, &mut player_won_cards, "Player");
                
                if captured {
                    last_capture = Some(GameEntity::Player);
                    if table.is_empty() {
                        println!("{}", "💥 Chkobba! Player cleared the table and gets a bonus point! 💥".yellow().bold().on_black());
                        player_chkobbas += 1;
                        play_sound(NICE_CHKOBBA_SOUND, 2000);
                    }
                }
            }
            println!("{}","=======================================================".dimmed());
            // 🏁 Pass baton to CPU
            current_turn = GameEntity::Cpu;
        }
            GameEntity::Cpu => {
                // Only run CPU turn if it actually has cards!
                if !cpu_hand.is_empty() {
                    println!("{}","--- CPU TURN ---".yellow().bold());


                    // Debugging logs to visualize CPU hand before decision
                    /*let display_cpu_hand = cpu_hand
                        .iter()
                        .map(|card| card.format_card())
                        .join(" ");
                    println!("We have on the cpu_hand: {}", display_cpu_hand);*/
                    
                    let best_move_index = cpu::choose_best_move(&cpu_hand, &table);
                    let cpu_dropped_card = cpu_hand.remove(best_move_index);
                    println!("CPU decides to play: {}", cpu_dropped_card.format_card());
                    
                    let captured = process_turn(cpu_dropped_card, &mut table, &mut cpu_won_cards, "CPU");
                    if captured {
                        last_capture = Some(GameEntity::Cpu);
                        if table.is_empty() {
                            println!("{}", "💥 Chkobba! CPU cleared the table and gets a bonus point! 💥".yellow().bold().on_black());
                            cpu_chkobbas += 1;
                            play_sound(TABLE_SLAM_SOUND, 2000);
                        }
                    }
                }
                println!("{}","=======================================================".dimmed());
                // 🏁 Pass baton to Player
                current_turn = GameEntity::Player;
            }
        }

        // --- CHECK TURN-END CONDITIONS ---
        if player_hand.is_empty() && cpu_hand.is_empty() {
            round_over = true;
            println!("\nBoth players are out of cards! Round Over!");
        }
    }
    /*------------------ End of game Loop ------------------ */
    if deck.is_empty() {
        // Check if there are actually any cards left stranded in the center for FINAL SWEEP
        if !table.is_empty() {
            println!("\n--- FINAL SWEEP ---");
            println!("Cards left on the table go to the last player who made a capture!");
            play_sound(SWEEP_SOUND, 1000);

            match last_capture {
                Some(GameEntity::Player) => {
                    println!("{} gets the final sweep!", "Player".green().bold());
                    // Append all remaining table cards into the player's won pile
                    player_won_cards.append(&mut table);
                }
                Some(GameEntity::Cpu) => {
                    println!("{} gets the final sweep!", "CPU".red().bold());
                    // Append all remaining table cards into the CPU's won pile
                    cpu_won_cards.append(&mut table);
                }
                None => {
                    println!("{}", "No one made a single capture all game! The remaining cards are discarded.".dimmed());
                }
            }
        }

        /* Winning condition */
        let player_stats = calculate_pile_stats(&player_won_cards);
        let cpu_stats = calculate_pile_stats(&cpu_won_cards);

        let mut player_final_score = player_chkobbas;
        let mut cpu_final_score = cpu_chkobbas;

        println!("{}","\n=============================".red());
        println!("{}","    FINAL CHKOBBA SCOREBOARD  ".white().on_red().bold());
        println!("{}","=============================".red());

        if player_stats.total_cards > cpu_stats.total_cards {
            player_final_score += 1;
            // 1. Format the string with the variable first, 2. Paint it, 3. Turn to string!
            let text = format!("🃏 Kārṭa: +1 Point to Player ({})", player_stats.total_cards).green().bold().to_string();
            println!("{}", text);
        } else if cpu_stats.total_cards > player_stats.total_cards {
            cpu_final_score += 1;
            let text = format!("🃏 Kārṭa: +1 Point to CPU ({})", cpu_stats.total_cards).red().bold().to_string();
            println!("{}", text);
        } else {
            println!("{}", "🃏 Kārṭa: Tie at 20-20! (0 points awarded)".dimmed());
        }

        // 2. Dīnārī (Most Diamonds - 1 Point)
        if player_stats.diamond_count > cpu_stats.diamond_count {
            player_final_score += 1;
            let text = format!("♦️ Dīnārī: +1 Point to Player ({})", player_stats.diamond_count).green().bold().to_string();
            println!("{}", text);
        } else if cpu_stats.diamond_count > player_stats.diamond_count {
            cpu_final_score += 1;
            let text = format!("♦️ Dīnārī: +1 Point to CPU ({})", cpu_stats.diamond_count).red().bold().to_string();
            println!("{}", text);
        } else {
            println!("{}", "♦️ Dīnārī: Tie at 5-5! (0 points awarded)".dimmed());
        }

        // 3. Sab‘a l-ḥayya (7 of Diamonds - 1 Point)
        if player_stats.has_haya {
            player_final_score += 1;
            let text = format!("✨ Sab‘a l-ḥayya (7♦️): +1 Point to Player!").green().bold().to_string();
            println!("{}", text);
        } else if cpu_stats.has_haya {
            cpu_final_score += 1;
            let text = format!("✨ Sab‘a l-ḥayya (7♦️): +1 Point to CPU!").red().bold().to_string();
            println!("{}", text);
        }

        // 4. Barmīla (The Prime / Most Sevens - 1 Point)
        if player_stats.sevens_count > cpu_stats.sevens_count {
            player_final_score += 1;
            let text = format!("🔥 Barmīla (Most 7s): +1 Point to Player ({})", player_stats.sevens_count).green().bold().to_string();
            println!("{}", text);
        } else if cpu_stats.sevens_count > player_stats.sevens_count {
            cpu_final_score += 1;
            let text = format!("🔥 Barmīla (Most 7s): +1 Point to CPU ({})", cpu_stats.sevens_count).red().bold().to_string();
            println!("{}", text);
        } else {
            // Tie-breaker: Check 6s if 7s are equal
            if player_stats.sixes_count > cpu_stats.sixes_count {
                player_final_score += 1;
                let text = format!("🔥 Barmīla (Tie broken by Sixes): +1 Point to Player!").green().bold().to_string();
                println!("{}", text);
            } else if cpu_stats.sixes_count > player_stats.sixes_count {
                cpu_final_score += 1;
                let text = format!("🔥 Barmīla (Tie broken by Sixes): +1 Point to CPU!").red().bold().to_string();
                println!("{}", text);
            } else {
                println!("{}","🔥 Barmīla: Complete Tie! (0 points awarded)".dimmed());
            }
        }

    // --- THE GRAND FINALE ---
            println!("-----------------------------");
            println!("Bonus Chkobbas during game: Player: {}, CPU: {}", player_chkobbas, cpu_chkobbas);
            println!("-----------------------------");
            println!("🏆 FINAL MATCH SCORE 🏆");

            if player_final_score > cpu_final_score {
                // Player won! Color player block, dim cpu block
                let p_block = format!(" PLAYER: {} Points ", player_final_score).green().bold().on_white().to_string();
                let c_block = format!("CPU: {} Points", cpu_final_score).dimmed().to_string();
                game_stats.player_wins += 1;
                player_match_wins += 1;
                play_sound(VICTORY_SOUND, 17000);
                println!("{} | {}", p_block, c_block);
                
            } else if cpu_final_score > player_final_score {
                // CPU won! Dim player block, color cpu block
                let p_block = format!("PLAYER: {} Points", player_final_score).dimmed().to_string();
                let c_block = format!(" CPU: {} Points ", cpu_final_score).red().bold().on_white().to_string();
                game_stats.cpu_wins += 1;
                cpu_match_wins += 1;
                play_sound(LOSS_SOUND, 5000);
                println!("{} | {}", p_block, c_block);
                
            } else {
                // It's a draw: Both get a subtle yellow style
                let p_block = format!("PLAYER: {} Points", player_final_score).yellow().to_string();
                let c_block = format!("CPU: {} Points", cpu_final_score).yellow().to_string();
                println!("{} | {}", p_block, c_block);
            }

            game_stats.total_games += 1;
            total_matches_played += 1;
            current_dealer = if current_dealer == GameEntity::Player {
                GameEntity::Cpu
            } else {
                GameEntity::Player
            };

            println!("=============================");
            println!("Deck is empty! Game Over!");

            // Save the updated stats to the JSON file
            if let Err(e) = save_stats(&game_stats) {
                eprintln!("Warning: Failed to save stats to disk: {}", e);
            } else {
                println!("{}", "💾 Historical leaderboard updated securely! 💾".dimmed());
            }

        game_over = true;
    }
    
    }
    // Asking the player if they want to play again after the game loop ends
    loop {
        println!("\n🔄 Do you want to play another match? (y/n)");
        let mut response = String::new();
        std::io::stdin().read_line(&mut response).unwrap();
        let trimmed = response.trim().to_lowercase();

        if trimmed == "y" {
            println!("\nShuffling the deck for a new match... Let's go! 🃏");
            break; // Breaks this prompt loop, lets the master loop restart fresh!
        } else if trimmed == "n" {
            println!("\nThanks for playing Chkobba! See you next time! 👋✨");
            keep_playing = false; // Tells the master loop to exit completely
            break; 
        } else {
            println!("❌ Invalid input. Please enter exactly 'y' or 'n'.");
        }
    }
    }
    // 📊 THE GRAND SESSION OVERVIEW 📊
    // This runs completely outside the loop, right before the program exits!
    println!("\n{}", "=================================".yellow().bold());
    println!("{}", "       SESSION OVERVIEW          ".white().on_yellow().bold());
    println!("{}", "=================================".yellow().bold());
    println!("Total Matches Played: {}", total_matches_played);
    println!("🏆 Player Match Victories: {}", player_match_wins.to_string().green().bold());
    println!("🤖 CPU Match Victories:    {}", cpu_match_wins.to_string().red().bold());
    
    // Quick custom flavor text based on performance!
    if player_match_wins > cpu_match_wins {
        println!("\n👑 Final Verdict: You dominated the CPU today! Great games!");
    } else if cpu_match_wins > player_match_wins {
        println!("\n🦾 Final Verdict: The machine won this round. Better luck next time!");
    } else {
        println!("\n🤝 Final Verdict: A perfect stalemate! An honorable draw.");
    }
    println!("=================================\n");
}
