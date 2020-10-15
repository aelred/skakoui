use crate::{Black, Board, BoardFlags, Piece, PieceType, Player, PlayerV, White};
use anyhow::{anyhow, Context, Error};
use arrayvec::ArrayVec;
use std::borrow::Borrow;

impl Board {
    /// Parse a board from
    /// [Forsyth-Edwards notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
    pub fn from_fen(fen: impl Into<String>) -> Result<Self, Error> {
        let fen_str = fen.into();
        let mut fields = fen_str.split_whitespace();
        let pieces_str = fields.next().context("Expected pieces")?;
        let pieces_by_rank = pieces_str.split('/');

        let mut pieces_vec = ArrayVec::<[[Option<Piece>; 8]; 8]>::new();
        for rank in pieces_by_rank {
            let mut rank_vec = ArrayVec::<[Option<Piece>; 8]>::new();
            for c in rank.chars() {
                let s = c.to_string();
                if let Ok(empties) = s.parse::<usize>() {
                    for _ in 0..empties {
                        rank_vec
                            .try_push(None)
                            .with_context(|| anyhow!("More than 8 squares in rank: {}", rank))?;
                    }
                } else if let Ok(piece) = s.parse::<Piece>() {
                    rank_vec
                        .try_push(Some(piece))
                        .with_context(|| anyhow!("More than 8 squares in rank: {}", rank))?;
                }
            }
            pieces_vec
                .try_push(
                    rank_vec
                        .into_inner()
                        .map_err(|_| anyhow!("Less than 8 squares in rank: {}", rank))?,
                )
                .with_context(|| anyhow!("More than 8 ranks: {}", pieces_str))?;
        }
        pieces_vec.reverse();
        let pieces_array = pieces_vec
            .into_inner()
            .map_err(|_| anyhow!("Less than 8 ranks: {}", pieces_str))?;

        let player = fields
            .next()
            .context("Expected player after pieces")?
            .parse::<PlayerV>()?;

        let flags = fields.next().map(|castling| {
            let mut set_flags = 0u8;
            if castling.contains('K') {
                set_flags |= White.castle_kingside_flag();
            }
            if castling.contains('Q') {
                set_flags |= White.castle_queenside_flag();
            }
            if castling.contains('k') {
                set_flags |= Black.castle_kingside_flag();
            }
            if castling.contains('q') {
                set_flags |= Black.castle_queenside_flag();
            }
            BoardFlags::new(set_flags)
        });

        // TODO: also parse en passant and number of moves

        Ok(Self::new(pieces_array, player, flags.unwrap_or_default()))
    }

    pub fn to_fen(&self) -> String {
        let mut array: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        for (square, piece) in self.pieces().iter() {
            array[square.rank().to_index() as usize][square.file().to_index() as usize] = *piece;
        }

        let mut fen = String::new();
        let mut empty_count = 0;

        fn push_empty_count(fen: &mut String, empty_count: &mut i32) {
            if *empty_count != 0 {
                fen.push_str(&empty_count.to_string());
                *empty_count = 0;
            }
        }

        for rank in array.iter().rev() {
            if !fen.is_empty() {
                fen.push('/');
            }

            for square in rank {
                match square {
                    Some(piece) => {
                        push_empty_count(&mut fen, &mut empty_count);
                        fen.push_str(&piece.to_fen())
                    }
                    None => empty_count += 1,
                }
            }
            push_empty_count(&mut fen, &mut empty_count);
        }

        fen.push(' ');
        fen.push_str(&self.player().to_fen());

        fen.push(' ');
        let mut can_castle = false;
        if self.flags().is_set(White.castle_kingside_flag()) {
            fen.push('K');
            can_castle = true;
        }
        if self.flags().is_set(White.castle_queenside_flag()) {
            fen.push('Q');
            can_castle = true;
        }
        if self.flags().is_set(Black.castle_kingside_flag()) {
            fen.push('k');
            can_castle = true;
        }
        if self.flags().is_set(Black.castle_queenside_flag()) {
            fen.push('q');
            can_castle = true;
        }
        if !can_castle {
            fen.push('-');
        }

        fen
    }
}

impl PlayerV {
    pub fn to_fen(self) -> String {
        self.char().to_ascii_lowercase().to_string()
    }

    pub fn from_fen(s: &str) -> Result<Self, Error> {
        let player = match s {
            "W" | "w" => Self::White,
            "B" | "b" => Self::Black,
            _ => return Err(anyhow!("Expected W, w, B or b")),
        };
        Ok(player)
    }
}

impl Piece {
    pub fn to_fen(&self) -> String {
        let c = self.piece_type().to_fen();
        if self.player() == PlayerV::White {
            c.to_ascii_uppercase()
        } else {
            c.to_ascii_lowercase()
        }
    }

    pub fn from_fen(str: &str) -> Result<Self, Error> {
        let piece = match str {
            "♔" | "K" => Self::WK,
            "♕" | "Q" => Self::WQ,
            "♖" | "R" => Self::WR,
            "♗" | "B" => Self::WB,
            "♘" | "N" => Self::WN,
            "♙" | "P" => Self::WP,
            "♚" | "k" => Self::BK,
            "♛" | "q" => Self::BQ,
            "♜" | "r" => Self::BR,
            "♝" | "b" => Self::BB,
            "♞" | "n" => Self::BN,
            "♟" | "p" => Self::BP,
            x => return Err(anyhow!("Unexpected piece '{}'", x)),
        };

        Ok(piece)
    }
}

impl PieceType {
    pub fn to_fen(self) -> String {
        self.to_char().to_string()
    }

    pub fn from_fen(s: &str) -> Result<Self, Error> {
        let piece_type = match s.to_ascii_uppercase().borrow() {
            "K" => Self::King,
            "Q" => Self::Queen,
            "R" => Self::Rook,
            "B" => Self::Bishop,
            "N" => Self::Knight,
            "P" => Self::Pawn,
            _ => return Err(anyhow!("Unexpected piece type '{}'", s)),
        };

        Ok(piece_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_board_from_fen_notation() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board, Board::default());
    }
}
