use std::cmp::Reverse;
use std::ptr::null;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use rand::thread_rng;
use rand::seq::SliceRandom;
use itertools::{cloned, Itertools};
use crate::card::{Card, Rank};
use crate::HandRanking::{FourOfAKind, StraightFlush};


mod card;

#[derive(Eq,PartialEq,Debug,Clone)]
struct Player {
    pub name: String,
    chip_stack: u64,
    current_bet: u64,
    has_folded: bool,
    final_action: bool,
    hole_cards: Vec<card::Card>,
    strongest_combo: Vec<card::Card>,
    hand_rank: HandRanking
}

impl Player {
    fn new(name:String, chip_stack:u64) -> Player {
        Player{name, chip_stack, current_bet: 0, has_folded: false, final_action: false, hole_cards: Vec::with_capacity(2), strongest_combo: Vec::new(), hand_rank: HandRanking::HighCard }
    }
}

#[derive(Debug)]
enum ActionType {
    CHECK,
    CALL,
    FOLD,
    RAISE,
    BLIND
}

#[derive(Debug,PartialEq,Clone)]
enum GameStreet {
    PRE,
    FLOP,
    TURN,
    RIVER,
    SHOWDOWN
}

#[derive(Debug,Eq,PartialEq,Clone)]
enum HandRanking {
    StraightFlush = 8,
    FourOfAKind = 7,
    FullHouse = 6,
    Flush = 5,
    Straight = 4,
    ThreeOfAKind = 3,
    TwoPair = 2,
    Pair = 1,
    HighCard = 0,
}


#[derive(Debug)]
struct Action {
    action: ActionType,
    player: Player,
    bet_size: u64,
    street: GameStreet
}

#[derive(Debug)]
struct InvalidActionError;

#[derive(Debug)]
struct Game {
    pub players: Vec<Player>,
    pub start_stack: u64,
    pub button: u64,
    pub actions: Vec<Action>,
    pub big_blind: u64,
    pub pot: u64,
    pub previous_raise: u64,
    pub previous_bet: u64,
    pub current_bet: u64,
    pub turn_marker: u64,
    pub street: GameStreet,
    pub deck: Vec<card::Card>,
    pub board: Vec<card::Card>,
    pub winners: Vec<Player>,
}

impl Game {
    fn new(start_stack:u64, big_blind:u64) -> Game {
        Game{players: Vec::with_capacity(9), start_stack, button:0, actions: Vec::new(), big_blind, pot: 0, previous_raise: 0, previous_bet: 0, current_bet: 0, turn_marker: 1, street: GameStreet::PRE, deck: Vec::new(), board: Vec::with_capacity(5), winners: Vec::new() }
    }

    fn add_player(&mut self, name:String) {
        self.players.push(Player::new(name, self.start_stack));
    }

    fn deal_hole_cards(&mut self) {
        let original_turn_marker = self.turn_marker;

        self.turn_marker = self.button+1;
        for i in (1..3).step_by(1) {
            for j in (1..self.players.len()+1).step_by(1) {
                self.increment_turn();
                self.players[self.turn_marker as usize].hole_cards.push(self.deck.pop().unwrap());
            }
        }
        self.turn_marker = original_turn_marker;
    }

    fn init_deck(&mut self) {
        self.deck = card::Card::init_deck();
    }

