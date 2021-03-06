use skakoui::magic::Magic;
use skakoui::{Bishop, Rook, Square};
use std::borrow::Borrow;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "magics", about = "Generate magic numbers")]
struct Opt {
    piece: String,
    squares: Vec<Square>,
    #[structopt(long, short)]
    bits: Option<u8>,
    #[structopt(long, short, default_value = "100000000")]
    tries: u64,
}

fn main() {
    let opt: Opt = Opt::from_args();

    let squares = if opt.squares.is_empty() {
        Square::all().collect::<Vec<Square>>()
    } else {
        opt.squares
    };

    let bishop = match opt.piece.borrow() {
        "bishop" => true,
        "rook" => false,
        s => panic!("Piece should be either 'bishop' or 'rook', not '{}'", s),
    };

    let bits = opt.bits;
    let tries = opt.tries;

    for square in squares {
        let result = if bishop {
            Bishop.find_magic(square, bits, tries)
        } else {
            Rook.find_magic(square, bits, tries)
        };
        let magic = result.expect("Couldn't find a magic!");
        println!("{}: {:#x?}", square, magic);
    }
}
