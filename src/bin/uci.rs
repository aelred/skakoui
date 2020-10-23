use anyhow::anyhow;
use skakoui::{Board, Move, PlayerV, Searcher};
use std::error::Error;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::str::FromStr;
use std::time::Duration;
use Command::{Go, IsReady, PonderHit, Position, Quit, Stop};
use Info::PV;
use Message::{BestMove, ReadyOK, UCIOK};
use OptionType::{Button, Check, Combo, Spin, String};
use ID::{Author, Name};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let reader = BufReader::new(stdin.lock());

    run(reader, &mut stdout.lock())?;
    Ok(())
}

struct UCI<W> {
    output: W,
    board: Board,
    ponder: Option<Move>,
    searcher: Searcher,
}

impl<W: Write> UCI<W> {
    fn run(&mut self, input: impl BufRead) -> Result<(), std::io::Error> {
        for try_line in input.lines() {
            let line = try_line?;
            eprintln!("{}", line);

            let command = match line.parse::<Command>() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };

            match command {
                Command::UCI => {
                    self.send(Message::ID(Name, "skakoui"))?;
                    self.send(Message::ID(Author, "Felix Chapman"))?;
                    self.send(Message::option("Ponder", Check, "true"))?;
                    self.send(UCIOK)?;
                }
                IsReady => {
                    self.send(ReadyOK)?;
                }
                Quit => break,
                Position { board, moves } => {
                    if let Some(board) = board {
                        self.board = *board;
                    }

                    for mov in moves {
                        self.board.make_move(mov);
                    }
                }
                PonderHit => {
                    if let Some(ponder) = self.ponder.take() {
                        self.board.make_move(ponder);
                    }
                    self.go();
                }
                Go {
                    movetime,
                    wtime,
                    btime,
                    ponder,
                } => {
                    self.go();

                    if let Some(movetime) = movetime {
                        std::thread::sleep(movetime);
                        self.stop()?;
                    }

                    if !ponder {
                        let clock = match self.board.player() {
                            PlayerV::White => wtime,
                            PlayerV::Black => btime,
                        };

                        if let Some(clock) = clock {
                            let max_wait = Duration::from_secs(5);
                            // Naively assume there's 40 moves to go in the game
                            std::thread::sleep((clock / 40).min(max_wait));
                            self.stop()?;
                        }
                    }
                }
                Stop => {
                    self.stop()?;
                }
            }
        }

        Ok(())
    }

    fn send(&mut self, message: Message) -> Result<(), std::io::Error> {
        writeln!(self.output, "{}", message)
    }

    fn go(&mut self) {
        self.searcher.go(&self.board, None);
    }

    fn stop(&mut self) -> Result<(), std::io::Error> {
        self.searcher.stop();
        let pv = self.searcher.principal_variation(&mut self.board);
        let mov = pv.first().copied();
        let ponder = pv.get(1).copied();

        self.send(Message::Info(vec![PV(pv)]))?;
        self.send(BestMove { mov, ponder })?;

        self.ponder = ponder;
        Ok(())
    }
}

enum Command {
    UCI,
    IsReady,
    Quit,
    Position {
        board: Option<Box<Board>>,
        moves: Vec<Move>,
    },
    PonderHit,
    Go {
        movetime: Option<Duration>,
        wtime: Option<Duration>,
        btime: Option<Duration>,
        ponder: bool,
    },
    Stop,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut args = s.split_whitespace().peekable();
        let command = args.next().ok_or_else(|| anyhow!("No command name"))?;

        let command = match command {
            "uci" => Command::UCI,
            "isready" => IsReady,
            "quit" => Quit,
            "position" => {
                let board = match args.peek() {
                    Some(&"startpos") => {
                        args.next();
                        Some(Box::new(Board::default()))
                    }
                    Some(&"fen") => {
                        args.next();
                        let fen = args.clone().take(6).collect::<Vec<&str>>().join(" ");
                        args.nth(5);
                        Some(Box::new(Board::from_fen(fen)?))
                    }
                    _ => None,
                };

                let mut moves = vec![];

                if args.peek() == Some(&&"moves") {
                    args.next();
                    for arg in args {
                        moves.push(Move::from_str(arg)?);
                    }
                }

                Position { board, moves }
            }
            "ponderhit" => PonderHit,
            "go" => {
                let mut movetime = None;
                let mut wtime = None;
                let mut btime = None;
                let mut ponder = false;

                while let Some(arg) = args.next() {
                    match arg {
                        "movetime" => {
                            movetime.replace(read_duration(&mut args)?);
                        }
                        "wtime" => {
                            wtime.replace(read_duration(&mut args)?);
                        }
                        "btime" => {
                            btime.replace(read_duration(&mut args)?);
                        }
                        "ponder" => {
                            ponder = true;
                        }
                        arg => eprintln!("Unrecognised arg: {}", arg),
                    }
                }

                Go {
                    movetime,
                    wtime,
                    btime,
                    ponder,
                }
            }
            "stop" => Stop,
            _ => return Err(anyhow!("Unrecognised command {}", command)),
        };

        Ok(command)
    }
}

enum Message<'a> {
    ID(ID, &'a str),
    UCIOK,
    ReadyOK,
    BestMove {
        mov: Option<Move>,
        ponder: Option<Move>,
    },
    Info(Vec<Info>),
    Option {
        name: &'a str,
        typ: OptionType,
        default: &'a str,
    },
}

impl<'a> Message<'a> {
    fn option(name: &'a str, typ: OptionType, default: &'a str) -> Self {
        Message::Option { name, typ, default }
    }
}

impl<'a> fmt::Display for Message<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::ID(id, value) => write!(f, "{} {}", id, value)?,
            UCIOK => write!(f, "uciok")?,
            ReadyOK => write!(f, "readyok")?,
            BestMove { mov, ponder } => {
                write!(f, "bestmove ")?;
                match mov {
                    None => write!(f, "0000"),
                    Some(mov) => write!(f, "{}", mov),
                }?;
                if let Some(ponder) = ponder {
                    write!(f, " {}", ponder)?;
                }
            }
            Message::Info(info) => {
                for i in info {
                    write!(f, "{} ", i)?;
                }
            }
            Message::Option { name, typ, default } => {
                write!(f, "option name {} type {} default {}", name, typ, default)?;
            }
        }
        Ok(())
    }
}

enum ID {
    Name,
    Author,
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Name => write!(f, "name"),
            Author => write!(f, "author"),
        }
    }
}

enum Info {
    PV(Vec<Move>),
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PV(moves) => {
                for mov in moves {
                    write!(f, "{} ", mov)?;
                }
            }
        }
        Ok(())
    }
}

enum OptionType {
    Check,
    Spin,
    Combo,
    Button,
    String,
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check => write!(f, "check"),
            Spin => write!(f, "spin"),
            Combo => write!(f, "combo"),
            Button => write!(f, "button"),
            String => write!(f, "string"),
        }
    }
}

fn read_duration<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<Duration, anyhow::Error> {
    let value = args.next().ok_or_else(|| anyhow!("No argument value"))?;
    Ok(Duration::from_millis(value.parse::<u64>()?))
}

fn run<R: BufRead, W: Write>(input: R, output: &mut W) -> Result<(), std::io::Error> {
    UCI {
        output,
        board: Board::default(),
        ponder: None,
        searcher: Searcher::default(),
    }
    .run(input)
}

#[cfg(test)]
#[cfg(feature = "expensive-test")]
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
        assert_that(&output_from(&["uci"])).contains("id name skakoui".to_string());
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
            "position fen 7k/8/8/8/8/8/8/K7 w - - 0 1",
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
