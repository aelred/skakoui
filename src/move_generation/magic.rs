use crate::move_generation::bishop::Bishop;
use crate::move_generation::rook::Rook;
use crate::{
    bitboards,
    bitboards::{ANTIDIAGONALS, DIAGONALS, FILES, RANKS},
    Bitboard, File, Rank, Square, SquareMap,
};
use lazy_static::lazy_static;

pub trait Magic {
    fn magic_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        self.attacks_array(square)[self.index(square, occupancy)]
    }

    fn index(&self, square: Square, occupancy: Bitboard) -> usize {
        let targets = self.mask(square);
        let (index_bits, magic) = self.magic(square);
        transform(targets & occupancy, magic, index_bits)
    }

    fn find_magic(&self, square: Square, bits: Option<u8>, tries: u64) -> Option<(u8, u64)> {
        let mask = self.mask(square);
        let bits = bits.unwrap_or_else(|| mask.count());

        let occupancy_moves: Vec<(Bitboard, Bitboard)> = mask
            .powerset()
            .map(|occupancy| (occupancy, self.calc_moves(square, occupancy)))
            .collect();

        for _ in 0..tries {
            // Small number of zeroes -> better magics
            let magic = rand::random::<u64>() & rand::random::<u64>() & rand::random::<u64>();

            if valid_magic(magic, mask, &occupancy_moves, bits) {
                return Some((bits, magic));
            }
        }

        None
    }

    fn mask(&self, square: Square) -> Bitboard;
    fn attacks_array(&self, square: Square) -> &[Bitboard; 0x10000];
    fn magic(&self, square: Square) -> (u8, u64);
    fn calc_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard;
}

impl Magic for Bishop {
    fn mask(&self, square: Square) -> Bitboard {
        BISHOP_TARGETS[square]
    }

    fn attacks_array(&self, square: Square) -> &[Bitboard; 0x10000] {
        &BISHOP_ATTACKS[square]
    }

    fn magic(&self, square: Square) -> (u8, u64) {
        BISHOP_MAGICS[square]
    }

    fn calc_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        Diagonal.slide(square, occupancy) | AntiDiagonal.slide(square, occupancy)
    }
}

impl Magic for Rook {
    fn mask(&self, square: Square) -> Bitboard {
        ROOK_TARGETS[square]
    }

    fn attacks_array(&self, square: Square) -> &[Bitboard; 0x10000] {
        &ROOK_ATTACKS[square]
    }

    fn magic(&self, square: Square) -> (u8, u64) {
        ROOK_MAGICS[square]
    }

    fn calc_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        NorthSouth.slide(square, occupancy) | EastWest.slide(square, occupancy)
    }
}

fn transform(occupied: Bitboard, magic: u64, index_bits: u8) -> usize {
    (u64::from(occupied)
        .wrapping_mul(magic)
        .wrapping_shr(64 - index_bits as u32)) as usize
}

fn valid_magic(
    magic: u64,
    mask: Bitboard,
    occupancy_moves: &[(Bitboard, Bitboard)],
    bits: u8,
) -> bool {
    // not sure what this check is for...
    if (u64::from(mask).wrapping_mul(magic) & 0xFF00000000000000).count_ones() < 6 {
        return false;
    }

    let mut used: [Option<Bitboard>; 0x10000] = [None; 0x10000];

    for (occupancy, moves) in occupancy_moves {
        let index = transform(*occupancy, magic, bits);
        if used[index].get_or_insert(*moves) != moves {
            // Indices clash
            return false;
        }
    }

    true
}

trait SlideDirection {
    fn bitboards(&self) -> (&SquareMap<Bitboard>, &SquareMap<Bitboard>);

