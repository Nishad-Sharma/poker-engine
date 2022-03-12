use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use rand::thread_rng;
use rand::seq::SliceRandom;
use itertools::Itertools;

#[derive(Debug, Eq, PartialEq, Clone, EnumIter, PartialOrd, Ord, Copy)]
pub enum Rank {
    Ace = 14,
    King = 13,
    Queen = 12,
    Jack = 11,
    Ten = 10,
    Nine = 9,
    Eight = 8,
    Seven = 7,
    Six = 6,
    Five = 5,
    Four = 4,
    Three = 3,
    Two = 2
}


#[derive(Debug, Eq, PartialEq, Clone, EnumIter, PartialOrd, Ord, Copy)]
pub enum Suit {
    Spade,
    Heart,
    Club,
    Diamond
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Card {
    rank: Rank,
    suit: Suit
}


impl Card {
    pub fn new(rank:Rank, suit:Suit) -> Card {
        Card{rank,suit}
    }

    pub fn init_deck() -> Vec<Card> {
        let mut vec = Vec::new();
        for s in Suit::iter() {
            for r in Rank::iter() {
                vec.push(Card::new(r.clone(), s.clone()));
            }
        }
        vec.shuffle(&mut thread_rng());
        return vec;
    }

    pub fn get_rank(&mut self) -> Rank {
        return self.rank;
    }

    pub fn get_suit(&mut self) -> Suit {
        return self.suit;
    }
}

