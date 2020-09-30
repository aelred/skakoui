#![cfg(feature = "expensive_tests")]

pub mod strategies;

use skakoui::{Board, File, Move, Piece, PieceType, PlayedMove, Player, Rank, Searcher, Square};

use anyhow::anyhow;
use anyhow::{Context, Error};
use lazy_static::lazy_static;
use regex::{Match, Regex};
use serde::export::Formatter;
use skakoui::pgn::Algebraic;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[test]
fn searcher_can_find_mate_in_1() {
    let mut searcher = Searcher::default();

    // Searcher is still too slow to try them all
    const LIMIT: usize = 10;

    for (mut board, mating_move) in mate_in_1s().into_iter().take(LIMIT) {
        println!(
            "Testing board\n{}\nExpect: {}\n{:?}",
            board, mating_move, board
        );
        searcher.go(&board, Some(2));
        searcher.wait();
        let pv = searcher.principal_variation();
        let mov = *pv.first().unwrap();

        let pmov = board.make_move(mov);
        let checkmate = board.checkmate();
        board.unmake_move(pmov);

        assert!(
            checkmate,
            "{}\nExpect: {}\nActual: {}\nPV: {:?}",
            board, mating_move, mov, pv
        );
    }
}

fn mate_in_1s() -> impl Iterator<Item = (Board, Move)> {
    mate_in_2s().into_iter().map(|(mut board, moves)| {
        board.make_move(moves[0]);
        board.make_move(moves[1]);
        (board, moves[2])
    })
}

fn mate_in_2s() -> impl Iterator<Item = (Board, Vec<Move>)> {
    let m8n2 = include_str!("m8n2.txt");
    let mut lines = m8n2.lines().peekable();

    std::iter::from_fn(move || {
        // Try to parse each line as a board position
        let mut board = lines.find_map(|line| Board::from_fen(line).ok())?;

        let mstr = lines.next()?;
        let mstr = mstr.replace("1.", "").replace("2.", "").replace(".", "");

        Some((board, mstr))
    })
    .filter_map(|(mut board, mstr)| {
        let mstr = mstr.split_whitespace();
        let amoves = mstr.map(|m| m.parse::<Algebraic>().unwrap());

        let mut moves: Vec<Move> = vec![];
        let mut undo: Vec<PlayedMove> = vec![];

        for amov in amoves {
            if let Some(mov) = amov.to_move(&mut board) {
                undo.push(board.make_move(mov));
                moves.push(mov);
            } else {
                eprintln!("Couldn't find {} on board:\n{}\n{:?}", amov, board, board);
                return None;
            }
        }

        for undo in undo.into_iter().rev() {
            board.unmake_move(undo);
        }

        Some((board, moves))
    })
}
