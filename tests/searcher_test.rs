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

    for (mut board, mating_move) in mate_in_1s().into_iter() {
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

fn mate_in_1s() -> Vec<(Board, Move)> {
    mate_in_2s()
        .into_iter()
        .map(|(mut board, moves)| {
            board.make_move(moves[0]);
            board.make_move(moves[1]);
            (board, moves[2])
        })
        .collect()
}

fn mate_in_2s() -> Vec<(Board, Vec<Move>)> {
    let m8n2 = include_str!("m8n2.txt");
    let mut lines = m8n2.lines().peekable();

    let mut mates = vec![];

    'mates: while lines.peek().is_some() {
        // Try to parse each line as a board position
        let board = lines.next().and_then(|line| Board::from_fen(line).ok());

        if let Some(mut board) = board {
            if let Some(mstr) = lines.next() {
                let mstr = mstr.replace("1.", "").replace("2.", "").replace(".", "");

                let mut mstr = mstr.split_whitespace();

                let amov1 = mstr.next().unwrap().parse::<Algebraic>().unwrap();
                let amov2 = mstr.next().unwrap().parse::<Algebraic>().unwrap();
                let amov3 = mstr.next().unwrap().parse::<Algebraic>().unwrap();

                let mut moves: Vec<Move> = vec![];
                let mut undo: Vec<PlayedMove> = vec![];

                for amov in [amov1, amov2, amov3].iter() {
                    if let Some(mov) = amov.to_move(&mut board) {
                        undo.push(board.make_move(mov));
                        moves.push(mov);
                    } else {
                        eprintln!("Couldn't find {} on board:\n{}\n{:?}", amov, board, board);
                        continue 'mates;
                    }
                }

                for undo in undo.into_iter().rev() {
                    board.unmake_move(undo);
                }

                mates.push((board, moves));
            }
        }
    }

    mates
}
