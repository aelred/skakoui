mod tree;

use crate::{typed_player, Board, Move, Player};
use arrayvec::ArrayVec;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::{fs, thread};
use tree::SearchTree;
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
        (
            *self.piece_boards(),
            *self.player_boards(),
            self.player(),
            self.flags(),
        )
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
        Self::new(num_threads(), TABLE_SIZE)
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
    pub fn new(num_threads: u32, table_size: usize) -> Self {
        let transposition_table = Arc::new(TranspositionTable::new(table_size));
        let mut txs = vec![];
        let mut rxs = vec![];

        for _ in 0..num_threads {
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
        }
    }

    pub fn go(&mut self, board: &Board, target_depth: Option<u16>) {
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

    pub fn principal_variation(&mut self, board: &mut Board) -> Vec<Move> {
        let pv = self.transposition_table.principal_variation(board);

        if cfg!(feature = "log-search2") {
            println!("Rebuilding search tree");
            let tree =
                SearchTree::from_table(board, &self.transposition_table, pv.len() as u16 + 1);
            println!("Dumping to file search-tree.json");
            fs::write(
                "search-tree.json",
                serde_json::to_string_pretty(&tree).unwrap(),
            )
            .unwrap();
        }

        pv
    }
}

// Maximum PV length to store. Depth 20 is grandmaster-level play.
// We can search deeper but won't have access to the PV to do efficient cut-off.
const MAX_PV: usize = 32;

struct ThreadSearcher<'a> {
    board: &'a mut Board,
    transposition_table: &'a Arc<TranspositionTable>,
    rx: &'a Receiver<Request>,
    abort: bool,
    target_depth: u16,
    max_depth: u16,
    principal_variation: ArrayVec<[Move; MAX_PV]>,
    // true while we are searching the left-most tree (i.e. the principal variation)
    leftmost: bool,
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
            principal_variation: ArrayVec::new(),
            leftmost: true,
        }
    }

    fn run(&mut self) {
        self.max_depth = 1;
        log_search!(self, self.max_depth, "start search");

        while !self.should_abort() {
            log_search!(self, self.max_depth, "search at depth");

            self.leftmost = true;

            typed_player!(self.board.player(), |p| self.search(
                p,
                self.max_depth,
                LOW_SCORE,
                HIGH_SCORE
            ));

            let pv = self
                .transposition_table
                .principal_variation(&mut self.board);
            self.principal_variation.clear();
            self.principal_variation.extend(pv);

            self.max_depth += 1;
        }

        log_search!(self, self.max_depth, "end search");
    }

    // alpha = lower bound for value of child nodes
    // beta = upper bound for value of child nodes
    fn search(&mut self, player: impl Player, depth: u16, mut alpha: i32, mut beta: i32) -> i32 {
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
            return self.quiesce(player, alpha, beta, 0);
        }

        let mut value = LOW_SCORE;

        // Try the PV if we're still searching the left-most tree
        let pv = self
            .principal_variation
            .get((self.max_depth - depth) as usize)
            .filter(|_| self.leftmost)
            .copied();
        let other_moves = self
            .board
            .pseudo_legal_moves_for(player)
            .filter(|mov| pv != Some(*mov));

        for mov in pv.into_iter().chain(other_moves) {
            let pmov = match self.board.make_if_legal(mov) {
                None => continue,
                Some(pmov) => pmov,
            };

            log_search!(self, depth, "{}:", mov);

            // Evaluate value of move for current player
            let mov_value = -self.search(
                player.opponent(),
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

            self.leftmost = false;
        }

        let no_legal_moves = value == LOW_SCORE;
        if no_legal_moves {
            value = if self.board.checkmate() {
                -WIN
            } else {
                0 // In stalemate, so this is a tie
            };
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
    fn quiesce(&mut self, player: impl Player, mut alpha: i32, beta: i32, depth: i16) -> i32 {
        // hard cut-off to depth of quiescent search
        if depth <= -1 {
            log_search!(self, depth, "woah that's deep enough");
            return self.board.eval();
        }

        let moves: Vec<Move>;

        if self.board.check(player) {
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
            moves = self.board.pseudo_legal_moves_for(player).collect();
        } else {
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
            moves = self.board.capturing_moves(player).collect();
        }

        let mut no_legal_moves = true;

        for mov in moves {
            let pmov = match self.board.make_if_legal(mov) {
                None => continue,
                Some(pmov) => pmov,
            };
            no_legal_moves = false;

            log_search!(self, depth, "trying {}", mov);
            let mov_value = -self.quiesce(player.opponent(), -beta, -alpha, depth - 1);
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

        if no_legal_moves {
            -WIN
        } else {
            alpha
        }
    }

    fn should_abort(&mut self) -> bool {
        self.abort = self.abort
            || self.max_depth > self.target_depth
            || self.rx.try_recv() == Ok(Request::AbortSearch);
        self.abort
    }
}
