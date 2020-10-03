use arrayvec::ArrayVec;
use proptest::array::uniform8;
use proptest::bool::weighted;
use proptest::collection::{vec, SizeRange};
use proptest::option;
use proptest::prelude::*;
use proptest::sample::{select, Index};
use skakoui::{Board, BoardFlags, Move, Piece, PieceType, PlayedMove, Player};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
struct GameState {
    board: Board,
    moves: Vec<PlayedMove>,
}

impl GameState {
    fn new(board: Board) -> Self {
        Self {
            board,
            moves: vec![],
        }
    }

    fn push_move(&mut self, mov: Move) {
        let pmov = self.board.make_move(mov);
        self.moves.push(pmov);
    }

    fn pop(&mut self) -> Option<Move> {
        self.moves.pop().map(|pmov| {
            self.board.unmake_move(pmov);
            pmov.mov
        })
    }

    fn moves(&self) -> impl Iterator<Item = &Move> {
        self.moves.iter().map(|pm| &pm.mov)
    }
}

pub fn arb_player() -> impl Strategy<Value = Player> {
    select(vec![Player::White, Player::Black])
}

pub fn all_pieces() -> Vec<Option<Piece>> {
    Board::default()
        .pieces()
        .iter()
        .map(|(_, piece)| *piece)
        .collect()
}

pub fn arb_pieces() -> impl Strategy<Value = [[Option<Piece>; 8]; 8]> {
    // odds are chosen per-board instead of per-piece, so we get boards with many pieces and some with few
    let keep_pieces = (0.0..=1.0).prop_flat_map(|keep_odds| vec(weighted(keep_odds), 64));

    let shuffled_pieces = Just(all_pieces()).prop_shuffle();
    (shuffled_pieces, keep_pieces)
        .prop_map(|(pieces, keep)| {
            pieces
                .into_iter()
                .zip(keep.into_iter())
                .map(|(piece, keep)| piece.filter(|p| keep || p.piece_type() == PieceType::King))
                .collect::<Vec<Option<Piece>>>()
        })
        .prop_map(|pieces| {
            let arr: ArrayVec<[[Option<Piece>; 8]; 8]> = pieces
                .chunks(8)
                .map(|slice| {
                    let mut arrvec = ArrayVec::<[Option<Piece>; 8]>::new();
                    arrvec.try_extend_from_slice(slice).unwrap();
                    arrvec.into_inner().unwrap()
                })
                .collect();
            arr.into_inner().unwrap()
        })
}

pub fn arb_piece_type() -> impl Strategy<Value = PieceType> {
    let mut pawns = std::iter::repeat(PieceType::Pawn).take(8).collect();
    let mut types = vec![
        PieceType::King,
        PieceType::Queen,
        PieceType::Rook,
        PieceType::Rook,
        PieceType::Bishop,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Knight,
    ];
    types.append(&mut pawns);
    select(types)
}

pub fn legal_move(board: &mut Board) -> impl Strategy<Value = Move> {
    let moves: Vec<Move> = board.moves().collect();
    any::<Index>().prop_filter_map("stalemate", move |idx| {
        if moves.is_empty() {
            None
        } else {
            Some(*idx.get(&moves))
        }
    })
}

pub fn arb_flags() -> impl Strategy<Value = BoardFlags> {
    any::<u8>().prop_map(BoardFlags::new)
}

pub fn legal_game_state(num_moves: impl Into<SizeRange>) -> impl Strategy<Value = GameState> {
    vec(any::<Index>(), num_moves).prop_map(|move_idxs| {
        let mut state = GameState::default();
        for move_idx in move_idxs.iter() {
            let moves: Vec<Move> = state.board.moves().collect();
            if moves.is_empty() {
                break;
            } else {
                state.push_move(*move_idx.get(&moves));
            };
        }

        state
    })
}

pub fn legal_board(num_moves: impl Into<SizeRange>) -> impl Strategy<Value = Board> {
    legal_game_state(num_moves).prop_map(|s| s.board)
}

pub fn board_and_move(
    boards: impl Strategy<Value = Board>,
) -> impl Strategy<Value = (Board, Move)> {
    boards
        .prop_filter_map("stalemate", |mut board| {
            if board.moves().next().is_some() {
                Some(board)
            } else {
                None
            }
        })
        .prop_flat_map(|mut board| {
            let moves = legal_move(&mut board);
            (Just(board), moves)
        })
}

pub fn mate_in_1_board() -> impl Strategy<Value = (Board, Move)> {
    board_and_move(legal_board(3..40)).prop_filter_map("not a mate-in-1", |(mut board, mov)| {
        dbg!(&board);
        let pmov = board.make_move(mov);
        let checkmate = board.checkmate();
        board.unmake_move(pmov);
        if checkmate {
            Some((board, mov))
        } else {
            None
        }
    })
}

pub fn arb_board() -> impl Strategy<Value = Board> {
    (arb_pieces(), arb_player(), arb_flags())
        .prop_map(|(pieces, player, flags)| Board::new(pieces, player, flags))
}