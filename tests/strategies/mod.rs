use chess::{Board, Move, Piece, PieceType, Player};
use proptest::array::{uniform1, uniform8};
use proptest::option;
use proptest::prelude::*;
use proptest::sample::{select, Index};
use proptest::strategy::Union;
use rand::thread_rng;

pub fn arb_player() -> impl Strategy<Value = Player> {
    select(vec![Player::White, Player::Black])
}

pub fn arb_piece_type() -> impl Strategy<Value = PieceType> {
    select(vec![
        PieceType::King,
        PieceType::Queen,
        PieceType::Rook,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Pawn,
    ])
}

pub fn legal_move(board: &mut Board) -> impl Strategy<Value = Option<Move>> {
    let moves: Vec<Move> = board.moves().collect();
    any::<Index>().prop_map(move |idx| {
        if moves.is_empty() {
            None
        } else {
            Some(*idx.get(&moves))
        }
    })
}

pub fn legal_board(num_moves: impl Strategy<Value = u32>) -> impl Strategy<Value = Board> {
    num_moves.prop_flat_map(|n| {
        let mut strategy = Just(Board::default()).boxed();
        for _ in 0..n {
            strategy = strategy
                .prop_flat_map(|mut board| {
                    legal_move(&mut board).prop_map(move |mov| {
                        let mut board = board.clone();
                        if let Some(m) = mov {
                            board.make_move(m);
                        }
                        board
                    })
                })
                .boxed();
        }
        strategy
    })
}

pub fn mate_in_1_board() -> impl Strategy<Value = (Board, Move)> {
    legal_board(0..60u32)
        .prop_filter_map("no mate-in-1", |orig_board| {
            let mut board = orig_board.clone();
            let moves: Vec<Move> = board.moves().collect();
            for mov in moves {
                board.make_move(mov);
                if board.checkmate() {
                    return Some((orig_board, mov));
                }
                board.unmake_move(mov);
            }

            None
        })
        .no_shrink()
}

prop_compose! {
    pub fn arb_piece()(player in arb_player(), piece_type in arb_piece_type()) -> Piece {
        Piece::new(player, piece_type)
    }
}

prop_compose! {
    pub fn arb_board()(pieces in uniform8(uniform8(option::of(arb_piece()))), player in arb_player()) -> Board {
        Board::new(pieces, player)
    }
}
