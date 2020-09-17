use crate::piece::PieceType::King;
use crate::Board;
use crate::Move;
use crate::PieceMap;
use crate::Player;
use crate::{Bitboard, Piece, PieceType};
use chashmap::CHashMap;
use std::collections::HashSet;
use std::io::Write;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use ttable::{Key, Node, NodeType, TranspositionTable};

mod ttable;

const HIGH_SCORE: i32 = std::i32::MAX;
const WIN: i32 = HIGH_SCORE - 1; // Very high, but not the highest possible value
const LOW_SCORE: i32 = -HIGH_SCORE; // Not std::i32::MIN or we get overflows on negation

macro_rules! log_search {
    ($depth:expr, $($arg:tt)*) => ({
        if cfg!(feature = "log-search") {
            let indent = std::iter::repeat(' ')
                .take(10 - $depth as usize * 2)
                .collect::<String>();
            println!("{}- {}. {}", indent, $depth, format_args!($($arg)*))
        }
    })
}

impl Board {
    fn key(&self) -> Key {
        (*self.bitboards(), self.player())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Request {
    StartSearch(Box<Board>),
    AbortSearch,
    AbortThread,
}

#[derive(Debug, PartialEq, Eq)]
enum Response {
    StoppedSearch,
}

pub struct Searcher {
    txs: Vec<Sender<Request>>,
    rxs: Vec<Receiver<Response>>,
    transposition_table: Arc<TranspositionTable>,
    board: Board,
}

fn num_threads() -> u32 {
    if cfg!(feature = "log-search") {
        1
    } else {
        4
    }
}

impl Default for Searcher {
    fn default() -> Self {
        // Each entry is ~128 bytes, so this should be ~128MB
        const TABLE_SIZE: usize = 1024 * 1024;

        let transposition_table = Arc::new(TranspositionTable::new(TABLE_SIZE));
        let mut txs = vec![];
        let mut rxs = vec![];

        for _ in 0..num_threads() {
            let transposition_table = transposition_table.clone();
            let (req_tx, req_rx) = std::sync::mpsc::channel();
            let (res_tx, res_rx) = std::sync::mpsc::channel();

            thread::spawn(move || worker_thread(&transposition_table, &req_rx, &res_tx));

            txs.push(req_tx);
            rxs.push(res_rx);
        }

        Self {
            txs,
            rxs,
            transposition_table,
            board: Board::default(),
        }
    }
}

fn worker_thread(
    transposition_table: &Arc<TranspositionTable>,
    rx: &Receiver<Request>,
    tx: &Sender<Response>,
) {
    loop {
        match rx.recv().unwrap() {
            Request::StartSearch(mut board) => {
                let mut searcher = ThreadSearcher::new(&mut board, transposition_table, rx);
                searcher.run();
                tx.send(Response::StoppedSearch).unwrap();
            }
            Request::AbortThread => {
                return;
            }
            request => panic!("Received unexpected request {:?}", request),
        }
    }
}

impl Drop for Searcher {
    fn drop(&mut self) {
        for tx in &self.txs {
            let _ = tx.send(Request::AbortThread);
        }
    }
}

impl Searcher {
    pub fn go(&mut self, board: &Board) {
        self.board = board.clone();

        for tx in &self.txs {
            tx.send(Request::StartSearch(Box::new(board.clone())))
                .unwrap();
        }
    }

    pub fn stop(&mut self) {
        for tx in &self.txs {
            tx.send(Request::AbortSearch).unwrap();
        }
        for rx in &self.rxs {
            match rx.recv().unwrap() {
                Response::StoppedSearch => {}
            }
        }
    }

    pub fn principal_variation(&self) -> Vec<Move> {
        let mut board = self.board.clone();

        let mut pv = vec![];

        let mut key_set = HashSet::new();

        let mut adjust = 1;

        'find_pv: loop {
            // Negate score based on who is playing
            adjust *= -1;

            let moves: Vec<Move> = board.moves().collect();

            let mut best_move = None;
            let mut best_score = LOW_SCORE;

            for mov in moves {
                board.make_move(mov);
                let key = board.key();
                board.unmake_move(mov);

                // Check for loop
                if !key_set.insert(key) {
                    break 'find_pv;
                }

                match self.transposition_table.get(&key) {
                    Some(entry) => {
                        let score = entry.value * adjust;
                        if entry.node_type == NodeType::PV && score > best_score {
                            best_move = Some(mov);
                            best_score = score;
                        }
                    }
                    _ => {}
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
            if let Some(mov) = board.moves().next() {
                pv.push(mov);
            }
        }

        pv
    }
}

struct ThreadSearcher<'a> {
    board: &'a mut Board,
    transposition_table: &'a Arc<TranspositionTable>,
    rx: &'a Receiver<Request>,
    abort: bool,
}

impl<'a> ThreadSearcher<'a> {
    fn new(
        board: &'a mut Board,
        transposition_table: &'a Arc<TranspositionTable>,
        rx: &'a Receiver<Request>,
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
            log_search!(depth, "start search");
            self.search(depth, LOW_SCORE, HIGH_SCORE);
            log_search!(depth, "end search");
            depth += 1
        }
    }

    // alpha = lower bound for value of child nodes
    // beta = upper bound for value of child nodes
    fn search(&mut self, depth: u32, mut alpha: i32, mut beta: i32) -> i32 {
        log_search!(depth, "search, alpha = {}, beta = {}", alpha, beta);

        let key = self.board.key();

        let alpha_orig = alpha;

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

        if depth == 0 {
            return self.quiesce(alpha, beta);
        }

        let mut moves = self.board.pseudo_legal_moves().peekable();

        if moves.peek().is_none() {
            return 0; // In stalemate, so this is a tie
        }

        let mut value = LOW_SCORE;

        for mov in moves {
            log_search!(depth, "{}:", mov);

            let rec_search = if self.board.make_move(mov) == Some(King) {
                log_search!(depth, "WIN");
                WIN
            } else {
                -self.search(depth - 1, -beta, -alpha)
            };
            self.board.unmake_move(mov);

            if self.should_abort() {
                return 0;
            }

            log_search!(depth, "{} = {}", mov, rec_search);

            value = value.max(rec_search);

            alpha = alpha.max(value);
            if alpha >= beta {
                log_search!(depth, "cut-off, {} >= {}", alpha, beta);
                break;
            }
        }

        let node_type = if value <= alpha_orig {
            NodeType::Cut
        } else if value >= beta {
            NodeType::All
        } else {
            NodeType::PV
        };

        log_search!(depth, "recording {:?} {:?}", value, node_type);

        let entry = Node {
            depth,
            value,
            node_type,
        };

        self.transposition_table.insert(key, entry);

        value
    }

    /// Evaluate how "quiescent" (quiet or stable) a board is.
    ///
    /// The idea is that a board with lots going on is worth investigating more deeply.
    /// This helps prevent the AI picking bad moves because the board "looks" good, even if an important
    /// piece could be taken in the next turn.
    fn quiesce(&mut self, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = self.board.eval();

        if stand_pat >= beta {
            return beta;
        }

        alpha = alpha.min(stand_pat);

        for mov in self.board.capturing_moves() {
            let score = if self.board.make_move(mov) == Some(King) {
                WIN
            } else {
                -self.quiesce(-beta, -alpha)
            };
            self.board.unmake_move(mov);

            if self.should_abort() {
                return 0;
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }

        alpha
    }

    fn should_abort(&mut self) -> bool {
        self.abort = self.abort || self.rx.try_recv() == Ok(Request::AbortSearch);
        self.abort
    }
}