    /// Slide a piece from the source square in the given direction.
    fn slide(&self, source: Square, occupancy: Bitboard) -> Bitboard {
        let (positive_bitboard, negative_bitboard) = self.bitboards();
        let pos_movement = positive_bitboard[source];
        let mut blockers = pos_movement & occupancy;
        // Set the last square so there is always a blocking square (no need to branch)
        blockers.set(Square::H8);
        let blocking_square = blockers.first_set();
        let pos_movement = pos_movement ^ positive_bitboard[blocking_square];

        let neg_movement = negative_bitboard[source];
        let mut blockers = neg_movement & occupancy;
        // Set the last square so there is always a blocking square (no need to branch)
        blockers.set(Square::A1);
        let blocking_square = blockers.last_set();
        let neg_movement = neg_movement ^ negative_bitboard[blocking_square];

        pos_movement | neg_movement
    }
}

struct NorthSouth;
impl SlideDirection for NorthSouth {
    fn bitboards(&self) -> (&SquareMap<Bitboard>, &SquareMap<Bitboard>) {
        (&bitboards::NORTH, &bitboards::SOUTH)
    }
}

struct EastWest;
impl SlideDirection for EastWest {
    fn bitboards(&self) -> (&SquareMap<Bitboard>, &SquareMap<Bitboard>) {
        (&bitboards::EAST, &bitboards::WEST)
    }
}

struct Diagonal;
impl SlideDirection for Diagonal {
    fn bitboards(&self) -> (&SquareMap<Bitboard>, &SquareMap<Bitboard>) {
        (&bitboards::NORTH_EAST, &bitboards::SOUTH_WEST)
    }
}

struct AntiDiagonal;
impl SlideDirection for AntiDiagonal {
    fn bitboards(&self) -> (&SquareMap<Bitboard>, &SquareMap<Bitboard>) {
        (&bitboards::NORTH_WEST, &bitboards::SOUTH_EAST)
    }
}

lazy_static! {
    // ROOK_TARGETS[x] == every square where a piece could block a rook at square x
    static ref ROOK_TARGETS: SquareMap<Bitboard> = SquareMap::from(|sq| {
        let file = FILES[sq.file()] & !RANKS[Rank::_1] & !RANKS[Rank::_8];
        let rank = RANKS[sq.rank()] & !FILES[File::A] & !FILES[File::H];
        (file | rank) & !Bitboard::from(sq)
    });

    // BISHOP_TARGETS[x] == every square where a piece could block a bishop at square x
    static ref BISHOP_TARGETS: SquareMap<Bitboard> = SquareMap::from(|sq| {
        let border = RANKS[Rank::_1] | RANKS[Rank::_8] | FILES[File::A] | FILES[File::H];
        (DIAGONALS[sq] ^ ANTIDIAGONALS[sq]) & !border
    });

    // Attack arrays looked up by indices calculated using **MAGIC**

    static ref ROOK_ATTACKS: SquareMap<Box<[Bitboard; 0x10000]>> =
        SquareMap::from(|square| {
            let mut attacks = Box::new([bitboards::EMPTY; 0x10000]);
            for occupancy in Rook.mask(square).powerset() {
                attacks[Rook.index(square, occupancy)] = Rook.calc_moves(square, occupancy);
            }
            attacks
        });

    // Attack array shared by rooks and bishops, looked up by indices calculated using **MAGIC**
    static ref BISHOP_ATTACKS: SquareMap<Box<[Bitboard; 0x10000]>> =
        SquareMap::from(|square| {
            let mut attacks = Box::new([bitboards::EMPTY; 0x10000]);
            for occupancy in Bishop.mask(square).powerset() {
                attacks[Bishop.index(square, occupancy)] = Bishop.calc_moves(square, occupancy);
            }
            attacks
        });
}

