use crate::card::{Card, Rank, Suit};
use rand::RngExt;

/// Represents a distinct combination of table card indices that can be captured.
#[derive(Debug, Clone)]
pub enum CaptureOption {
    Direct(usize),               // Direct single-card index match
    Pair(usize, usize),          // Two table indices that sum to target
    Triplet(usize, usize, usize), // Three table indices that sum to target
}

impl CaptureOption {
    pub fn to_indices(&self) -> Vec<usize> {
        match self {
            CaptureOption::Direct(i) => vec![*i],
            CaptureOption::Pair(i, j) => vec![*i, *j],
            CaptureOption::Triplet(i, j, k) => vec![*i, *j, *k],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Hard,
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

    // ⚖️ STRICT CHKOBBA RULE: Direct matches take absolute priority!
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

/// Main entry point for the AI brain. 
/// Returns (hand_card_index, chosen_capture_option).
pub fn choose_best_move(
    cpu_hand: &[Card], 
    table: &[Card], 
    difficulty: Difficulty
) -> (usize, Option<CaptureOption>) {
    let mut candidates = Vec::new();

    for (hand_idx, card) in cpu_hand.iter().enumerate() {
        let possible_captures = find_all_options_for_card(card, table);

        if possible_captures.is_empty() {
            let mut score = 0;
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
            for capture in possible_captures {
                let mut score = 0;
                let total_cards_to_capture = match capture {
                    CaptureOption::Direct(_) => 1,
                    CaptureOption::Pair(_, _) => 2,
                    CaptureOption::Triplet(_, _, _) => 3,
                };

                if total_cards_to_capture > 1 { score += 20; }

                match capture {
                    CaptureOption::Direct(i) => {
                        let target_card = &table[i];
                        if is_haya(target_card) || is_haya(card) { score += 100; }
                        if target_card.rank == Rank::Seven || card.rank == Rank::Seven { score += 10; }
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

    // Return the full candidate (both hand index and capture choice)
    // Sort candidates from highest score to lowest score
    // Sort candidates from highest score to lowest score
    candidates.sort_by(|a, b| b.score.cmp(&a.score));

    let mut rng = rand::rng();

    match difficulty {
        Difficulty::Easy => {
            // 🎲 60% chance to pick the top move, 40% chance to pick a random/sub-optimal one
            if rng.random_bool(0.60) {
                candidates
                    .first()
                    .map(|m| (m.hand_index, m.capture.clone()))
                    .unwrap_or((0, None))
            } else {
                let choice_idx = rng.random_range(0..candidates.len());
                let chosen = &candidates[choice_idx];
                (chosen.hand_index, chosen.capture.clone())
            }
        }
        Difficulty::Hard => {
            // 🎲 80% chance to pick the optimal move, 20% chance to slip up
            if rng.random_bool(0.80) {
                candidates
                    .first()
                    .map(|m| (m.hand_index, m.capture.clone()))
                    .unwrap_or((0, None))
            } else {
                let choice_idx = rng.random_range(0..candidates.len());
                let chosen = &candidates[choice_idx];
                (chosen.hand_index, chosen.capture.clone())
            }
        }
    }
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