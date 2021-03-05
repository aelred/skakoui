use crate::move_generation::bishop::Bishop;
use crate::move_generation::rook::Rook;
use crate::{
    bitboards,
    bitboards::{ANTIDIAGONALS, DIAGONALS, FILES, RANKS},
    Bitboard, File, Rank, Square, SquareMap,
};
use lazy_static::lazy_static;
use std::ops::Deref;

pub fn queen_moves(square: Square, occupancy: Bitboard) -> Bitboard {
    rook_moves(square, occupancy) | bishop_moves(square, occupancy)
}

pub fn rook_moves(square: Square, occupancy: Bitboard) -> Bitboard {
    moves(Rook, square, occupancy)
}

pub fn bishop_moves(square: Square, occupancy: Bitboard) -> Bitboard {
    moves(Bishop, square, occupancy)
}

fn moves(piece: impl Magic + Copy, square: Square, occupancy: Bitboard) -> Bitboard {
    piece.attacks(square)[index(piece, square, occupancy)]
}

fn index(piece: impl Magic, square: Square, occupancy: Bitboard) -> usize {
    let targets = piece.mask(square);
    let (magic, index_bits) = piece.magic(square);
    transform(targets & occupancy, magic, index_bits)
}

fn transform(occupied: Bitboard, magic: u64, index_bits: u8) -> usize {
    (u64::from(occupied)
        .wrapping_mul(magic)
        .wrapping_shr(64 - index_bits as u32)) as usize
}

pub trait Magic {
    fn mask(&self, square: Square) -> Bitboard;
    fn attacks(&self, square: Square) -> &[Bitboard; 0x10000];
    fn magic(&self, square: Square) -> (u64, u8);
    fn calc_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard;
}

impl Magic for Bishop {
    fn mask(&self, square: Square) -> Bitboard {
        BISHOP_TARGETS[square]
    }

    fn attacks(&self, square: Square) -> &[Bitboard; 0x10000] {
        &BISHOP_ATTACKS[square]
    }

    fn magic(&self, square: Square) -> (u64, u8) {
        (BISHOP_MAGICS[square], BISHOP_BITS[square])
    }

    fn calc_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        Diagonal.slide(square, occupancy) | AntiDiagonal.slide(square, occupancy)
    }
}

impl Magic for Rook {
    fn mask(&self, square: Square) -> Bitboard {
        ROOK_TARGETS[square]
    }

    fn attacks(&self, square: Square) -> &[Bitboard; 0x10000] {
        &ROOK_ATTACKS[square]
    }

    fn magic(&self, square: Square) -> (u64, u8) {
        (ROOK_MAGICS[square], ROOK_BITS[square])
    }

    fn calc_moves(&self, square: Square, occupancy: Bitboard) -> Bitboard {
        NorthSouth.slide(square, occupancy) | EastWest.slide(square, occupancy)
    }
}

