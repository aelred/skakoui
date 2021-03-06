//! Check count of number of moves up to a certain depth
//! https://www.chessprogramming.org/Perft_Results#Position_2

use skakoui::{Board, Move};
use std::convert::TryInto;
use std::fmt;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn number_of_moves_is_correct_for_initial_position(c: &mut Criterion) {
    run_perft(
        "perft_init",
        c,
        Board::default(),
        vec![20, 400, 8902, 197_281, 4_865_609],
    );
}

/// This position allows moves like castling and en-passant
/// https://www.chessprogramming.org/Perft_Results#Position_2
fn number_of_moves_is_correct_for_kiwipete_position(c: &mut Criterion) {
    run_perft(
        "perft_kiwipete",
        c,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        vec![48, 2039, 97_862, 4_085_603],
    );
}

criterion_group!(
    perft,
    number_of_moves_is_correct_for_initial_position,
    number_of_moves_is_correct_for_kiwipete_position
);
criterion_main!(perft);

fn run_perft(
    name: &'static str,
    c: &mut Criterion,
    board: impl TryInto<Board, Error = impl fmt::Debug>,
    expected_moves_at_depth: Vec<usize>,
) {
    let mut board = board.try_into().unwrap();

    for (depth, expected_moves) in expected_moves_at_depth.iter().enumerate() {
        let depth = depth + 1;
        c.bench_with_input(BenchmarkId::new(name, depth), &depth, |b, depth| {
            b.iter(|| {
                expect_moves(&mut board, *depth, *expected_moves);
            });
        });
    }
}

fn expect_moves(board: &mut Board, depth: usize, expected_moves: usize) {
    let actual_moves = board.perft(depth);
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
                let num_moves = board.perft(depth - 1);
                move_counts.push_str(&format!("{}: {}\n", mov, num_moves));
                board.unmake_move(pmov);
            }
            move_counts
        },
        board
    );
}
