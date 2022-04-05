use crate::card;
use crate::lib;

fn main() {
    let mut g = Game::new(5000, 100);
    g.add_player(String::from("Alice"));
    g.add_player(String::from("Bob"));
    g.add_player(String::from("Charlie"));

    // initialise deck order
    g.init_deck();
    // place blinds
    g.force_blinds();
    // give each player 2 cards
    g.deal_hole_cards();
}