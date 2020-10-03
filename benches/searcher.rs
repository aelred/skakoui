use skakoui::{Board, Move, PlayedMove, Searcher};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use skakoui::pgn::Algebraic;
use skakoui::GameState;

fn searcher_can_find_mate(c: &mut Criterion) {
    let mut searcher = Searcher::default();

    let puzzles = mate_in_1s().chain(mate_in_2s()).chain(mate_in_3s());

    for (board, mating_moves) in puzzles {
        test_find_mate(c, &mut searcher, board, &mating_moves);
    }
}

criterion_group!(searcher, searcher_can_find_mate);
criterion_main!(searcher);

fn test_find_mate(c: &mut Criterion, searcher: &mut Searcher, board: Board, mating_moves: &[Move]) {
    let fen_url = board.fen_url();
    let mut state = GameState::new(board);
    let n = mating_moves.len();

    c.bench_with_input(
        BenchmarkId::new(format!("mate_in_{}", n / 2 + 1), &fen_url),
        &state.board,
        |b, board| {
            b.iter(|| {
                searcher.go(&board, Some(n as u16 + 1));
                searcher.wait();
            });
        },
    );

    let mut moves = searcher.principal_variation();
    moves.truncate(n);

    for mov in moves.iter() {
        state.push_move(*mov);
    }
    let checkmate = state.board.checkmate();
    for _ in moves.iter() {
        state.pop();
    }

    assert!(
        checkmate,
        "{}\n{}\nExpect: {}\nActual: {}",
        state.board,
        fen_url,
        mating_moves.iter().join(" "),
        moves.iter().join(" ")
    );
}

fn mate_in_1s() -> impl Iterator<Item = (Board, Vec<Move>)> {
    mate_in_2s().into_iter().map(|(mut board, moves)| {
        board.make_move(moves[0]);
        board.make_move(moves[1]);
        (board, vec![moves[2]])
    })
}

fn mate_in_2s() -> impl Iterator<Item = (Board, Vec<Move>)> {
    read_mates(include_str!("m8n2.txt"))
}

fn mate_in_3s() -> impl Iterator<Item = (Board, Vec<Move>)> {
    // TODO: still too slow to try these all
    read_mates(include_str!("m8n3.txt")).take(20)
}

fn read_mates(mates_str: &'static str) -> impl Iterator<Item = (Board, Vec<Move>)> {
    let mut lines = mates_str.lines().peekable();

    std::iter::from_fn(move || {
        // Try to parse each line as a board position
        let board = lines.find_map(|line| Board::from_fen(line).ok())?;

        let mstr = lines.next()?;
        let mstr = mstr
            .replace("1.", "")
            .replace("2.", "")
            .replace("3.", "")
            .replace(".", "")
            .replace("*", "");

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
