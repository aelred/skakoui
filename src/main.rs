use chess::Board;
use rand::seq::IteratorRandom;

fn main() {
    let mut rng = rand::thread_rng();

    let mut board = Board::default();
    println!("{}", board);

    while let Some(mov) = board.moves().choose(&mut rng) {
        board.make_move(mov);
        println!("\n{}", board);
    }
}