    fn find_winner(&mut self) -> Result<(), InvalidActionError> {
        if self.street != GameStreet::SHOWDOWN {
            return Err(InvalidActionError);
        }
        let mut possible_winners = Vec::new();
        for p in self.players.iter(){
            if !(p.has_folded) {
                possible_winners.push(p.clone());
            }
        }
        let mut best_rank = HandRanking::HighCard;

        let mut i = 0;

        while i < self.players.len() {
            let best_hand = self.evaluate_hand(self.players[i].clone().hole_cards);
            self.players[i].strongest_combo = best_hand.clone();
            self.players[i].hand_rank = self.rank_five_card_combo(best_hand.clone());

            if self.players[i].hand_rank.clone() as u8 > best_rank.clone() as u8 {
                best_rank = self.players[i].hand_rank.clone();
            }
            i += 1;
        }

        for mut p in possible_winners.iter_mut() {
            let best_hand = self.evaluate_hand(p.clone().hole_cards);
            p.strongest_combo = best_hand.clone();
            p.hand_rank = self.rank_five_card_combo(best_hand.clone());
            if p.hand_rank.clone() as u8 > best_rank.clone() as u8 {
                best_rank = p.hand_rank.clone();
            }
        }
        possible_winners.retain(|x| x.hand_rank == best_rank.clone());
        if possible_winners.len() == 1 {
            self.winners.push(possible_winners[0].clone());
        } else if best_rank == HandRanking::StraightFlush {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_straight(p.clone().strongest_combo) {
                    highest_rank = self.rank_straight(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_straight(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        } else if best_rank == HandRanking::FourOfAKind {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_four_of_a_kind(p.clone().strongest_combo) {
                    highest_rank = self.rank_four_of_a_kind(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_four_of_a_kind(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        } else if best_rank == HandRanking::FullHouse {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_full_house(p.clone().strongest_combo) {
                    highest_rank = self.rank_full_house(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_full_house(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        } else if best_rank == HandRanking::Flush {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_flush(p.clone().strongest_combo) {
                    highest_rank = self.rank_flush(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_flush(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        } else if best_rank == HandRanking::Straight {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_straight(p.clone().strongest_combo) {
                    highest_rank = self.rank_straight(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_straight(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        } else if best_rank == HandRanking::ThreeOfAKind {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_three_of_a_kind(p.clone().strongest_combo) {
                    highest_rank = self.rank_three_of_a_kind(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_three_of_a_kind(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        } else if best_rank == HandRanking::TwoPair {
            let mut highest_rank = 0;
            for p in possible_winners.clone() {
                if highest_rank < self.rank_two_pair(p.clone().strongest_combo) {
                    highest_rank = self.rank_two_pair(p.clone().strongest_combo);
                }
            }
            for p in possible_winners.clone() {
                if self.rank_two_pair(p.clone().strongest_combo) == highest_rank {
                    self.winners.push(p.clone());
                }
            }
        }
        Ok(())
    }

    fn rank_high_card(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        for h in hand.clone() {
            rank += h.clone().get_rank() as u8;
        }
        return rank as u64;
    }

    fn rank_pair(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        if hand[0].clone().get_rank() as u8 == hand[1].clone().get_rank() as u8 {
            rank = hand[0].clone().get_rank() as u64 * 1000 + hand[2].clone().get_rank() as u64 + hand[3].clone().get_rank() as u64 + hand[4].clone().get_rank() as u64;
        } else if hand[1].clone().get_rank() as u64 == hand[2].clone().get_rank() as u64 {
            rank = hand[1].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 + hand[3].clone().get_rank() as u64 + hand[4].clone().get_rank() as u64;
        } else if hand[2].clone().get_rank() as u64 == hand[3].clone().get_rank() as u64 {
            rank = hand[2].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 + hand[1].clone().get_rank() as u64 + hand[4].clone().get_rank() as u64;
        } else if hand[3].clone().get_rank() as u64 == hand[4].clone().get_rank() as u64 {
            rank = hand[3].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 + hand[1].clone().get_rank() as u64 + hand[2].clone().get_rank() as u64;
        }
        return rank as u64;
    }

    fn rank_two_pair(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        if hand[0].clone().get_rank() as u8 == hand[1].clone().get_rank() as u8 &&
           hand[2].clone().get_rank() as u8 == hand[3].clone().get_rank() as u8 {
            rank = hand[2].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 * 100 + hand[4].clone().get_rank() as u64;
        } else if hand[0].clone().get_rank() as u8 == hand[1].clone().get_rank() as u8 &&
           hand[3].clone().get_rank() as u8 == hand[4].clone().get_rank() as u8 {
            rank = hand[3].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 * 100 + hand[2].clone().get_rank() as u64;
        } else if hand[1].clone().get_rank() as u8 == hand[2].clone().get_rank() as u8 &&
           hand[3].clone().get_rank() as u8 == hand[4].clone().get_rank() as u8 {
            rank = hand[3].clone().get_rank() as u64 * 1000 + hand[1].clone().get_rank() as u64 * 100 + hand[0].clone().get_rank() as u64;
        }
        return rank as u64;
    }

    fn rank_three_of_a_kind(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        if hand[0].clone().get_rank() as u8 == hand[1].clone().get_rank() as u8 &&
           hand[1].clone().get_rank() as u8 == hand[2].clone().get_rank() as u8 {
            rank = hand[0].clone().get_rank() as u64 * 1000 + hand[3].clone().get_rank() as u64 + hand[4].clone().get_rank() as u64;
        } else if hand[1].clone().get_rank() as u8 == hand[2].clone().get_rank() as u8 &&
           hand[2].clone().get_rank() as u8 == hand[3].clone().get_rank() as u8 {
            rank = hand[1].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 + hand[4].clone().get_rank() as u64;
        } else if hand[2].clone().get_rank() as u8 == hand[3].clone().get_rank() as u8 &&
           hand[3].clone().get_rank() as u8 == hand[4].clone().get_rank() as u8 {
            rank = hand[2].clone().get_rank() as u64 * 1000 + hand[0].clone().get_rank() as u64 + hand[1].clone().get_rank() as u64;

        }
        return rank as u64;
    }

    fn rank_flush(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        for h in hand.clone() {
            rank += h.clone().get_rank() as u8;
        }
        return rank as u64;
    }

    fn rank_full_house(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        if hand[0].clone().get_rank() as u8 == hand[1].clone().get_rank() as u8 && hand[1].clone().get_rank() as u8 == hand[2].clone().get_rank() as u8 {
            rank = (hand[2].clone().get_rank() as u64 * 1000) + hand[4].clone().get_rank() as u64;
        } else {
            rank = (hand[3].clone().get_rank() as u64 * 1000) + hand[0].clone().get_rank() as u64;
        }
        return rank as u64
    }

    fn rank_four_of_a_kind(&mut self, hand: Vec<card::Card>) -> u64 {
        let mut rank = 0;
        if hand[0].clone().get_rank() as u8 == hand[1].clone().get_rank() as u8 {
            rank = (hand[2].clone().get_rank() as u64 * 1000) + hand[4].clone().get_rank() as u64;
        } else {
            rank = (hand[2].clone().get_rank() as u64 * 1000) + hand[0].clone().get_rank() as u64;
        }
        return rank as u64;
    }

    fn rank_straight(&mut self, hand:Vec<card::Card>) -> u64 {
        let mut rank = 0;
        if hand[4].clone().get_rank() == Rank::Ace && hand[0].clone().get_rank() == Rank::Two {
            rank = 1;
        } else {
            rank = hand[2].clone().get_rank() as u8;
        }
        return rank as u64;
    }

    fn evaluate_hand(&mut self, hand: Vec<card::Card>) -> Vec<card::Card> {
        let mut cards = self.board.clone();
        cards.append(&mut hand.clone());
        cards.sort_by_key(|c| c.clone().get_suit());
        cards.sort_by_key(|c| c.clone().get_rank());
        let it = cards.into_iter().combinations(5);

        let mut best_hand_rank = HandRanking::HighCard;
        let mut best_hands = Vec::new();

        for i in it {
            let r  = self.rank_five_card_combo(i.clone());
            if (r.clone() as u8) > (best_hand_rank.clone() as u8) {
                best_hand_rank = r.clone();
                best_hands.clear();
                best_hands.push(i.clone());
            } else if (r.clone() as u8) == (best_hand_rank.clone() as u8) {
                best_hands.push(i.clone());
            }
        }

        if (best_hand_rank == HandRanking::HighCard) || best_hand_rank == HandRanking::Pair || best_hand_rank == HandRanking::TwoPair || best_hand_rank == HandRanking::ThreeOfAKind || best_hand_rank == HandRanking::Flush || best_hand_rank == HandRanking::FullHouse || best_hand_rank == HandRanking::FourOfAKind {
            let best_hand = best_hands[best_hands.len()-1].clone();
            return best_hand.clone();
        } else if best_hand_rank == HandRanking::Straight || best_hand_rank == HandRanking::StraightFlush {
            let best_hand = self.best_straight_card_combo(best_hands.clone());
            return best_hand.clone();
        }
        return Vec::new();

    }

    fn rank_five_card_combo(&mut self, mut cards: Vec<card::Card>) -> HandRanking {
        cards.sort_by_key(|c| c.clone().get_suit());
        cards.sort_by_key(|c| c.clone().get_rank());

        if self.is_straight_flush(cards.clone()) {
            return HandRanking::StraightFlush;
        } else if self.is_four_of_a_kind(cards.clone()) {
            return HandRanking::FourOfAKind;
        } else if self.is_full_house(cards.clone()) {
            return HandRanking::FullHouse;
        } else if self.is_flush(cards.clone()) {
            return HandRanking::Flush;
        } else if self.is_straight(cards.clone()) {
            return HandRanking::Straight
        } else if self.is_three_of_a_kind(cards.clone()) {
            return HandRanking::ThreeOfAKind
        } else if self.is_two_pair(cards.clone()) {
            return HandRanking::TwoPair
        } else if self.is_pair(cards.clone()) {
            return HandRanking::Pair
        }
        return HandRanking::HighCard
    }

    fn best_straight_card_combo(&mut self, mut combos: Vec<Vec<card::Card>>) -> Vec<card::Card> {
        if combos.len() > 1 {
            let mut highest_rank = 0;
            let mut winning_combo = Vec::new();
            for combo in combos.clone() {
                if combo[2].clone().get_rank() as u8 > highest_rank {
                    highest_rank = combo[2].clone().get_rank() as u8;
                    winning_combo.clear();
                    winning_combo = combo.clone();
                }
            }
            return winning_combo.clone();
        }
        return combos[0].clone();
    }

    fn is_pair(&mut self, cards: Vec<card::Card>) -> bool {
        if cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8 {
            return true;
        } else if cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8 {
            return true;
        } else if cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8 {
            return true;
        } else if cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8 {
            return true;
        }
        return false;
    }

    fn is_two_pair(&mut self, cards: Vec<card::Card>) -> bool {
        if (cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8) &&
            (cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8) {
            return true;
        } else if (cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8) &&
            (cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8) {
            return true;
        } else if (cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8) &&
            (cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8) {
            return true;
        }
        return false;
    }

    fn is_three_of_a_kind(&mut self, cards: Vec<card::Card>) -> bool {
        if (cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8) &&
            (cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8) {
            return true;
        } else if (cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8) &&
            (cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8) {
            return true;
        } else if (cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8) &&
            (cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8) {
            return true;
        }
        return false;
    }

    fn is_straight(&mut self, cards: Vec<card::Card>) -> bool {
        if cards[4].clone().get_rank() == Rank::Ace && cards[0].clone().get_rank() == Rank::Two {
            for i in 0..3 {
                if (cards[i].clone().get_rank() as u8 + 1 as u8) != (cards[i+1].clone().get_rank() as u8) {
                    return false;
                }
            }
            return true;
        }

        for i in 0..4 {
            if (cards[i].clone().get_rank() as u8 + 1 as u8) != (cards[i+1].clone().get_rank() as u8) {
                return false;
            }
        }
        return true;
    }

    fn is_flush(&mut self, cards: Vec<card::Card>) -> bool {
        let first_suit = cards[0].clone().get_suit();
        for c in cards.clone() {
            if c.clone().get_suit() != first_suit {
                return false;
            }
        }
        return true;
    }

    fn is_full_house(&mut self, cards: Vec<card::Card>) -> bool {
        if (cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8) &&
            (cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8) &&
            (cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8) {
            return true;
        } else if (cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8) &&
            (cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8) &&
            (cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8) {
            return true;
        }
        return false;
    }

    fn is_four_of_a_kind(&mut self, cards: Vec<card::Card>) -> bool {
        if (cards[0].clone().get_rank() as u8 == cards[1].clone().get_rank() as u8) &&
           (cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8) &&
           (cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8) {
            return true;
        } else if (cards[1].clone().get_rank() as u8 == cards[2].clone().get_rank() as u8) &&
           (cards[2].clone().get_rank() as u8 == cards[3].clone().get_rank() as u8) &&
           (cards[3].clone().get_rank() as u8 == cards[4].clone().get_rank() as u8) {
            return true;
        }
        return false;
    }

    fn is_straight_flush(&mut self, cards: Vec<card::Card>) -> bool {
        if self.is_straight(cards.clone()) && self.is_flush(cards.clone()) {
            return true;
        }
        return false;
    }

    fn reset_current_bet(&mut self) {
        for mut p in self.players.iter_mut() {
            p.current_bet = 0;
        }
    }

    fn progress_street(&mut self) {
        for p in self.players.iter() {
            if !p.final_action {
                return;
            }
        }
        if self.street == GameStreet::PRE {
            self.street = GameStreet::FLOP;
            self.deck.pop();
            self.board.push(self.deck.pop().unwrap());
            self.board.push(self.deck.pop().unwrap());
            self.board.push(self.deck.pop().unwrap());
            self.turn_marker = self.button+1
        } else if self.street == GameStreet::FLOP {
            self.street = GameStreet::TURN;
            self.deck.pop();
            self.board.push(self.deck.pop().unwrap());
            self.turn_marker = self.button+1
        } else if self.street == GameStreet::TURN {
            self.street = GameStreet::RIVER;
            self.deck.pop();
            self.board.push(self.deck.pop().unwrap());
            self.turn_marker = self.button+1
        } else if self.street == GameStreet::RIVER {
            self.street = GameStreet::SHOWDOWN;
        } else if self.street == GameStreet::SHOWDOWN {
            return;
        }
        self.reset_final_action();
        self.reset_current_bet();
        self.previous_raise = self.big_blind;
        self.turn_marker = self.button+1;
        self.previous_bet = 0;
        self.current_bet = 0;
        self.previous_raise = 0;

    }

    fn place_blind(&mut self, mut bet: u64) {
        if self.players[self.turn_marker as usize].chip_stack < bet {
            bet = self.players[self.turn_marker as usize].chip_stack;
        }
        self.players[self.turn_marker as usize].chip_stack -= bet;
        self.players[self.turn_marker as usize].current_bet = bet;
        self.current_bet = bet;
        self.pot += bet;
        self.previous_raise = self.big_blind;
        self.previous_bet = bet;
        let forced_blind = Action{
            action: ActionType::BLIND,
            player: self.players[self.turn_marker as usize].clone(),
            bet_size: bet,
            street: self.street.clone()
        };
        self.actions.push(forced_blind);
        self.increment_turn();
    }


    fn force_blinds(&mut self) {
        self.place_blind(self.big_blind/2);
        self.place_blind(self.big_blind);
    }

    fn increment_turn(&mut self) {
        if self.turn_marker < (self.players.len() - 1) as u64 {
            self.turn_marker += 1;
        } else {
            self.turn_marker = 0;
        }
    }

    fn decrement_turn(&mut self) {
        if self.turn_marker < 1 {
            self.turn_marker = (self.players.len() - 1) as u64;
        } else {
            self.turn_marker -= 1;
        }
    }

    fn check(&mut self, name:String) -> Result<(), InvalidActionError> {
        if self.players[self.turn_marker as usize].name != name {
            return Err(InvalidActionError)
        }
        if self.players[self.turn_marker as usize].has_folded || self.players[self.turn_marker as usize].final_action  {
            return Err(InvalidActionError)
        }
        if self.players[self.turn_marker as usize].current_bet != self.current_bet {
            return Err(InvalidActionError)
        }
        self.players[self.turn_marker as usize].final_action = true;
        let action = Action{
            action: ActionType::CHECK,
            player: self.players[self.turn_marker as usize].clone(),
            bet_size: 0,
            street: self.street.clone()
        };
        self.actions.push(action);
        self.increment_turn();
        Ok(())
    }


    fn call(&mut self, name:String) -> Result<(), InvalidActionError> {
        if self.players[self.turn_marker as usize].name != name {
            return Err(InvalidActionError)
        }
        if self.players[self.turn_marker as usize].has_folded || self.players[self.turn_marker as usize].final_action  {
            return Err(InvalidActionError)
        }
        if self.players[self.turn_marker as usize].current_bet >= self.current_bet {
            return Err(InvalidActionError)
        }
        let bet = self.current_bet - self.players[self.turn_marker as usize].current_bet;
        self.players[self.turn_marker as usize].chip_stack -= bet;
        self.players[self.turn_marker as usize].current_bet += bet;
        self.pot += bet;
        self.previous_bet = bet;
        self.players[self.turn_marker as usize].final_action = true;

        let action = Action{
            action: ActionType::CALL,
            player: self.players[self.turn_marker as usize].clone(),
            bet_size: bet,
            street: self.street.clone()
        };
        self.actions.push(action);
        self.increment_turn();
        Ok(())
    }

    fn fold(&mut self, name:String) -> Result<(), InvalidActionError> {
        if self.players[self.turn_marker as usize].name != name {
            return Err(InvalidActionError)
        }
        if self.players[self.turn_marker as usize].has_folded || self.players[self.turn_marker as usize].final_action  {
            return Err(InvalidActionError)
        }
        self.players[self.turn_marker as usize].has_folded = true;
        self.players[self.turn_marker as usize].final_action = true;
        let action = Action{
            action: ActionType::FOLD,
            player: self.players[self.turn_marker as usize].clone(),
            bet_size: 0,
            street: self.street.clone()
        };
        self.actions.push(action);
        self.increment_turn();
        Ok(())
    }

    fn reset_final_action(&mut self) {
        for mut p in self.players.iter_mut() {
            p.final_action = false;
        }
    }


    fn raise(&mut self, name:String, bet:u64) -> Result<(), InvalidActionError> {
        if self.players[self.turn_marker as usize].name != name {
            return Err(InvalidActionError)
        }
        if self.players[self.turn_marker as usize].has_folded || self.players[self.turn_marker as usize].final_action  {
            return Err(InvalidActionError)
        }
        if bet == self.players[self.turn_marker as usize].chip_stack {
            self.current_bet = bet + self.players[self.turn_marker as usize].current_bet;
            self.pot += bet;
            self.players[self.turn_marker as usize].chip_stack -= bet;
            if bet > self.previous_bet {
                if bet - self.previous_bet > self.previous_raise {
                    self.previous_raise = bet - self.previous_bet;
                }
            }
            self.previous_bet = bet;
            self.players[self.turn_marker as usize].current_bet += bet;
            self.reset_final_action();
            self.players[self.turn_marker as usize].final_action = true;
            let action = Action{
                action: ActionType::RAISE,
                player: self.players[self.turn_marker as usize].clone(),
                bet_size: bet,
                street: self.street.clone()
            };
            self.actions.push(action);
            self.increment_turn();

            return Ok(())
        }
        if bet < self.big_blind {
            return Err(InvalidActionError)
        }
        if bet < self.previous_raise + self.current_bet || bet > self.players[self.turn_marker as usize].chip_stack {
            return Err(InvalidActionError)
        }

        self.current_bet = bet + self.players[self.turn_marker as usize].current_bet; // THIS IS WRONG??? why does player have current bet, does this need to be reset as well??
        self.pot += bet;
        self.players[self.turn_marker as usize].chip_stack -= bet;
        self.players[self.turn_marker as usize].chip_stack -= bet;
        self.previous_raise = bet - self.previous_bet;
        self.previous_bet = bet;
        self.players[self.turn_marker as usize].current_bet += bet;
        self.reset_final_action();
        self.players[self.turn_marker as usize].final_action = true;
        let action = Action{
            action: ActionType::RAISE,
            player: self.players[self.turn_marker as usize].clone(),
            bet_size: bet,
            street: self.street.clone()
        };
        self.actions.push(action);
        self.increment_turn();

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use crate::{card, card::Card, Game, Player};
    use crate::card::{Rank, Suit};

    #[test]
    fn init_game() {
        let mut g = Game::new(5000, 100);
        g.add_player(String::from("Alice"));
        g.add_player(String::from("Bob"));
        g.add_player(String::from("Charlie"));
        g.add_player(String::from("Dave"));
        g.add_player(String::from("Fred"));
        g.add_player(String::from("George"));

        g.init_deck();

        // Place blinds
        g.force_blinds();

        g.deal_hole_cards();

        let mut ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        g.progress_street();
        // dbg!(&g);

        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        g.progress_street();

        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        g.progress_street();

        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        g.progress_street();

        // dbg!(&g);
        dbg!("find winners")

        g.find_winner();

        dbg!(&g);
        dbg!(&g.winners.len());

        // g.evaluate_hand(g.players[0].hole_cards.clone());
        // ans = g.raise(g.players[g.turn_marker as usize].clone().name, 100);
        // dbg!(&g);
        // dbg!(ans);

        // Start Check Tests
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.check(g.players[g.turn_marker as usize].clone());
        // dbg!(&g);
        // dbg!(ans);
        // End Check Tests



        // Start Fold Tests
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // ans = g.fold(g.players[g.turn_marker as usize].clone());
        // dbg!(&g);
        // dbg!(ans);
        // End Fold Tests


        // Start Call Tests
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // dbg!(ans);
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // dbg!(ans);
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // dbg!(ans);
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // dbg!(ans);
        // // dbg!(&g);
        // End Call Tests




        // Start Raise Tests
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 500);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 1000);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 2000);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 4000);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 4900);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 4800);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 4500);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 4000);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 3000);
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 950);
        // dbg!(&g);
        // End Raise Tests


        // let c1 = card::Card::new(Rank::Ace, Suit::Club);
        // let c2 = card::Card::new(Rank::King, Suit::Club);
        // let c3 = card::Card::new(Rank::Queen, Suit::Club);
        // let c4 = card::Card::new(Rank::Jack, Suit::Club);
        // let c5 = card::Card::new(Rank::Ten, Suit::Club);
        //
        // let c6 = card::Card::new(Rank::Ace, Suit::Heart);
        // let c7 = card::Card::new(Rank::Ace, Suit::Diamond);
        // let c8 = card::Card::new(Rank::Ace, Suit::Spade);
        //
        // let c9 = card::Card::new(Rank::King, Suit::Heart);
        //
        // let c10 = card::Card::new(Rank::Nine, Suit::Club);
        //
        // let c11 = card::Card::new(Rank::Two, Suit::Heart);
        // let c12 = card::Card::new(Rank::Three, Suit::Diamond);
        // let c13 = card::Card::new(Rank::Four, Suit::Spade);
        // let c14 = card::Card::new(Rank::Five, Suit::Club);
        //
        // let c15 = card::Card::new(Rank::Two, Suit::Club);
        // let c16 = card::Card::new(Rank::Three, Suit::Club);
        // let c17 = card::Card::new(Rank::Four, Suit::Club);
        //
        // let c18 = card::Card::new(Rank::Six, Suit::Club);
        // let c19 = card::Card::new(Rank::Seven, Suit::Club);

        // let c20 = card::Card::new(Rank::Two, Suit::Club);
        // let c21 = card::Card::new(Rank::Two, Suit::Diamond);
        // let c22 = card::Card::new(Rank::Two, Suit::Spade);
        //
        // let c23 = card::Card::new(Rank::Eight, Suit::Heart);
        // let c24 = card::Card::new(Rank::Eight, Suit::Spade);
        //
        // let c25 = card::Card::new(Rank::Ace, Suit::Spade);
        // let c26 = card::Card::new(Rank::King, Suit::Heart);
        //
        // let mut board2 = Vec::new();
        // let mut hb2 = Vec::new();
        //
        // board2.push(c20);
        // board2.push(c21);
        // board2.push(c22);
        // board2.push(c23);
        // board2.push(c24);
        //
        // hb2.push(c25);
        // hb2.push(c26);
        //
        // g.board = board2;
        //
        // g.evaluate_hand(hb2);







        // let mut board1 = Vec::new();
        // let mut hb1 = Vec::new();
        //
        // board1.push(c1);
        // board1.push(c19);
        // board1.push(c14);
        // board1.push(c15);
        // board1.push(c16);
        //
        // hb1.push(c17);
        // hb1.push(c18);
        //
        // g.board = board1;
        //
        // g.evaluate_hand(hb1);

        //
        //
        // // straight flush
        // let mut hand1 = Vec::new();
        // hand1.push(c1.clone());
        // hand1.push(c3.clone());
        // hand1.push(c5.clone());
        // hand1.push(c4.clone());
        // hand1.push(c2.clone());
        //
        // let rank1 = g.rank_five_card_combo(hand1.clone());
        // dbg!(rank1);
        //
        // // quads
        // let mut hand2 = Vec::new();
        // hand2.push(c1.clone());
        // hand2.push(c2.clone());
        // hand2.push(c6.clone());
        // hand2.push(c7.clone());
        // hand2.push(c8.clone());
        //
        // let rank2 = g.rank_five_card_combo(hand2.clone());
        // dbg!(rank2);
        //
        // // boat
        // let mut hand3 = Vec::new();
        // hand3.push(c1.clone());
        // hand3.push(c2.clone());
        // hand3.push(c6.clone());
        // hand3.push(c7.clone());
        // hand3.push(c9.clone());
        //
        // let rank3 = g.rank_five_card_combo(hand3.clone());
        // dbg!(rank3);
        //
        // // flush
        // let mut hand4 = Vec::new();
        // hand4.push(c1.clone());
        // hand4.push(c3.clone());
        // hand4.push(c4.clone());
        // hand4.push(c5.clone());
        // hand4.push(c10.clone());
        //
        // let rank4 = g.rank_five_card_combo(hand4.clone());
        // dbg!(rank4);
        //
        // // straight
        // let mut hand5 = Vec::new();
        // hand5.push(c1.clone());
        // hand5.push(c3.clone());
        // hand5.push(c4.clone());
        // hand5.push(c5.clone());
        // hand5.push(c9.clone());
        //
        // let rank5 = g.rank_five_card_combo(hand5.clone());
        // dbg!(rank5);
        //
        // // trips
        // let mut hand6 = Vec::new();
        // hand6.push(c6.clone());
        // hand6.push(c8.clone());
        // hand6.push(c7.clone());
        // hand6.push(c2.clone());
        // hand6.push(c10.clone());
        //
        // let rank6 = g.rank_five_card_combo(hand6.clone());
        // dbg!(rank6);
        //
        // // two pair
        // let mut hand7 = Vec::new();
        // hand7.push(c6.clone());
        // hand7.push(c8.clone());
        // hand7.push(c5.clone());
        // hand7.push(c2.clone());
        // hand7.push(c9.clone());
        //
        // let rank7 = g.rank_five_card_combo(hand7.clone());
        // dbg!(rank7);
        //
        // // pair
        // let mut hand8 = Vec::new();
        // hand8.push(c10.clone());
        // hand8.push(c8.clone());
        // hand8.push(c5.clone());
        // hand8.push(c2.clone());
        // hand8.push(c9.clone());
        //
        // let rank8 = g.rank_five_card_combo(hand8.clone());
        // dbg!(rank8);
        //
        // // Ahigh straight
        // let mut hand9 = Vec::new();
        // hand9.push(c13.clone());
        // hand9.push(c11.clone());
        // hand9.push(c14.clone());
        // hand9.push(c1.clone());
        // hand9.push(c12.clone());
        //
        // let rank9 = g.rank_five_card_combo(hand9.clone());
        // dbg!(rank9);
        //
        // // a high straight flush
        // let mut hand10 = Vec::new();
        // hand10.push(c17.clone());
        // hand10.push(c16.clone());
        // hand10.push(c14.clone());
        // hand10.push(c1.clone());
        // hand10.push(c15.clone());
        //
        // let rank10 = g.rank_five_card_combo(hand10.clone());
        // dbg!(rank10);


    }
}