const ROOK_MAGICS: SquareMap<(u8, u64)> = SquareMap::new([
    /*a1*/ (12, 0x1d80102140008000),
    /*b1*/ (11, 0x80c0002000491002),
    /*c1*/ (11, 0x0900082001004010),
    /*d1*/ (11, 0xa080080080100004),
    /*e1*/ (11, 0x0100020411000800),
    /*f1*/ (11, 0x2200080110040200),
    /*g1*/ (11, 0x1880410012001080),
    /*h1*/ (12, 0x4080190000644080),
    /*a2*/ (11, 0x0004800098204000),
    /*b2*/ (10, 0x1802401000402004),
    /*c2*/ (10, 0xc002002a00104080),
    /*d2*/ (10, 0x4088800801801000),
    /*e2*/ (10, 0x1804800400818800),
    /*f2*/ (10, 0x8046804400020080),
    /*g2*/ (10, 0x0084000201048810),
    /*h2*/ (11, 0x0020802841001480),
    /*a3*/ (11, 0x8119020022084080),
    /*b3*/ (10, 0x0000404010002001),
    /*c3*/ (10, 0x5200220010804208),
    /*d3*/ (10, 0x0000090020100100),
    /*e3*/ (10, 0x0800808008000401),
    /*f3*/ (10, 0x0002010100040008),
    /*g3*/ (10, 0x6400040008028110),
    /*h3*/ (11, 0x0010020000408104),
    /*a4*/ (11, 0x4a00400480008020),
    /*b4*/ (10, 0x48c0401080200080),
    /*c4*/ (10, 0x4430008480122002),
    /*d4*/ (10, 0x821d100080080282),
    /*e4*/ (10, 0x5204040080080080),
    /*f4*/ (10, 0x0004010040020040),
    /*g4*/ (10, 0x0804b00400010248),
    /*h4*/ (11, 0x7018802180005300),
    /*a5*/ (11, 0x0180400080800023),
    /*b5*/ (10, 0x006000c020c01000),
    /*c5*/ (10, 0x1801044011002000),
    /*d5*/ (10, 0x0004080480801000),
    /*e5*/ (10, 0x0000800400800800),
    /*f5*/ (10, 0x1882001c06000810),
    /*g5*/ (10, 0x081005504c003802),
    /*h5*/ (11, 0x03000c0042001481),
    /*a6*/ (11, 0x0081400280228000),
    /*b6*/ (10, 0x5000200050004001),
    /*c6*/ (10, 0x0110004020010100),
    /*d6*/ (10, 0x40000a0020120040),
    /*e6*/ (10, 0x8000040008008080),
    /*f6*/ (10, 0x5000a04004080110),
    /*g6*/ (10, 0x0000228810040041),
    /*h6*/ (11, 0x3c8000930542000c),
    /*a7*/ (11, 0x0100408001002500),
    /*b7*/ (10, 0x008c884002200c80),
    /*c7*/ (10, 0x022cc19280220200),
    /*d7*/ (10, 0x0514800800100080),
    /*e7*/ (10, 0x0104028008000480),
    /*f7*/ (10, 0x6009002400022900),
    /*g7*/ (10, 0x0004800200010080),
    /*h7*/ (11, 0x0082140049208200),
    /*a8*/ (12, 0x0205042010800041),
    /*b8*/ (11, 0x104492a501400081),
    /*c8*/ (11, 0x8044804200081022),
    /*d8*/ (11, 0x0442100184200901),
    /*e8*/ (11, 0x0401000800021085),
    /*f8*/ (11, 0x2609000400080201),
    /*g8*/ (11, 0x8000083092011004),
    /*h8*/ (12, 0x8000008302284402),
]);

