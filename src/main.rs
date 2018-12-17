use chess::search;
use chess::Board;
use chess::Move;
use chess::Player;
use std::fs::File;

fn main() {
    let mut board = Board::default();
    let mut searcher = search::Searcher::new();
    println!("{}", board);

    while let Some(mov) = decide(&mut searcher, &board) {
        board.make_move(mov);
        println!("\n{}", board);
    }
}

fn decide(searcher: &mut search::Searcher<Move, Board>, board: &Board) -> Option<Move> {
    let maximising_player = board.player() == Player::White;

    let (mov, value) = searcher.run(board, 4, maximising_player);
    mov
}
