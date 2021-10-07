use crate::{Bitboard, Board, BoardFlags, GameState, Move, Piece, PieceTypeV, PlayerV, Square};
use arrayvec::ArrayVec;
use proptest::bool::weighted;
use proptest::collection::{vec, SizeRange};
use proptest::prelude::*;
use proptest::sample::{select, Index};

pub fn arb_player() -> impl Strategy<Value = PlayerV> {
    const PLAYERS: &[PlayerV] = &[PlayerV::White, PlayerV::Black];
    select(PLAYERS)
}

pub fn all_pieces() -> Vec<Option<Piece>> {
    Board::default().iter().map(|(_, piece)| *piece).collect()
}

pub fn arb_pieces() -> impl Strategy<Value = [[Option<Piece>; 8]; 8]> {
    let all_pieces = all_pieces();
    let len = all_pieces.len();

    // odds are chosen per-board instead of per-piece, so we get boards with many pieces and some with few
    let keep_pieces = (0.0..=1.0).prop_flat_map(move |keep_odds| vec(weighted(keep_odds), len));

    let shuffled_pieces = Just(all_pieces).prop_shuffle();
    (shuffled_pieces, keep_pieces)
        .prop_map(|(pieces, keep)| {
            pieces
                .into_iter()
                .zip(keep.into_iter())
                .map(|(piece, keep)| piece.filter(|p| keep || p.piece_type() == PieceTypeV::King))
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

pub fn arb_piece_type() -> impl Strategy<Value = PieceTypeV> {
    let mut pawns = std::iter::repeat(PieceTypeV::Pawn).take(8).collect();
    let mut types = vec![
        PieceTypeV::King,
        PieceTypeV::Queen,
        PieceTypeV::Rook,
        PieceTypeV::Rook,
        PieceTypeV::Bishop,
        PieceTypeV::Bishop,
        PieceTypeV::Knight,
        PieceTypeV::Knight,
    ];
    types.append(&mut pawns);
    select(types)
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
            let moves: Vec<Move> = board.moves().collect();
            if moves.is_empty() {
                None
            } else {
                Some((board, moves))
            }
        })
        .prop_flat_map(|(board, moves)| (Just(board), select(moves)))
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

pub fn arb_bitboard() -> impl Strategy<Value = Bitboard> {
    (any::<u64>(), any::<u64>()).prop_map(|(x, y)| Bitboard::new(x & y))
}

pub fn arb_square() -> impl Strategy<Value = Square> {
    (0..64u8).prop_map(Square::from_index)
}
