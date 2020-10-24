use crate::{Board, File, Move, Piece, PieceType, Rank, Square};
use anyhow::{anyhow, Context, Error};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::str::FromStr;

/// A move in
/// [Standard Algebraic Notation](https://en.wikipedia.org/wiki/Algebraic_notation_(chess)).
#[derive(Copy, Clone)]
pub enum Algebraic {
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
    pub fn to_move(self, board: &mut Board) -> Option<Move> {
        match self {
            Algebraic::Move {
                piece_type,
                source_file,
                source_rank,
                capturing: _,
                target,
                promoting,
            } => board.pseudo_legal_moves().find(|mov| {
                if board[mov.from()] != Some(Piece::new(board.player(), piece_type)) {
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

                board.check_legal(*mov)
            }),
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
                write!(f, "{}", target)?;
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

                let piece_type = match c.name("pt") {
                    Some(m) => m.as_str().parse::<PieceType>().unwrap(),
                    None => PieceType::Pawn,
                };
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

lazy_static! {
    static ref ALGEBRA_MOVE_RE: Regex = Regex::new(&format!("{}|{}", CASTLE_RE, MOVE_RE)).unwrap();
}

const CASTLE_RE: &str = "(?P<kc>0-0|O-O)|(?P<qc>0-0-0|O-O-O)";
const MOVE_RE: &str = "(?P<pt>[KQRNBP])?(?P<sf>[a-h])?(?P<sr>[1-8])?(?P<cap>x)?(?P<t>[a-h][1-8])(=?(?P<pro>[QRNB]))?(?P<ch>[#+])?";
