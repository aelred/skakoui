use skakoui::{Board, Move, PlayedMove, Searcher};

use criterion::measurement::Measurement;
use criterion::{
    criterion_group, criterion_main, BatchSize, BenchmarkGroup, Criterion, SamplingMode, Throughput,
};
use itertools::Itertools;
use skakoui::pgn::Algebraic;

fn searcher_can_find_mate(c: &mut Criterion) {
    let mut group = c.benchmark_group("mate");
    // for long-running benchmarks
    group.sampling_mode(SamplingMode::Flat).sample_size(10);

    test_find_mates("in1", &mut group, mate_in_1s());
    test_find_mates("in2", &mut group, mate_in_2s());
    // TODO: too slow for all of these
    test_find_mates("in3", &mut group, mate_in_3s().take(10));

    group.finish();
}

criterion_group!(searcher, searcher_can_find_mate);
criterion_main!(searcher);

fn test_find_mates<M: Measurement>(
    name: &'static str,
    g: &mut BenchmarkGroup<M>,
    mates: impl Iterator<Item = (Board, Vec<Move>)>,
) {
    // Only run a sample for benchmarking, but run all if testing
    let mates = mates.take(if cfg!(bench) { 10 } else { usize::MAX });

    let mates: Vec<(Board, Vec<Move>)> = mates.collect();
    g.throughput(Throughput::Elements(mates.len() as u64));
    g.bench_function(name, move |b| {
        b.iter_batched(
            || (Searcher::default(), mates.clone()),
            |(mut searcher, mates)| {
                for (board, mating_moves) in mates {
                    test_find_mate(&mut searcher, board, &mating_moves);
                }
            },
            BatchSize::LargeInput,
        );
    });
}

fn test_find_mate(searcher: &mut Searcher, mut board: Board, mating_moves: &[Move]) {
    let n = mating_moves.len();

    searcher.go(&board, Some(n as u16 + 1));
    searcher.wait();

    let mut moves = searcher.principal_variation(&mut board);
    moves.truncate(n);

    let mut test_board = board.clone();
    for mov in moves.iter() {
        test_board.make_move(*mov);
    }
    let checkmate = test_board.checkmate();

    assert!(
        checkmate,
        "{}\n{}\nExpect: {}\nActual: {}",
        board,
        board.fen_url(),
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
