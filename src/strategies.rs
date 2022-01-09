use crate::piece::{BK, WK};
use crate::{Bitboard, Board, BoardFlags, GameState, Move, PieceTypeV, PieceV, PlayerV, Square};
use proptest::bits;
use proptest::collection::{vec, SizeRange};
use proptest::prelude::*;
use proptest::sample::{select, subsequence, Index};

pub fn arb_player() -> impl Strategy<Value = PlayerV> {
    const PLAYERS: &[PlayerV] = &[PlayerV::White, PlayerV::Black];
    select(PLAYERS)
}

pub fn arb_pieces() -> impl Strategy<Value = [[Option<PieceV>; 8]; 8]> {
    let non_king_pieces: Vec<PieceV> = Board::default()
        .iter()
        .flat_map(|(_, piece)| *piece)
        .filter(|piece| piece.piece_type != PieceTypeV::King)
        .collect();

    let all_squares: Vec<Square> = Square::all().collect();
    let squares = Just(all_squares).prop_shuffle().no_shrink();

    (Just(non_king_pieces), squares)
        .prop_flat_map(|(non_king_pieces, squares)| {
            let (king_sq, non_king_sq) = squares.split_at(2);
            let wk = king_sq[0];
            let bk = king_sq[1];
            let zipped: Vec<(Square, PieceV)> =
                non_king_sq.iter().copied().zip(non_king_pieces).collect();
            let len = zipped.len();
            // This 0..=len means proptest will shrink by removing pieces
            subsequence(zipped, 0..=len).prop_map(move |mut pieces| {
                pieces.push((wk, WK.value()));
                pieces.push((bk, BK.value()));
                pieces
            })
        })
        .prop_map(|pieces| {
            let mut arr: [[Option<PieceV>; 8]; 8] = Default::default();
            for (sq, piece) in pieces {
                arr[sq.rank().to_index() as usize][sq.file().to_index() as usize] = Some(piece);
            }
            arr
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
    bits::u16::ANY.prop_map(BoardFlags::new)
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
    (bits::u64::ANY, bits::u64::ANY).prop_map(|(x, y)| Bitboard::new(x & y))
}

pub fn arb_square() -> impl Strategy<Value = Square> {
    (0..64u8).prop_map(Square::from_index)
}
