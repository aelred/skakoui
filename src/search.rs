use crate::Bitboard;
use crate::Board;
use crate::Move;
use crate::PieceMap;
use crate::Player;
use chashmap::CHashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

#[derive(Debug)]
enum Message {
    StartSearch(Box<Board>),
    AbortSearch,
    AbortThread,
}

pub struct Searcher {
    txs: Vec<Sender<Message>>,
    transposition_table: TranspositionTable,
}

const NUM_THREADS: u32 = 4;

impl Default for Searcher {
    fn default() -> Self {
        let transposition_table = TranspositionTable::default();
        let mut txs = vec![];

        for _ in 0..NUM_THREADS {
            let transposition_table = transposition_table.clone();
            let (tx, rx) = std::sync::mpsc::channel();

            thread::spawn(move || worker_thread(&transposition_table, &rx));

            txs.push(tx);
        }

        Self {
            txs,
            transposition_table,
        }
    }
}

fn worker_thread(transposition_table: &TranspositionTable, rx: &Receiver<Message>) {
    loop {
        match rx.recv().unwrap() {
            Message::StartSearch(mut board) => {
                let mut searcher = ThreadSearcher::new(&mut board, transposition_table, rx);
                searcher.run();
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
    pub fn run(&mut self, board: &mut Board) -> (Option<Move>, i32) {
        for tx in &self.txs {
            tx.send(Message::StartSearch(Box::new(board.clone())))
                .unwrap();
        }

        let transposition_table = self.transposition_table.clone();
        let mut local_searcher = LocalSearcher::new(board, transposition_table);

        let result = local_searcher.run();

        for tx in &self.txs {
            tx.send(Message::AbortSearch).unwrap();
        }

        result
    }
}

const DEPTH: u32 = 5;

struct ThreadSearcher<'a> {
    board: &'a mut Board,
    transposition_table: &'a TranspositionTable,
    rx: &'a Receiver<Message>,
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
        }
    }

    fn run(&mut self) {
        let mut moves: Vec<Move> = self.board.pseudo_legal_moves().collect();
        moves.shuffle(&mut thread_rng());

        let mut alpha = LOW_SCORE;

        for mov in moves {
            println!("{}", mov);
            self.board.make_move(mov);
            if let Some(value) = self.search(DEPTH - 1, LOW_SCORE, -alpha) {
                let value = -value;

                if value > alpha {
                    alpha = value;
                }
            } else {
                return;
            }
            self.board.unmake_move(mov);
        }
    }

    fn search(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> Option<i32> {
        let alpha_orig = alpha;

        let key = self.board.key();

        if let Some(entry) = self.transposition_table.get(&key) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => {
                        return Some(entry.value);
                    }
                    Flag::LowerBound => {
                        alpha = i32::max(alpha, entry.value);
                    }
                    Flag::UpperBound => {
                        beta = i32::min(beta, entry.value);
                    }
                }

                if alpha >= beta {
                    return Some(entry.value);
                }
            }
        }

        let value = self.search_uncached(depth, alpha, beta)?;

        let flag = if value <= alpha_orig {
            Flag::UpperBound
        } else if value >= beta {
            Flag::LowerBound
        } else {
            Flag::Exact
        };

        let entry = TranspositionEntry { depth, value, flag };

        self.transposition_table.insert(key, entry);

        Some(value)
    }

    fn search_uncached(&mut self, depth: u32, mut alpha: i32, beta: i32) -> Option<i32> {
        if depth == 0 {
            if let Ok(Message::AbortSearch) = self.rx.try_recv() {
                return None;
            }

            return Some(quiesce(&mut self.board, alpha, beta));
        }

        let mut moves = self.board.pseudo_legal_moves().peekable();

        if moves.peek().is_none() {
            return Some(LOW_SCORE);
        }

        for mov in moves {
            self.board.make_move(mov);
            let value = -self.search(depth - 1, -beta, -alpha)?;
            self.board.unmake_move(mov);

            if value >= beta {
                return Some(beta);
            }
            if value > alpha {
                alpha = value;
            }
        }

        Some(alpha)
    }
}

struct LocalSearcher<'a> {
    board: &'a mut Board,
    transposition_table: TranspositionTable,
}

impl<'a> LocalSearcher<'a> {
    fn new(board: &'a mut Board, transposition_table: TranspositionTable) -> Self {
        Self {
            board,
            transposition_table,
        }
    }

    fn run(&mut self) -> (Option<Move>, i32) {
        let moves = self.board.pseudo_legal_moves();

        let mut alpha = LOW_SCORE;
        let mut best_moves = vec![];

        for mov in moves {
            self.board.make_move(mov);
            let value = -self.search(DEPTH, LOW_SCORE, -alpha);
            self.board.unmake_move(mov);

            if value > alpha {
                alpha = value;
                best_moves = vec![mov];
            } else if value == alpha {
                best_moves.push(mov);
            }
        }

        let best_move = best_moves.choose(&mut rand::thread_rng()).cloned();

        (best_move, alpha)
    }

    fn search(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
        let alpha_orig = alpha;

        let key = self.board.key();

        if let Some(entry) = self.transposition_table.get(&key) {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => {
                        return entry.value;
                    }
                    Flag::LowerBound => {
                        alpha = i32::max(alpha, entry.value);
                    }
                    Flag::UpperBound => {
                        beta = i32::min(beta, entry.value);
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
            return quiesce(&mut self.board, alpha, beta);
        }

        let mut moves = self.board.pseudo_legal_moves().peekable();

        if moves.peek().is_none() {
            return LOW_SCORE;
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
}

fn quiesce(board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
    let stand_pat = board.eval();

    if stand_pat >= beta {
        return beta;
    }

    if alpha < stand_pat {
        alpha = stand_pat;
    }

    for mov in board.capturing_moves() {
        board.make_move(mov);
        let value = -quiesce(board, -beta, -alpha);
        board.unmake_move(mov);

        if value >= beta {
            return beta;
        }
        if value > alpha {
            alpha = value;
        }
    }

    alpha
}
