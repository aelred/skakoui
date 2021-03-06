use crate::move_generation::bishop::Bishop;
use crate::move_generation::rook::Rook;
use crate::{
    bitboards,
    bitboards::{ANTIDIAGONALS, DIAGONALS, FILES, RANKS},
    Bitboard, File, Rank, Square, SquareMap,
};
use lazy_static::lazy_static;

pub fn rook_moves(square: Square, occupancy: Bitboard) -> Bitboard {
    Rook.magic_moves(square, occupancy)
}

pub fn bishop_moves(square: Square, occupancy: Bitboard) -> Bitboard {
    Bishop.magic_moves(square, occupancy)
}

pub trait Magic {
    fn magic_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        self.attacks_array(square)[self.index(square, occupancy)]
    }

    fn index(&self, square: Square, occupancy: Bitboard) -> usize {
        let targets = self.mask(square);
        let (index_bits, magic) = self.magic(square);
        transform(targets & occupancy, magic, index_bits)
    }

    fn find_magic(&self, square: Square, bits: Option<u8>, tries: u64) -> Option<u64> {
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
                return Some(magic);
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
    (12, 0x0100104021008000),
    (11, 0x0100108020400108),
    (11, 0x0200084320801201),
    (11, 0x020004c028306200),
    (11, 0x2200020008200410),
    (11, 0x0300010008040022),
    (11, 0x0080020001000080),
    (12, 0x0600020020910844),
    (11, 0x0201002040800100),
    (10, 0x0010402010004000),
    (10, 0x0404802000100089),
    (10, 0x0684801000080080),
    (10, 0x0002800400800800),
    (10, 0x210a000802005004),
    (10, 0x8000800100020080),
    (11, 0x0002001090410204),
    (11, 0x0040008000802050),
    (10, 0x0970004016200040),
    (10, 0x4220018020801008),
    (10, 0x0008808010000801),
    (10, 0x8800828004000800),
    (10, 0x2182008002800400),
    (10, 0x2416004040010080),
    (11, 0x00000200040040a1),
    (11, 0x0080004140022000),
    (10, 0x0240008080200040),
    (10, 0x0020110100200840),
    (10, 0x8200100080080082),
    (10, 0x0181040180080080),
    (10, 0x4800040080020080),
    (10, 0x4843000100040200),
    (11, 0x8001904200108401),
    (11, 0x8038400420800088),
    (10, 0x4200200080804009),
    (10, 0x9600441501002000),
    (10, 0x0300100080800801),
    (10, 0x2104820400800800),
    (10, 0x00001020080104c0),
    (10, 0x0000010804001002),
    (11, 0x1c82010486000854),
    (11, 0x0000800040008022),
    (10, 0x0002200050014002),
    (10, 0x00004500a0070011),
    (10, 0x8010000800808010),
    (10, 0x0000880100050010),
    (10, 0x4020020004008080),
    (10, 0x1000040200010100),
    (11, 0x1200040a4c820023),
    (11, 0xa221008000443500),
    (10, 0x0481004088220200),
    (10, 0x0080200010008480),
    (10, 0x0005811002080080),
    (10, 0x0000800800040080),
    (10, 0x44a0020004008080),
    (10, 0x0000021028410400),
    (11, 0x0000004400810200),
    (12, 0x000a820040132302),
    (11, 0x0040008021001041),
    (11, 0x0020410020000811),
    (11, 0x5400090020041001),
    (11, 0x0011000408000211),
    (11, 0x0019000208040001),
    (11, 0x10080118d0020804),
    (12, 0x001014068100e042),
]);

const BISHOP_MAGICS: SquareMap<(u8, u64)> = SquareMap::new([
    (6, 0x0564048884030a00),
    (5, 0x0024414832008000),
    (5, 0x10104080910a0000),
    (5, 0x0004504600000800),
    (5, 0x1084042000020008),
    (5, 0x0901100210200228),
    (5, 0x44050801100a0002),
    (6, 0x0410904c04200880),
    (5, 0x09000604100a0204),
    (5, 0x0040820801010e00),
    (5, 0x001004c104010001),
    (5, 0x8800090405014000),
    (5, 0x0042040421400400),
    (5, 0x2008008220200004),
    (5, 0x0201004a10646060),
    (5, 0x0a002a07a0880810),
    (5, 0x011040090210040c),
    (5, 0x0904000810040070),
    (7, 0x80a0800101040280),
    (7, 0x2050202208220008),
    (7, 0x2048100101400040),
    (7, 0x00b4400088201008),
    (5, 0x5004000229040200),
    (5, 0x08010003e1280220),
    (5, 0x0004221004200400),
    (5, 0x0102028210100206),
    (7, 0x2244820150040110),
    (9, 0x9004040188020808),
    (9, 0x0820848044002000),
    (7, 0x1008004080806018),
    (5, 0x00e2120004010100),
    (5, 0x9004010000806115),
    (5, 0x00101c9013202288),
    (5, 0x0052106440d08112),
    (7, 0x4000405000084400),
    (9, 0x0002020080080080),
    (9, 0x9042008400020020),
    (7, 0x0090210202004042),
    (5, 0x0011080110048400),
    (5, 0x6a08092d00104040),
    (5, 0x0408020221011010),
    (5, 0x0102110120000800),
    (7, 0x002100413a019000),
    (7, 0x8000010401000820),
    (7, 0x0290481014000040),
    (7, 0x0010011000241101),
    (5, 0x8004500202030043),
    (5, 0x8092208401090590),
    (5, 0x0014210422201001),
    (5, 0x0000421090080002),
    (5, 0x0201011088902001),
    (5, 0x0088002020880080),
    (5, 0x0400601142020001),
    (5, 0x44140a3010008808),
    (5, 0x80085001020400e4),
    (5, 0x0409020414002108),
    (6, 0x8000110090100802),
    (5, 0x0000020084040220),
    (5, 0x0240500040441000),
    (5, 0x0022080020420200),
    (5, 0x040c000020120482),
    (5, 0x0009040408108100),
    (5, 0x040022081001004a),
    (6, 0x0010010800840a40),
]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard;

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

        assert_eq!(expected, rook_moves(Square::D4, occupancy));
    }

    #[test]
    fn rook_magics_are_correct() {
        for _ in 0..1000 {
            let occupancy = Bitboard::new(rand::random());

            for square in Square::all() {
                let calculated = Rook.calc_moves(square, occupancy);
                let magicked = rook_moves(square, occupancy);
                assert_eq!(
                    calculated, magicked,
                    "Occupancy: {:?}\nSquare: {}",
                    occupancy, square
                );
            }
        }
    }

    #[test]
    fn bishop_magics_are_correct() {
        for _ in 0..1000 {
            let occupancy = Bitboard::new(rand::random());

            for square in Square::all() {
                let calculated = Bishop.calc_moves(square, occupancy);
                let magicked = bishop_moves(square, occupancy);
                assert_eq!(
                    calculated, magicked,
                    "Occupancy: {:?}\nSquare: {}",
                    occupancy, square
                );
            }
        }
    }
}
