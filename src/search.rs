mod tree;

use crate::search::tree::SearchTree;
use crate::Board;
use crate::Move;

use std::collections::HashSet;

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::{fs, thread};
use ttable::{Key, Node, NodeType, TranspositionTable};

mod ttable;

const HIGH_SCORE: i32 = std::i32::MAX;
const LOW_SCORE: i32 = -HIGH_SCORE; // Not std::i32::MIN or we get overflows on negation
const WIN: i32 = 1_000_000;

macro_rules! log_search {
    ($searcher:expr, $depth:expr, $($arg:tt)*) => ({
        if cfg!(feature = "log-search") {
            let indent = std::iter::repeat(' ')
                .take(($searcher.max_depth as i16 - $depth as i16) as usize * 2)
                .collect::<String>();
            println!("{}- {}. {}", indent, $depth, format_args!($($arg)*))
        }
    })
}

impl Board {
    fn key(&self) -> Key {
        (*self.bitboards(), self.player(), self.flags())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Request {
    StartSearch {
        board: Box<Board>,
        target_depth: Option<u16>,
    },
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
        // Each table entry is 8 bytes
        const TABLE_SIZE: usize = 20_000_000;

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
            Request::StartSearch {
                mut board,
                target_depth,
            } => {
                let mut searcher =
                    ThreadSearcher::new(&mut board, transposition_table, rx, target_depth);
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
    pub fn go(&mut self, board: &Board, target_depth: Option<u16>) {
        self.board = board.clone();

        for tx in &self.txs {
            let start_search = Request::StartSearch {
                board: Box::new(board.clone()),
                target_depth,
            };
            tx.send(start_search).unwrap();
        }
    }

    pub fn stop(&mut self) {
        for tx in &self.txs {
            tx.send(Request::AbortSearch).unwrap();
        }
        self.wait();
    }

    /// Wait to stop - only call this after `stop()` or if there is a stopping condition!
    pub fn wait(&mut self) {
        for rx in &self.rxs {
            match rx.recv().unwrap() {
                Response::StoppedSearch => {}
            }
        }
    }

    pub fn principal_variation(&self) -> Vec<Move> {
        let mut board = self.board.clone();

        if cfg!(feature = "log-search2") {
            let tree = SearchTree::from_table(&mut board, &self.transposition_table);
            println!("Rebuilding search tree");
            println!("Dumping to file search-tree.json");
            fs::write(
                "search-tree.json",
                serde_json::to_string_pretty(&tree).unwrap(),
            )
            .unwrap();
        }

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
                let pmov = board.make_move(mov);
                let key = board.key();
                board.unmake_move(pmov);

                // Check for loop
                if !key_set.insert(key) {
                    break 'find_pv;
                }

                if let Some(entry) = self.transposition_table.get(&key) {
                    let score = entry.value * adjust;
                    if entry.node_type == NodeType::PV && score > best_score {
                        best_move = Some(mov);
                        best_score = score;
                    }
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
    target_depth: u16,
    max_depth: u16,
}

impl<'a> ThreadSearcher<'a> {
    fn new(
        board: &'a mut Board,
        transposition_table: &'a Arc<TranspositionTable>,
        rx: &'a Receiver<Request>,
        target_depth: Option<u16>,
    ) -> Self {
        Self {
            board,
            transposition_table,
            rx,
            abort: false,
            target_depth: target_depth.unwrap_or(u16::MAX),
            max_depth: 0,
        }
    }

    fn run(&mut self) {
        self.max_depth = 1;
        log_search!(self, self.max_depth, "start search");

        while !self.should_abort() {
            log_search!(self, self.max_depth, "search at depth");
            self.search(self.max_depth, LOW_SCORE, HIGH_SCORE);
            self.max_depth += 1;
        }

        log_search!(self, self.max_depth, "end search");
    }

    // alpha = lower bound for value of child nodes
    // beta = upper bound for value of child nodes
    fn search(&mut self, depth: u16, mut alpha: i32, mut beta: i32) -> i32 {
        log_search!(self, depth, "search, alpha = {}, beta = {}", alpha, beta);

        let key = self.board.key();

        let alpha_orig = alpha;

        if let Some(entry) = self.transposition_table.get(&key) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::PV => {
                        return entry.value;
                    }
                    NodeType::Cut => {
                        alpha = alpha.max(entry.value);
                    }
                    NodeType::All => {
                        beta = beta.min(entry.value);
                    }
                }

                if alpha >= beta {
                    return entry.value;
                }
            }
        }

        if depth == 0 {
            return self.quiesce(alpha, beta, 0);
        }

        let moves: Vec<Move> = self.board.moves().collect();

        let mut value = if moves.is_empty() {
            if self.board.checkmate() {
                -WIN
            } else {
                0 // In stalemate, so this is a tie
            }
        } else {
            LOW_SCORE
        };

        for mov in moves {
            log_search!(self, depth, "{}:", mov);

            // Evaluate value of move for current player
            let pmov = self.board.make_move(mov);
            let mov_value = -self.search(
                depth - 1,
                // If our maximum possible score is `x`, then the opponent is guaranteed to
                // score at least `-x`
                -beta,
                // If we're guaranteed a score of `y` already, then the opponent can't possibly
                // get more than `-y`
                -alpha,
            );
            self.board.unmake_move(pmov);

            if self.should_abort() {
                return 0;
            }

            log_search!(self, depth, "{} = {}", mov, mov_value);

            value = value.max(mov_value);

            // If value exceeds the old lower-bound, then we can increase the lower-bound
            alpha = alpha.max(value);

            if alpha >= beta {
                // We've found a move that reaches our upper-bound, so no point searching further
                log_search!(self, depth, "cut-off, {} >= {}", alpha, beta);
                break;
            }
        }

        let node_type = if value <= alpha_orig {
            NodeType::All
        } else if value >= beta {
            NodeType::Cut
        } else {
            NodeType::PV
        };

        log_search!(self, depth, "recording {:?} {:?}", value, node_type);

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
    fn quiesce(&mut self, mut alpha: i32, beta: i32, depth: i16) -> i32 {
        // hard cut-off to depth of quiescent search
        if depth <= -1 {
            log_search!(self, depth, "woah that's deep enough");
            return self.board.eval();
        }

        let moves: Vec<Move>;

        if !self.board.check(self.board.player()) {
            // "standing pat" is a heuristic based on current board state.
            // It's assumed that there is always some move that will improve our position, so we use
            // it as our lower-bound.
            let stand_pat = self.board.eval();

            log_search!(
                self,
                depth,
                "quiesce: pat={}, alpha={}, beta={}",
                stand_pat,
                alpha,
                beta
            );

            if stand_pat >= beta {
                return beta;
            }

            alpha = alpha.max(stand_pat);
            moves = self.board.capturing_moves().collect();
        } else {
            // We don't want to use the "standing pat" if we're in check, because it may well be
            // that ANY move is worse than the current state.
            log_search!(
                self,
                depth,
                "quiesce: check, alpha={}, beta={}",
                alpha,
                beta
            );
            // When in check, assess all moves that get out of check, not just captures
            moves = self.board.moves().collect();

            if moves.is_empty() {
                return -WIN;
            }
        }

        for mov in moves {
            log_search!(self, depth, "trying {}", mov);
            let pmov = self.board.make_move(mov);
            let mov_value = -self.quiesce(-beta, -alpha, depth - 1);
            self.board.unmake_move(pmov);

            log_search!(self, depth, "{} = {}", mov, mov_value);

            if self.should_abort() {
                return 0;
            }

            alpha = alpha.max(mov_value);
            if alpha >= beta {
                break;
            }
        }

        alpha
    }

    fn should_abort(&mut self) -> bool {
        self.abort = self.abort
            || self.max_depth > self.target_depth
            || self.rx.try_recv() == Ok(Request::AbortSearch);
        self.abort
    }
}
