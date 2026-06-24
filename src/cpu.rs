use crate::card::{Card, Rank, Suit};

/// Represents a distinct combination of table card indices that can be captured.
#[derive(Debug, Clone)]
pub enum CaptureOption {
    Direct(usize),               // Direct single-card index match
    Pair(usize, usize),          // Two table indices that sum to target
    Triplet(usize, usize, usize), // Three table indices that sum to target
}

/// A fully calculated candidate move for the CPU to rank.
struct CandidateMove {
    hand_index: usize,
    capture: Option<CaptureOption>,
    score: i32,
}

/// Evaluates a hand card against the table and finds every single valid capture combo.
fn find_all_options_for_card(card: &Card, table: &[Card]) -> Vec<CaptureOption> {
    let mut options = Vec::new();
    let target = card.rank.value();

    // 1. Check for all direct matches
    for (i, table_card) in table.iter().enumerate() {
        if table_card.rank.value() == target {
            options.push(CaptureOption::Direct(i));
        }
    }
    
    // If we found direct matches, we can prioritize those and skip combo checks
    if !options.is_empty() {
        return options;
    }

    // 2. Check for all pair combos
    for i in 0..table.len() {
        for j in (i + 1)..table.len() {
            if table[i].rank.value() + table[j].rank.value() == target {
                options.push(CaptureOption::Pair(i, j));
            }
        }
    }

    // 3. Check for all triplet combos
    for i in 0..table.len() {
        for j in (i + 1)..table.len() {
            for k in (j + 1)..table.len() {
                if table[i].rank.value() + table[j].rank.value() + table[k].rank.value() == target {
                    options.push(CaptureOption::Triplet(i, j, k));
                }
            }
        }
    }

    options
}

/// Helper to check if a specific table card is the 7 of Diamonds (Haya)
fn is_haya(card: &Card) -> bool {
    card.suit == Suit::Diamonds && card.rank == Rank::Seven
}

/// Main entry point for the AI brain. Returns the chosen card index from the CPU's hand.
pub fn choose_best_move(cpu_hand: &[Card], table: &[Card]) -> usize {
    let mut candidates = Vec::new();

    for (hand_idx, card) in cpu_hand.iter().enumerate() {
        let possible_captures = find_all_options_for_card(card, table);

        if possible_captures.is_empty() {
            // SCENARIO A: No matches available. The card will be dropped.
            let mut score = 0;

            // DANGER ZONE EVALUATION:
            // If dropping this card makes the total table sum easy to clear (1-7), apply a penalty
            let table_sum: usize = table.iter().map(|c| c.rank.value()).sum();
            let future_sum = table_sum + card.rank.value();
            if future_sum <= 7 {
                score -= 10; 
            }

            candidates.push(CandidateMove {
                hand_index: hand_idx,
                capture: None,
                score,
            });
        } else {
            // SCENARIO B: Matches are available! Calculate heuristic scores.
            for capture in possible_captures {
                let mut score = 0;

                // Track total cards involved in this capture option
                let total_cards_to_capture = match capture {
                    CaptureOption::Direct(_) => 1,
                    CaptureOption::Pair(_, _) => 2,
                    CaptureOption::Triplet(_, _, _) => 3,
                };

                // Reward capturing multiple cards (+20 per combo) to secure El Kārṭa
                if total_cards_to_capture > 1 {
                    score += 20;
                }

                // Evaluate individual items inside the combination
                match capture {
                    CaptureOption::Direct(i) => {
                        let target_card = &table[i];
                        if is_haya(target_card) || is_haya(card) { score += 100; } // Capture 7♦
                        if target_card.rank == Rank::Seven || card.rank == Rank::Seven { score += 10; } // Capturing a 7
                    }
                    CaptureOption::Pair(i, j) => {
                        let c1 = &table[i];
                        let c2 = &table[j];
                        if is_haya(c1) || is_haya(c2) || is_haya(card) { score += 100; }
                        if c1.rank == Rank::Seven { score += 10; }
                        if c2.rank == Rank::Seven { score += 10; }
                        if card.rank == Rank::Seven { score += 10; }
                    }
                    CaptureOption::Triplet(i, j, k) => {
                        let c1 = &table[i];
                        let c2 = &table[j];
                        let c3 = &table[k];
                        if is_haya(c1) || is_haya(c2) || is_haya(c3) || is_haya(card) { score += 100; }
                        if c1.rank == Rank::Seven { score += 10; }
                        if c2.rank == Rank::Seven { score += 10; }
                        if c3.rank == Rank::Seven { score += 10; }
                        if card.rank == Rank::Seven { score += 10; }
                    }
                }

                // Check for Chkobba (Board Sweep)
                // If the number of cards we are taking matches the table count, it's a board wipe!
                if total_cards_to_capture == table.len() {
                    score += 50;
                }

                candidates.push(CandidateMove {
                    hand_index: hand_idx,
                    capture: Some(capture),
                    score,
                });
            }
        }
    }

    // Find the move with the highest heuristic score
    // If scores are tied, it naturally defaults to the last one checked
    candidates
        .into_iter()
        .max_by_key(|m| m.score)
        .map(|m| m.hand_index)
        .unwrap_or(0) // Fallback safety, though a hand will never be completely empty here
}
/// Evaluates whether a cut card is worth keeping in a vacuum (before table cards are dealt).
pub fn evaluate_blind_cut_card(first_card: &Card) -> bool {
    // 1. Absolute Priority: Sabʿa l-ḥayya (7 of Diamonds) is a mandatory keep.
    if first_card.suit == Suit::Diamonds && first_card.rank == Rank::Seven {
        return true;
    }

    // 2. High Priority: Any other 7 or any 6 (Crucial for El Barmīla)
    if first_card.rank == Rank::Seven || first_card.rank == Rank::Six {
        return true;
    }

    // 3. Medium-High Priority: Aces (High combo potential, worth points in many variations)
    if first_card.rank == Rank::Ace {
        return true;
    }

    // 4. Diamonds (Dīnārī): Any diamond increases chances for the flush point
    if first_card.suit == Suit::Diamonds && first_card.rank.value() <= 5 {
        // Keep low diamonds because they are easy to pair or drop safely
        return true;
    }

    // Default: Reject middle cards or high un-pairable face cards like Queens and Jacks
    false
}