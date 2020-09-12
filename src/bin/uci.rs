use chess::{Board, Move, Searcher};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::str::FromStr;
use tee::TeeReader;

fn main() -> Result<(), Box<dyn Error>> {
    let log_file = File::create("/Users/aelred/Desktop/chess.log")?;

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let reader = BufReader::new(TeeReader::new(stdin.lock(), log_file));

    run(reader, &mut stdout.lock())?;
    Ok(())
}

fn run<R: BufRead, W: Write>(input: R, output: &mut W) -> Result<(), std::io::Error> {
    let mut searcher = Searcher::default();
    let mut board = Board::default();

    for try_line in input.lines() {
        let line = try_line?;
        let words: Vec<&str> = line.split_whitespace().collect();
        let command = words[0];
        let mut args = &words[1..];

        match command {
            "uci" => {
                writeln!(output, "id name skaki")?;
                writeln!(output, "id author Felix Chapman")?;
                writeln!(output, "uciok")?;
            }
            "isready" => {
                writeln!(output, "readyok")?;
            }
            "quit" => break,
            "position" => {
                if args.get(0) == Some(&"startpos") {
                    board = Board::default();
                    args = &args[1..];
                } else if args.get(0) == Some(&"fen") {
                    board = Board::from_fen(args[1..7].join(" ")).unwrap();
                    args = &args[7..];
                }

                if args.get(0) == Some(&"moves") {
                    for arg in &args[1..] {
                        let mov = Move::from_str(arg).unwrap();
                        board.make_move(mov);
                    }
                }
            }
            "go" => {
                searcher.go(&mut board);
            }
            "stop" => {
                let (mov, _) = searcher.stop();
                let mov_str = mov
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "0000".to_string());
                writeln!(output, "bestmove {}", mov_str)?;
            }
            _ => (), // Ignore unknown commands
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::borrow::Borrow;
    use std::io::BufReader;

    #[test]
    fn when_no_input_then_no_output() {
        assert_that(&output_from(&[])).is_empty();
    }

    #[test]
    fn when_input_uci_then_output_name() {
        assert_that(&output_from(&["uci"])).contains("id name skaki".to_string());
    }

    #[test]
    fn when_input_uci_then_output_author() {
        assert_that(&output_from(&["uci"])).contains("id author Felix Chapman".to_string());
    }

    #[test]
    fn when_input_uci_then_output_uciok() {
        assert_that(&output_from(&["uci"])).contains("uciok".to_string());
    }

    #[test]
    fn when_input_isready_then_output_readyok() {
        assert_that(&output_from(&["isready"])).contains("readyok".to_string());
    }

    #[test]
    fn when_input_quit_then_stop_and_ignore_later_commands() {
        assert_that(&output_from(&["quit", "isready"])).does_not_contain("readyok".to_string());
    }

    #[test]
    fn when_input_go_stop_then_return_a_valid_white_opening_move() {
        assert_that(&output_from(&["uci", "go", "stop"]))
            .matching_contains(|out| white_openings().contains(&out.borrow()))
    }

    #[test]
    fn when_input_position_moves_then_return_a_valid_move_from_that_position() {
        assert_that(&output_from(&["uci", "position moves a2a3", "go", "stop"]))
            .matching_contains(|out| black_openings().contains(&out.borrow()))
    }

    #[test]
    fn when_input_position_startpos_then_reset_the_board() {
        assert_that(&output_from(&[
            "uci",
            "position moves a2a3",
            "position startpos",
            "go",
            "stop",
        ]))
        .matching_contains(|out| white_openings().contains(&out))
    }

    #[test]
    fn when_input_position_fen_then_set_board_as_specified() {
        let valid_moves = vec![
            "bestmove a1a2".to_string(),
            "bestmove a1b1".to_string(),
            "bestmove a1b2".to_string(),
        ];

        assert_that(&output_from(&[
            "uci",
            "position fen 7k/8/8/8/8/8/8/K7 w KQkq - 0 1",
            "go",
            "stop",
        ]))
        .matching_contains(|out| valid_moves.contains(&out))
    }

    fn white_openings() -> Vec<String> {
        let moves = vec![
            "a2a3", "a2a4", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4", "e2e3", "e2e4", "f2f3",
            "f2f4", "g2g3", "g2g4", "h2h3", "h2h4", "b1a3", "b1c3", "g1f3", "g1h3",
        ];
        moves
            .into_iter()
            .map(|m| format!("bestmove {}", m))
            .collect()
    }

    fn black_openings() -> Vec<String> {
        let moves = vec![
            "a7a6", "a7a5", "b7b6", "b7b5", "c7c6", "c7c5", "d7d6", "d7d5", "e7e6", "e7e5", "f7f6",
            "f7f5", "g7g6", "g7g5", "h7h6", "h7h5", "b8a6", "b8c6", "g8f6", "g8h6",
        ];
        moves
            .into_iter()
            .map(|m| format!("bestmove {}", m))
            .collect()
    }

    fn output_from(input: &[&str]) -> Vec<String> {
        let input_strs: Vec<String> = input.into_iter().map(|x| x.to_string()).collect();
        let input_str = input_strs.join("\n");
        let reader = BufReader::new(input_str.as_bytes());
        let mut output: Vec<u8> = Vec::new();
        run(reader, &mut output).unwrap();
        let output_str = String::from_utf8_lossy(&output);
        output_str.lines().map(|x| x.to_string()).collect()
    }
}
