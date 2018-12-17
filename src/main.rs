#![cfg_attr(feature = "strict", deny(warnings))]

use chess::search;
use chess::Board;
use chess::Move;
use chess::Player;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::io::Write;

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock().lines();

    let mut board = Board::default();
    let mut searcher = search::Searcher::default();
    println!("{}", board);

    loop {
        let valid_moves: HashSet<Move> = board.moves().collect();

        let player_move = loop {
            print!("Player: ");
            io::stdout().flush().expect("Could not flush stdout");

            let line = input.next().unwrap().unwrap();
            if let Ok(mov) = line.parse() {
                if valid_moves.contains(&mov) {
                    break mov;
                }
            }
        };

        board.make_move(player_move);
        println!("\n{}", board);

        let computer_move = decide(&mut searcher, &board).unwrap();
        println!("Computer: {}", computer_move);
        board.make_move(computer_move);
        println!("\n{}", board);
    }
}

fn decide(searcher: &mut search::Searcher<Move, Board>, board: &Board) -> Option<Move> {
    let maximising_player = board.player() == Player::White;

    let (mov, _) = searcher.run(board, 4, maximising_player);
    mov
}
