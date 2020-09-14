use crate::Bitboard;
use crate::Board;
use crate::Move;
use crate::PieceMap;
use crate::Player;
use chashmap::CHashMap;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

const HIGH_SCORE: i32 = std::i32::MAX;
const LOW_SCORE: i32 = -HIGH_SCORE;

/// Table of moves, the key represents the game-state
type TranspositionTable = Arc<CHashMap<Key, Node>>;

type Key = (PieceMap<Bitboard>, Player);

#[derive(Debug)]
struct Node {
    depth: u32,
    value: i32,
    node_type: NodeType,
}

#[derive(Debug, Eq, PartialEq)]
enum NodeType {
    /// Principal variation node, fully explored and value is exact
    PV,
    /// Cut node, or fail-high node, was beta-cutoff, value is a lower bound
    Cut,
    /// All-node, or fail-low node, no moves exceeded alpha, value is an upper bound
    All,
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
    transposition_table: TranspositionTable,
    board: Board,
}

const NUM_THREADS: u32 = 4;

impl Default for Searcher {
    fn default() -> Self {
        let transposition_table = TranspositionTable::default();
        let mut txs = vec![];

        for _ in 0..NUM_THREADS {
            let transposition_table = transposition_table.clone();
            let (msg_tx, msg_rx) = std::sync::mpsc::channel();

            thread::spawn(move || worker_thread(&transposition_table, &msg_rx));

            txs.push(msg_tx);
        }

        Self {
            txs,
            transposition_table,
            board: Board::default(),
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
    pub fn go(&mut self, board: &Board) {
        self.board = board.clone();

        for tx in &self.txs {
            tx.send(Message::StartSearch(Box::new(board.clone())))
                .unwrap();
        }
    }

    pub fn stop(&mut self) {
        for tx in &self.txs {
            tx.send(Message::AbortSearch).unwrap();
        }
    }

    pub fn principal_variation(&self) -> Vec<Move> {
        let mut board = self.board.clone();

        let mut pv = vec![];

        let mut key_set = HashSet::new();

        loop {
            let moves = board.pseudo_legal_moves();

            let mut best_move = None;
            let mut best_score = i32::MIN;

            for mov in moves {
                board.make_move(mov);
                let key = board.key();
                board.unmake_move(mov);

                // Check for loop
                if !key_set.insert(key) {
                    break;
                }

                let entry = self.transposition_table.get(&key);

                match entry {
                    Some(e) if e.node_type == NodeType::PV && e.value > best_score => {
                        best_move = Some(mov);
                        best_score = e.value;
                    }
                    _ => (),
                }
            }

            if let Some(best) = best_move {
                pv.push(best);
                board.make_move(best);
            } else {
                break;
            }
        }

        // Return a totally random move if we couldn't find anything.
        // This can happen if search is stopped very quickly.
        if pv.is_empty() {
            if let Some(mov) = board.pseudo_legal_moves().next() {
                pv.push(mov);
            }
        }

        pv
    }

    pub fn clear(&self) {
        self.transposition_table.clear();
    }
}

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

    fn run(&mut self) {
        let mut depth = 1;

        while !self.should_abort() {
            self.search(depth, LOW_SCORE, HIGH_SCORE);
            depth += 1
        }
    }

    fn search(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
        if self.should_abort() {
            return self.board.eval();
        }

        let key = self.board.key();

        if let Some(entry) = self.transposition_table.get(&key) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::PV => {
                        return entry.value;
                    }
                    NodeType::All => {
                        alpha = alpha.max(entry.value);
                    }
                    NodeType::Cut => {
                        beta = beta.min(entry.value);
                    }
                }

                if alpha >= beta {
                    return entry.value;
                }
            }
        }

        let value = self.search_uncached(depth, alpha, beta);
        let node_type = if value >= beta {
            NodeType::Cut
        } else if value <= alpha {
            NodeType::All
        } else {
            NodeType::PV
        };
        let entry = Node {
            depth,
            value,
            node_type,
        };

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
