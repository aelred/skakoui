#![cfg(feature = "expensive_tests")]

pub mod strategies;

use skakoui::{Board, File, Move, Piece, PieceType, PlayedMove, Player, Rank, Searcher, Square};

use anyhow::anyhow;
use anyhow::{Context, Error};
use lazy_static::lazy_static;
use regex::{Match, Regex};
use serde::export::Formatter;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

const CASTLE_RE: &'static str = "(?P<kc>0-0|O-O)|(?P<qc>0-0-0|O-O-O)";
const MOVE_RE: &'static str = "(?P<pt>[KQRNBP])?(?P<sf>[a-h])?(?P<sr>[1-8])?(?P<cap>x)?(?P<t>[a-h][1-8])(=?(?P<pro>[QRNB]))?(?P<ch>[#+])?";

lazy_static! {
    static ref ALGEBRA_MOVE_RE: Regex = Regex::new(&format!("{}|{}", CASTLE_RE, MOVE_RE)).unwrap();
}

#[derive(Copy, Clone)]
enum Algebraic {
    Move {
        piece_type: PieceType,
        source_file: Option<File>,
        source_rank: Option<Rank>,
        capturing: bool,
        target: Square,
        promoting: Option<PieceType>,
    },
    CastleKingside,
    CastleQueenside,
}

impl Algebraic {
    fn to_move(self, board: &mut Board) -> Option<Move> {
        match self {
            Algebraic::Move {
                piece_type,
                source_file,
                source_rank,
                capturing: _,
                target,
                promoting,
            } => {
                let player = board.player();
                let pieces = board.pieces().clone();
                board.moves().find(|mov| {
                    if pieces[mov.from()] != Some(Piece::new(player, piece_type)) {
                        return false;
                    }

                    if let Some(file) = source_file {
                        if mov.from().file() != file {
                            return false;
                        }
                    }

                    if let Some(rank) = source_rank {
                        if mov.from().rank() != rank {
                            return false;
                        }
                    }

                    if mov.to() != target {
                        return false;
                    }

                    if mov.promoting() != promoting {
                        return false;
                    }

                    true
                })
            }
            Algebraic::CastleKingside => Some(Move::castle_kingside(board.player())),
            Algebraic::CastleQueenside => Some(Move::castle_queenside(board.player())),
        }
    }
}

impl fmt::Display for Algebraic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Algebraic::Move {
                piece_type,
                source_file,
                source_rank,
                capturing,
                target,
                promoting,
            } => {
                write!(f, "{}", piece_type)?;
                if let Some(source_file) = source_file {
                    write!(f, "{}", source_file)?;
                }
                if let Some(source_rank) = source_rank {
                    write!(f, "{}", source_rank)?;
                }
                if *capturing {
                    write!(f, "x")?;
                }
                write!(f, "{}", target);
                if let Some(promoting) = promoting {
                    write!(f, "{}", promoting)?;
                }
                Ok(())
            }
            Algebraic::CastleKingside => write!(f, "O-O"),
            Algebraic::CastleQueenside => write!(f, "O-O-O"),
        }
    }
}

impl FromStr for Algebraic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re: &Regex = &ALGEBRA_MOVE_RE;
        re.captures(s)
            .context(anyhow!("{} isn't an algebraic move", s))
            .and_then(|c| {
                if c.name("kc").is_some() {
                    return Ok(Self::CastleKingside);
                } else if c.name("qc").is_some() {
                    return Ok(Self::CastleQueenside);
                }

                let piece_type = c
                    .name("pt")
                    .map(|m| m.as_str().parse::<PieceType>().unwrap())
                    .unwrap_or(PieceType::Pawn);
                let source_file = c.name("sf").map(|m| m.as_str().parse::<File>().unwrap());
                let source_rank = c.name("sr").map(|m| m.as_str().parse::<Rank>().unwrap());
                let capturing = c.name("cap").is_some();
                let target = c
                    .name("t")
                    .context(anyhow!("Couldn't find a target square"))?
                    .as_str()
                    .parse::<Square>()?;
                let promoting = c
                    .name("pro")
                    .map(|m| m.as_str().parse::<PieceType>().unwrap());

                Ok(Self::Move {
                    piece_type,
                    source_file,
                    source_rank,
                    capturing,
                    target,
                    promoting,
                })
            })
    }
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
