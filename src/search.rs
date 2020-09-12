use crate::Bitboard;
use crate::Board;
use crate::Move;
use crate::PieceMap;
use crate::Player;
use chashmap::CHashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

const HIGH_SCORE: i32 = std::i32::MAX;
const LOW_SCORE: i32 = -HIGH_SCORE;

/// Table of moves, the key represents the game-state
type TranspositionTable = Arc<CHashMap<Key, TranspositionEntry>>;

type Key = (PieceMap<Bitboard>, Player);

struct TranspositionEntry {
    depth: u32,
    value: i32,
    flag: Flag,
}

enum Flag {
    Exact,
    LowerBound,
    UpperBound,
}

impl Board {
    fn key(&self) -> Key {
        (*self.bitboards(), self.player())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Message {
    StartSearch(Box<Board>),
    AbortSearch,
    AbortThread,
}

pub struct Searcher {
    txs: Vec<Sender<Message>>,
    rxs: Vec<Receiver<(Option<Move>, i32)>>,
    transposition_table: TranspositionTable,
}

const NUM_THREADS: u32 = 4;

impl Default for Searcher {
    fn default() -> Self {
        let transposition_table = TranspositionTable::default();
        let mut txs = vec![];
        let mut rxs = vec![];

        for _ in 0..NUM_THREADS {
            let transposition_table = transposition_table.clone();
            let (msg_tx, msg_rx) = std::sync::mpsc::channel();
            let (moves_tx, moves_rx) = std::sync::mpsc::channel();

            thread::spawn(move || worker_thread(&transposition_table, &msg_rx, &moves_tx));

            txs.push(msg_tx);
            rxs.push(moves_rx);
        }

        Self {
            txs,
            rxs,
            transposition_table,
        }
    }
}

fn worker_thread(
    transposition_table: &TranspositionTable,
    rx: &Receiver<Message>,
    moves: &Sender<(Option<Move>, i32)>,
) {
    loop {
        match rx.recv().unwrap() {
            Message::StartSearch(mut board) => {
                let mut searcher = ThreadSearcher::new(&mut board, transposition_table, rx);
                let result = searcher.run();
                moves.send(result).unwrap();
            }
            Message::AbortThread => {
                return;
            }
            Message::AbortSearch => {
                // We're already done searching, so ignore
            }
        }
    }
}

impl Drop for Searcher {
    fn drop(&mut self) {
        for tx in &self.txs {
            let _ = tx.send(Message::AbortThread);
        }
    }
}

impl Searcher {
    pub fn go(&mut self, board: &mut Board) {
        for tx in &self.txs {
            tx.send(Message::StartSearch(Box::new(board.clone())))
                .unwrap();
        }
    }

    pub fn stop(&mut self) -> (Option<Move>, i32) {
        println!("info string stopping");
        for tx in &self.txs {
            tx.send(Message::AbortSearch).unwrap();
        }
        println!("info string sent stop message to all threads");

        println!("info string waiting for list of moves");
        let moves = self.rxs.iter().map(|x| x.recv().unwrap());
        moves.max_by_key(|(_, score)| *score).unwrap()
    }

    pub fn clear(&self) {
        self.transposition_table.clear();
    }
}

const DEPTH: u32 = 5;

struct ThreadSearcher<'a> {
    board: &'a mut Board,
    transposition_table: &'a TranspositionTable,
    rx: &'a Receiver<Message>,
    abort: bool,
}

impl<'a> ThreadSearcher<'a> {
    fn new(
        board: &'a mut Board,
        transposition_table: &'a TranspositionTable,
        rx: &'a Receiver<Message>,
    ) -> Self {
        Self {
            board,
            transposition_table,
            rx,
            abort: false,
        }
    }

    fn run(&mut self) -> (Option<Move>, i32) {
        let mut moves: Vec<Move> = self.board.pseudo_legal_moves().collect();
        let rng = &mut thread_rng();
        moves.shuffle(rng);

        let mut alpha = LOW_SCORE;
        let mut best_moves = vec![];

        for mov in moves {
            self.board.make_move(mov);
            let value = -self.search(DEPTH - 1, LOW_SCORE, -alpha);
            self.board.unmake_move(mov);

            match value.cmp(&alpha) {
                Ordering::Greater => {
                    alpha = value;
                    best_moves = vec![mov];
                }
                Ordering::Equal => best_moves.push(mov),
                Ordering::Less => (),
            }
        }

        let best_move = best_moves.choose(rng).cloned();

        (best_move, alpha)
    }

    fn search(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
        if self.should_abort() {
            return self.board.eval();
        }

        let alpha_orig = alpha;

        let key = self.board.key();

        if let Some(entry) = self.transposition_table.get(&key) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => {
                        return entry.value;
                    }
                    Flag::LowerBound => {
                        alpha = alpha.max(entry.value);
                    }
                    Flag::UpperBound => {
                        beta = beta.min(entry.value);
                    }
                }

                if alpha >= beta {
                    return entry.value;
                }
            }
        }

        let value = self.search_uncached(depth, alpha, beta);

        let flag = if value <= alpha_orig {
            Flag::UpperBound
        } else if value >= beta {
            Flag::LowerBound
        } else {
            Flag::Exact
        };

        let entry = TranspositionEntry { depth, value, flag };

        self.transposition_table.insert(key, entry);

        value
    }

    fn search_uncached(&mut self, depth: u32, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            return self.quiesce(alpha, beta);
        }

        let mut moves = self.board.pseudo_legal_moves().peekable();

        if moves.peek().is_none() {
            return 0; // Being unable to move is a tie
        }

        for mov in moves {
            self.board.make_move(mov);
            let value = -self.search(depth - 1, -beta, -alpha);
            self.board.unmake_move(mov);

            if value >= beta {
                return beta;
            }
            if value > alpha {
                alpha = value;
            }
        }

        alpha
    }

    /// Evaluate how "quiescent" (quiet or stable) a board is.
    ///
    /// The idea is that a board with lots going on is worth investigating more deeply.
    /// This helps prevent the AI picking bad moves because the board "looks" good, even if an important
    /// piece could be taken in the next turn.
    fn quiesce(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.board.eval();

        if self.should_abort() {
            return stand_pat;
        }

        if stand_pat >= beta {
            return beta;
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        for mov in self.board.capturing_moves() {
            self.board.make_move(mov);
            let value = -self.quiesce(-beta, -alpha);
            self.board.unmake_move(mov);

            if value >= beta {
                return beta;
            }
            if value > alpha {
                alpha = value;
            }
        }

        alpha
    }

    fn should_abort(&mut self) -> bool {
        self.abort = self.abort || self.rx.try_recv() == Ok(Message::AbortSearch);
        self.abort
    }
}