const BISHOP_MAGICS: SquareMap<(u8, u64)> = SquareMap::new([
    /*a1*/ (6, 0xb00410902d010010),
    /*b1*/ (5, 0x20c421022a220000),
    /*c1*/ (5, 0x1404080081044000),
    /*d1*/ (5, 0x80c9040504004080),
    /*e1*/ (5, 0x0402021040020000),
    /*f1*/ (5, 0x31020a9004090000),
    /*g1*/ (5, 0x2011110110410000),
    /*h1*/ (6, 0x0084820080a00820),
    /*a2*/ (5, 0x0813622001020c88),
    /*b2*/ (5, 0x8181100409040530),
    /*c2*/ (5, 0x0112111142060090),
    /*d2*/ (5, 0x1311444040800000),
    /*e2*/ (5, 0x00000110400082c0),
    /*f2*/ (5, 0x0000410188408081),
    /*g2*/ (5, 0x0000120230040422),
    /*h2*/ (5, 0x0000208428821000),
    /*a3*/ (5, 0x0520401242100104),
    /*b3*/ (5, 0x3048109010008088),
    /*c3*/ (7, 0x0004000884220200),
    /*d3*/ (7, 0x0028000420405204),
    /*e3*/ (7, 0x0142004400940440),
    /*f3*/ (7, 0x1188800808900800),
    /*g3*/ (5, 0x3001000058080401),
    /*h3*/ (5, 0x00020101104d0c24),
    /*a4*/ (5, 0x1004c04020020400),
    /*b4*/ (5, 0x02422045022c8408),
    /*c4*/ (7, 0x80812a0104080a02),
    /*d4*/ (9, 0x0026008208008082),
    /*e4*/ (9, 0x0082840048802010),
    /*f4*/ (7, 0x0010450088900801),
    /*g4*/ (5, 0x00008a0021080200),
    /*h4*/ (5, 0x0181004001084802),
    /*a5*/ (5, 0x4021101000086004),
    /*b5*/ (5, 0x10020110302002a0),
    /*c5*/ (7, 0x0005124400281800),
    /*d5*/ (9, 0x0080020081080080),
    /*e5*/ (9, 0x8010810200040208),
    /*f5*/ (7, 0x0008004500809004),
    /*g5*/ (5, 0x4414010a040c1083),
    /*h5*/ (5, 0x8018020044088042),
    /*a6*/ (5, 0x8401012020009201),
    /*b6*/ (5, 0x0140480410004408),
    /*c6*/ (7, 0x80c1010802002104),
    /*d6*/ (7, 0x4000004200804808),
    /*e6*/ (7, 0x0000602208800408),
    /*f6*/ (7, 0x9108021042000c12),
    /*g6*/ (5, 0x000414088a00b402),
    /*h6*/ (5, 0x1104010208220200),
    /*a7*/ (5, 0x0060410808402004),
    /*b7*/ (5, 0x4002050101110281),
    /*c7*/ (5, 0x200010440404010b),
    /*d7*/ (5, 0x1208000b20a810a0),
    /*e7*/ (5, 0x0200801102020001),
    /*f7*/ (5, 0x0008081001820500),
    /*g7*/ (5, 0x0104111002008420),
    /*h7*/ (5, 0x0010044084004a10),
    /*a8*/ (6, 0x0885008800880500),
    /*b8*/ (5, 0x2080290402014501),
    /*c8*/ (5, 0x0840100200922100),
    /*d8*/ (5, 0x0204400000208800),
    /*e8*/ (5, 0x00000800c0082220),
    /*f8*/ (5, 0x8008204039410100),
    /*g8*/ (5, 0x0280103010808880),
    /*h8*/ (6, 0x0210010808044042),
]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard;
    use crate::strategies::*;
    use proptest::proptest;

    #[test]
    fn rook_moves_works() {
        let occupancy = bitboard! {
            . . . X . . . .
            . . . . . . . .
            . . . . . . X .
            . . X . . . . .
            . . . X X . . .
            . . . . . . . .
            . . . X . . . .
            . . . . . . . .
        };

        let expected = bitboard! {
            . . . X . . . .
            . . . X . . . .
            . . . X . . . .
            . . . X . . . .
            X X X . X . . .
            . . . X . . . .
            . . . X . . . .
            . . . . . . . .
        };

        assert_eq!(expected, Rook.magic_moves(Square::D4, occupancy));
    }

    proptest! {
        #[test]
        fn rook_magics_are_correct(occupancy in arb_bitboard(), square in arb_square()) {
            let calculated = Rook.calc_moves(square, occupancy);
            let magicked = Rook.magic_moves(square, occupancy);
            assert_eq!(
                calculated, magicked,
                "Occupancy: {:?}\nSquare: {}",
                occupancy, square
            );
        }

        #[test]
        fn bishop_magics_are_correct(occupancy in arb_bitboard(), square in arb_square()) {
            let calculated = Bishop.calc_moves(square, occupancy);
            let magicked = Bishop.magic_moves(square, occupancy);
            assert_eq!(
                calculated, magicked,
                "Occupancy: {:?}\nSquare: {}",
                occupancy, square
            );
        }
    }
}
