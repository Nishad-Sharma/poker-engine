use pokerengine::Game;

fn main() {
    println!("Hello from an example!");
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

    // preflop
    g.call(String::from("Alice"));
    g.call(String::from("Bob"));
    g.check(String::from("Charlie"));
    // flop
    g.check(String::from("Bob"));
    g.check(String::from("Charlie"));
    g.check(String::from("Alice"));
    // turn
    g.check(String::from("Bob"));
    g.check(String::from("Charlie"));
    g.check(String::from("Alice"));
    // river
    g.check(String::from("Bob"));
    g.check(String::from("Charlie"));
    g.check(String::from("Alice"));
    //showdown
    g.find_winner();
    g.payout_winners();
    g.prep_next_hand();
}
