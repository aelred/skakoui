#![cfg_attr(feature = "strict", deny(warnings))]

use chess::search::Searcher;
use chess::Board;
use chess::Move;
use chess::Player;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::io::Lines;
use std::io::Write;

fn main() {
    let stdin = io::stdin();
    let lock = stdin.lock();

    let mut board = Board::default();
    println!("{}", board);

    let mut white = Computer::default();
    let mut black = Human::new(lock);

    loop {
        play(&mut white, &mut board);
        play(&mut black, &mut board);
    }
}

fn play<A: Agent>(agent: &mut A, board: &mut Board) {
    println!();
    print!("{}: ", A::NAME);
    io::stdout().flush().expect("Could not flush stdout");
    if let Some(mov) = agent.get_move(board) {
        println!("{}", mov);
        board.make_move(mov);
        println!();
        println!("{}", board);
    } else {
        println!("Game Over!");
        panic!(); // TODO: don't just panic on game over ¯\_(ツ)_/¯
    }
}

trait Agent {
    const NAME: &'static str;

    fn get_move(&mut self, board: &mut Board) -> Option<Move>;
}

#[derive(Default)]
struct Computer {
    searcher: Searcher<Board>,
}

impl Agent for Computer {
    const NAME: &'static str = "Computer";

    fn get_move(&mut self, board: &mut Board) -> Option<Move> {
        let maximising_player = board.player() == Player::White;
        let (mov, _) = self.searcher.run(board, 5, maximising_player);
        mov
    }
}

struct Human<B> {
    input: Lines<B>,
}

impl<B: BufRead> Human<B> {
    fn new(input: B) -> Self {
        Self {
            input: input.lines(),
        }
    }
}

impl<B: BufRead> Agent for Human<B> {
    const NAME: &'static str = "Player";

    fn get_move(&mut self, board: &mut Board) -> Option<Move> {
        let valid_moves: HashSet<Move> = board.moves().collect();

        loop {
            let line = self.input.next().unwrap().unwrap();
            if let Ok(mov) = line.parse() {
                if valid_moves.contains(&mov) {
                    return Some(mov);
                }
            }
        }
    }
}
