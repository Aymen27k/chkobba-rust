use std::fmt::format;
use colored::*;
use crate::card::Suit::Spades;

#[derive(strum::EnumIter, Clone, PartialEq, Copy)]
#[derive(Debug)]
pub enum Suit {
    Hearts,
    Spades,
    Diamonds,
    Clubs,

}
#[derive(strum::EnumIter, Clone, PartialEq,Copy)]
#[derive(Debug)]
pub enum Rank {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Queen = 8,
    Jack = 9,
    King = 10
}
#[derive(Debug)]
pub struct Card {
    pub suit : Suit,
    pub rank: Rank,
}
impl Suit {
    pub fn symbol(&self) -> &str {
        match self {
        Suit::Hearts => "♥",
        Suit::Clubs => "♣",
        Suit::Diamonds => "♦",
        Suit::Spades => "♠",

        }
    }
    
}
impl Rank {
    pub fn card_value(&self) -> &str {
    match self {
    Rank::Ace => "A",
    Rank::Two => "2",
    Rank::Three => "3",
    Rank::Four => "4",
    Rank::Five => "5",
    Rank::Six => "6",
    Rank::Seven => "7",
    Rank::Queen => "Q",
    Rank::Jack => "J",
    Rank::King => "K",

    }
}
    pub fn value (&self) -> usize {
        *self as usize
    }

}
impl Card {
    pub fn format_card(&self) -> String {
        let card_suit = self.suit.symbol();
        let card_value = self.rank.card_value();
        
        let raw_display = format!(" {}{} ", card_value, card_suit); // Added spaces so the background block looks padded and clean
        
        let colored_card = match self.suit {
            // Diamonds: Red text on White background
            Suit::Diamonds => {
                raw_display.red().bold().on_white().to_string()
            }
            // Hearts: White text on Red background
            Suit::Hearts => {
                raw_display.white().bold().on_red().to_string()
            }
            // Clubs: White text on Black background
            Suit::Clubs => {
                raw_display.white().bold().on_black().to_string()
            }
            // Spades: Black text on White background
            Suit::Spades => {
                raw_display.black().bold().on_white().to_string()
            }
        };
        
        // Wrap the beautifully colored block in standard brackets so it still fits your current layout style
        format!("{}", colored_card)
    }
}