pub fn find_magic(piece: impl Magic, square: Square, bits: Option<u8>, tries: u64) -> Option<u64> {
    let mask = piece.mask(square);
    let bits = bits.unwrap_or_else(|| piece.mask(square).count());

    let occupancy_moves: Vec<(Bitboard, Bitboard)> = mask
        .powerset()
        .map(|occupancy| (occupancy, piece.calc_moves(square, occupancy)))
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

trait SlideDirection: Sized {
    fn bitboards(&self) -> (&SquareMap<Bitboard>, &SquareMap<Bitboard>);

    /// Slide a piece from the source square in the given direction.
    fn slide(self, source: Square, occupancy: Bitboard) -> Bitboard {
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

const ROOK_MAGICS: SquareMap<u64> = SquareMap::new([
    0x100104021008000,
    0x100108020400108,
    0x200084320801201,
    0x20004c028306200,
    0x2200020008200410,
    0x300010008040022,
    0x80020001000080,
    0x600020020910844,
    0x201002040800100,
    0x10402010004000,
    0x404802000100089,
    0x684801000080080,
    0x2800400800800,
    0x210a000802005004,
    0x8000800100020080,
    0x2001090410204,
    0x40008000802050,
    0x970004016200040,
    0x4220018020801008,
    0x8808010000801,
    0x8800828004000800,
    0x2182008002800400,
    0x2416004040010080,
    0x200040040a1,
    0x80004140022000,
    0x240008080200040,
    0x20110100200840,
    0x8200100080080082,
    0x181040180080080,
    0x4800040080020080,
    0x4843000100040200,
    0x8001904200108401,
    0x8038400420800088,
    0x4200200080804009,
    0x9600441501002000,
    0x300100080800801,
    0x2104820400800800,
    0x1020080104c0,
    0x10804001002,
    0x1c82010486000854,
    0x800040008022,
    0x2200050014002,
    0x4500a0070011,
    0x8010000800808010,
    0x880100050010,
    0x4020020004008080,
    0x1000040200010100,
    0x1200040a4c820023,
    0xa221008000443500,
    0x481004088220200,
    0x80200010008480,
    0x5811002080080,
    0x800800040080,
    0x44a0020004008080,
    0x21028410400,
    0x4400810200,
    0xa820040132302,
    0x40008021001041,
    0x20410020000811,
    0x5400090020041001,
    0x11000408000211,
    0x19000208040001,
    0x10080118d0020804,
    0x1014068100e042,
]);

const ROOK_BITS: SquareMap<u8> = SquareMap::new([
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
]);

const BISHOP_MAGICS: SquareMap<u64> = SquareMap::new([
    0x564048884030a00,
    0x24414832008000,
    0x10104080910a0000,
    0x4504600000800,
    0x1084042000020008,
    0x901100210200228,
    0x44050801100a0002,
    0x410904c04200880,
    0x9000604100a0204,
    0x40820801010e00,
    0x1004c104010001,
    0x8800090405014000,
    0x42040421400400,
    0x2008008220200004,
    0x201004a10646060,
    0xa002a07a0880810,
    0x11040090210040c,
    0x904000810040070,
    0x80a0800101040280,
    0x2050202208220008,
    0x2048100101400040,
    0xb4400088201008,
    0x5004000229040200,
    0x8010003e1280220,
    0x4221004200400,
    0x102028210100206,
    0x2244820150040110,
    0x9004040188020808,
    0x820848044002000,
    0x1008004080806018,
    0xe2120004010100,
    0x9004010000806115,
    0x101c9013202288,
    0x52106440d08112,
    0x4000405000084400,
    0x2020080080080,
    0x9042008400020020,
    0x90210202004042,
    0x11080110048400,
    0x6a08092d00104040,
    0x408020221011010,
    0x102110120000800,
    0x2100413a019000,
    0x8000010401000820,
    0x290481014000040,
    0x10011000241101,
    0x8004500202030043,
    0x8092208401090590,
    0x14210422201001,
    0x421090080002,
    0x201011088902001,
    0x88002020880080,
    0x400601142020001,
    0x44140a3010008808,
    0x80085001020400e4,
    0x409020414002108,
    0x8000110090100802,
    0x20084040220,
    0x240500040441000,
    0x22080020420200,
    0x40c000020120482,
    0x9040408108100,
    0x40022081001004a,
    0x10010800840a40,
]);

const BISHOP_BITS: SquareMap<u8> = SquareMap::new([
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
]);

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
            for occupancy in ROOK_TARGETS[square].powerset() {
                attacks[index(Rook, square, occupancy)] = Rook.calc_moves(square, occupancy);
            }
            attacks
        });

    // Attack array shared by rooks and bishops, looked up by indices calculated using **MAGIC**
    static ref BISHOP_ATTACKS: SquareMap<Box<[Bitboard; 0x10000]>> =
        SquareMap::from(|square| {
            let mut attacks = Box::new([bitboards::EMPTY; 0x10000]);
            for occupancy in BISHOP_TARGETS[square].powerset() {
                attacks[index(Bishop, square, occupancy)] = Bishop.calc_moves(square, occupancy);
            }
            attacks
        });
}

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
