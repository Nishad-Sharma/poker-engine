use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(Eq,PartialEq,Debug,Clone)]
struct Player {
    pub name: String,
    chip_stack: u64,
    current_bet: u64,
    has_folded: bool,
    final_action: bool,
    hole_cards: Vec<Card>
}

impl Player {
    fn new(name:String, chip_stack:u64) -> Player {
        Player{name, chip_stack, current_bet: 0, has_folded: false, final_action: false, hole_cards: Vec::with_capacity(2)}
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

#[derive(Debug, Eq, PartialEq, Clone, EnumIter)]
enum Card {
    HeartA,
    Heart2,
    Heart3,
    Heart4,
    Heart5,
    Heart6,
    Heart7,
    Heart8,
    Heart9,
    HeartT,
    HeartJ,
    HeartQ,
    HeartK,
    SpadeA,
    Spade2,
    Spade3,
    Spade4,
    Spade5,
    Spade6,
    Spade7,
    Spade8,
    Spade9,
    SpadeT,
    SpadeJ,
    SpadeQ,
    SpadeK,
    DiamondA,
    Diamond2,
    Diamond3,
    Diamond4,
    Diamond5,
    Diamond6,
    Diamond7,
    Diamond8,
    Diamond9,
    DiamondT,
    DiamondJ,
    DiamondQ,
    DiamondK,
    ClubA,
    Club2,
    Club3,
    Club4,
    Club5,
    Club6,
    Club7,
    Club8,
    Club9,
    ClubT,
    ClubJ,
    ClubQ,
    ClubK
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
    pub deck: Vec<Card>,
    pub board: Vec<Card>,
}

impl Game {
    fn new(start_stack:u64, big_blind:u64) -> Game {
        Game{ players: Vec::with_capacity(9), start_stack, button:0, actions: Vec::new(), big_blind, pot: 0, previous_raise: 0, previous_bet: 0, current_bet: 0, turn_marker: 1, street: GameStreet::PRE, deck: Vec::new(), board: Vec::with_capacity(5) }
    }

    fn add_player(&mut self, name:String) {
        self.players.push(Player::new(name, self.start_stack));
    }

    fn deal_hole_cards(&mut self) {
        let original_turn_marker = self.turn_marker;

        self.turn_marker = self.button+1;
        for i in (1..3).step_by(1) {
            // dbg!(i);
            for j in (1..self.players.len()+1).step_by(1) {
                // dbg!(j);
                // dbg!(&self.players[self.turn_marker as usize].name);
                // dbg!(&self.turn_marker);
                self.increment_turn();
                self.players[self.turn_marker as usize].hole_cards.push(self.deck.pop().unwrap());
            }
        }
        self.turn_marker = original_turn_marker;
    }

    fn init_deck(&mut self) {
        for c in Card::iter(){
            self.deck.push(c);
        }
        self.deck.shuffle(&mut thread_rng());
    }

    // fn find_winner(&mut self) {
    //     let mut possible_winners = Vec::new();
    //     for p in self.players.iter(){
    //         if !(p.has_folded) {
    //             possible_winners.push(p.clone(0));
    //         }
    //     }
    //     let mut
    // }

    // fn evaluate_hand(&mut self, hand: Vec<Card>) {
    //     let mut cards = self.board.clone();
    //     cards.append(&mut hand.clone());
    // }

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
        // otherwise progress street
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
            return; // ERROR NO STREET AFTER SHOWDONW?
        }
        self.reset_final_action();
        self.reset_current_bet();
        self.previous_raise = self.big_blind;
        self.turn_marker = self.button+1;
        self.previous_bet = 0;
        self.current_bet = 0;
        self.previous_raise = 0;

    }

    // need a check for all ins->progress to showdown?
    // fn showdown_time(&mut self) {
    //
    // }

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
    use crate::{Card, Game, Player};

    // #[test]
    // fn it_works() {
    //     let result = 2 + 2;
    //     assert_eq!(result, 4);
    // }
    #[test]
    fn init_game() {
        let mut g = Game::new(5000, 100);
        g.add_player(String::from("Alice"));
        g.add_player(String::from("Bob"));
        g.add_player(String::from("Charlie"));
        g.add_player(String::from("Dave"));
        g.add_player(String::from("Fred"));
        g.add_player(String::from("George"));

        // let c = Card::new()
        g.init_deck();

        // let p = *g.players.get(&Player { name: "Alice".to_string() }).unwrap();
        // Place blinds
        g.force_blinds();

        g.deal_hole_cards();
        // g.progress_street();

        dbg!(&g);

        // let mut ans = g.raise(g.players[g.turn_marker as usize].clone(), 200);
        // dbg!(ans);
        // ans = g.call(g.players[g.turn_marker as usize].clone());
        // ans = g.raise(g.players[g.turn_marker as usize].clone(), 400);
        let mut ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        ans = g.call(g.players[g.turn_marker as usize].clone().name);
        // // ans = g.call(g.players[g.turn_marker as usize].clone());
        ans = g.check(g.players[g.turn_marker as usize].clone().name);
        g.progress_street();
        dbg!(&g);

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

        dbg!(&g);

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


    }
}


