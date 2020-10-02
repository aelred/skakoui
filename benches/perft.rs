//! Check count of number of moves up to a certain depth
//! https://www.chessprogramming.org/Perft_Results#Position_2

use skakoui::{Board, Move};
use std::convert::TryInto;
use std::fmt;

#[ignore]
#[test]
fn number_of_moves_is_correct_for_initial_position() {
    perft(
        Board::default(),
        vec![
            20, 400, 8902, 197_281, // 4_865_609 - TODO: fails
        ],
    );
}

/// This position allows moves like castling and en-passant
/// https://www.chessprogramming.org/Perft_Results#Position_2
#[test]
fn number_of_moves_is_correct_for_kiwipete_position() {
    perft(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        vec![48],
    );
}

fn perft(board: impl TryInto<Board, Error = impl fmt::Debug>, expected_moves_at_depth: Vec<usize>) {
    let mut board = board.try_into().unwrap();

    for (depth, expected_moves) in expected_moves_at_depth.iter().enumerate() {
        expect_moves(&mut board, depth + 1, *expected_moves);
    }
}

fn expect_moves(board: &mut Board, depth: usize, expected_moves: usize) {
    let actual_moves = count_moves(board, depth);
    assert_eq!(
        expected_moves,
        actual_moves,
        "Expected {} moves, but counted {}\n{}\non board:\n{:?}",
        expected_moves,
        actual_moves,
        {
            let moves: Vec<Move> = board.moves().collect();
            let mut move_counts = String::new();
            for mov in moves {
                let pmov = board.make_move(mov);
                let num_moves = count_moves(board, depth - 1);
                move_counts.push_str(&format!("{}: {}\n", mov, num_moves));
                board.unmake_move(pmov);
            }
            move_counts
        },
        board
    );
}

fn count_moves(board: &mut Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut count = 0;

    let moves: Vec<Move> = board.moves().collect();

    // Optimisation - skip making and un-making last moves
    if depth == 1 {
        return moves.len();
    }

    for mov in moves {
        let pmov = board.make_move(mov);
        count += count_moves(board, depth - 1);
        board.unmake_move(pmov);
    }

    count
}
