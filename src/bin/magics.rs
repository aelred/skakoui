use skakoui::magic::MagicPiece;
use skakoui::{magic, Square};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "magics", about = "Generate magic numbers")]
struct Opt {
    piece: MagicPiece,
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

    let piece = opt.piece;
    let bits = opt.bits;
    let tries = opt.tries;

    for square in squares {
        let result = magic::find_magic(piece, square, bits, tries);
        let magic = result.expect("Couldn't find a magic!");
        println!("{}: {:#x?}", square, magic);
    }
}
