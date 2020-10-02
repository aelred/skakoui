use skakoui::{Board, Move, PlayedMove, Searcher};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use skakoui::pgn::Algebraic;

fn searcher_can_find_mate_in_1(c: &mut Criterion) {
    let mut searcher = Searcher::default();

    // Searcher tries to castle out of check - FIXME
    const LIMIT: usize = 10;

    for (mut board, mating_move) in mate_in_1s().into_iter().take(LIMIT) {
        c.bench_with_input(
            BenchmarkId::new("mate_in_1", board.fen_url()),
            &board,
            |b, board| {
                b.iter(|| {
                    searcher.go(&board, Some(2));
                    searcher.wait();
                });
            },
        );

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

criterion_group!(searcher, searcher_can_find_mate_in_1);
criterion_main!(searcher);

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
