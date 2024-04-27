use std::time::Instant;
use crate::types::*;
use crate::ataxx_move::*;
use arrayvec::ArrayVec;

/*
42 43 44 45 46 47 48
35 36 37 38 39 40 41
28 29 30 31 32 33 34
21 22 23 24 25 26 27
14 15 16 17 18 19 20
 7  8  9 10 11 12 13
 0  1  2  3  4  5  6
*/

pub const SQUARE_TO_STR: [&str; 49] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", 
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", 
    "a4", "b4", "c4", "d4", "e4", "f4", "g4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", 
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", 
];

#[allow(dead_code)]
pub fn square_rank(sq: Square) -> Rank {
    unsafe { std::mem::transmute((sq / 7) as u8) }
}

#[allow(dead_code)]
pub fn square_file(sq: Square) -> File {
    unsafe { std::mem::transmute((sq % 7) as u8) }
}


pub fn opp_color(color: Color) -> Color {
    match color {
        Color::Red => Color::Blue,
        Color::Blue => Color::Red,
        Color::None => Color::None
    }
}

pub fn pop_lsb(value: &mut u64) -> u8 {
    assert!(*value > 0);
    let index = value.trailing_zeros() as usize;
    *value &= value.wrapping_sub(1);
    index as u8
}

pub fn char_to_digit(ch: char) -> u8 {
    assert!(ch.is_digit(10));
    ch.to_digit(10).unwrap() as u8
}

pub fn digit_to_char(num: u8) -> char {
    assert!(num <= 9);
    (num + b'0') as char
}

#[allow(dead_code)]
pub fn print_bitboard(bb: u64) {
    // Format as a binary string with the lowest 49 bits
    let bitset = format!("{:049b}", bb);

    for chunk in bitset.chars().collect::<Vec<_>>().chunks(7) {
        let chunk_str: String = chunk.into_iter().rev().collect();
        println!("{}", chunk_str);
    }
}

pub fn str_to_square(sq: &str) -> Square {
    let file = sq.chars().next().unwrap() as usize - 'a' as usize;
    let rank = sq.chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
    (rank * 7 + file) as Square
}

pub fn milliseconds_elapsed(start_time: Instant) -> u64 {
    let now = Instant::now();
    now.duration_since(start_time).as_millis() as u64
}

pub fn incremental_sort(
    moves: &mut ArrayVec<AtaxxMove, 256>, 
    moves_scores: &mut ArrayVec<i32, 256>, 
    i: usize) 
    -> (AtaxxMove, i32)
{
    for j in ((i+1) as usize)..(moves.len() as usize) 
    {
        if moves_scores[j] > moves_scores[i] 
        {
            (moves[i], moves[j]) = (moves[j], moves[i]);
            (moves_scores[i], moves_scores[j]) = (moves_scores[j], moves_scores[i]);
        }
    }

    (moves[i], moves_scores[i])
}

#[macro_export]
macro_rules! tunable_params {
    ($($name:ident : $type:ty = $default:expr, $min_value:expr, $max_value:expr, $step:expr;)+) => {
        $(
            pub mod $name {
                pub static mut VALUE: $type = $default;
                pub type TYPE = $type;

                #[inline]
                pub fn set(new_val: $type) {
                    unsafe { VALUE = new_val; }
                }
            }

            #[inline]
            pub fn $name() -> $type {
                unsafe { $name::VALUE }
            }
        )+

        #[allow(dead_code)]
        pub fn list_params() {
            $(
                let is_float: bool = std::any::TypeId::of::<$type>() == std::any::TypeId::of::<f32>() 
                                     || std::any::TypeId::of::<$type>() == std::any::TypeId::of::<f64>();

                println!("option name {} type {} default {} min {} max {} step {}",
                    stringify!($name),
                    if is_float { "tunable_float" } else { "tunable_int" },
                    $default,
                    $min_value,
                    $max_value,
                    $step
                );
            )*
        }

        pub fn set_param(param_name: &str, new_value: f64) -> Result<String, &str> {
            match param_name {
                $(
                    stringify!($name) => {
                        $name::set(new_value as $name::TYPE);
                        Ok($name().to_string())
                    }
                )*
                _ => Err("Parameter does not exist")
            }
        }
    };
}

/*
#[allow(dead_code)]
pub fn get_attacks() -> ([u64; 49], [u64; 49])
{
    const ADJACENT_OFFSETS: [[i8; 2]; 8] = [
        [0, 1], [0, -1], [1,  0], [-1,  0],
        [1, 1], [1, -1], [-1, 1], [-1, -1]
    ];

    const LEAP_OFFSETS: [[i8; 2]; 16] = [
        [0, 2], [0, -2], [2,  0], [-2,  0],
        [2, 2], [2, -2], [-2, 2], [-2, -2],
        [1, 2], [1, -2], [-1, 2], [-1, -2],
        [2, 1], [2, -1], [-2, 1], [-2, -1],
    ];

    let mut ADJACENT: [u64; 49] = [0; 49];
    let mut DOUBLES: [u64; 49] = [0; 49];

    for sq in 0..49
    {
        ADJACENT[sq] = 0;
        let rank: i16 = square_rank(sq as u8) as i16;
        let file: i16 = square_file(sq as u8) as i16;
        // Init adjacent squares for this sq
        for i in 0..8
        {
            let rank2 = rank + ADJACENT_OFFSETS[i][0] as i16;
            let file2 = file + ADJACENT_OFFSETS[i][1] as i16;
            if rank2 >= 0 && rank2 <= 6 && file2 >= 0 && file2 <= 6
            {
                let adjacent_sq: u8 = (rank2 * 7 + file2) as u8;
                ADJACENT[sq] |= 1u64 << adjacent_sq;
            }
        }
        // Init leap squares for this sq
        for i in 0..16
        {
            let rank2 = rank + LEAP_OFFSETS[i][0] as i16;
            let file2 = file + LEAP_OFFSETS[i][1] as i16;
            if rank2 >= 0 && rank2 <= 6 && file2 >= 0 && file2 <= 6
            {
                let leap_sq: u8 = (rank2 * 7 + file2) as u8;
                DOUBLES[sq] |= 1u64 << leap_sq;
            }
        }
    }

    return (ADJACENT, DOUBLES)
}
*/

// [square]
pub const ADJACENT: [u64; 49] = [
    386, 901, 1802, 3604, 7208, 14416, 12320,
    49411, 115335, 230670, 461340, 922680, 1845360, 1577056,
    6324608, 14762880, 29525760, 59051520, 118103040, 236206080, 201863168, 
    809549824, 1889648640, 3779297280, 7558594560, 15117189120, 30234378240, 25838485504, 
    103622377472, 241875025920, 483750051840, 967500103680, 1935000207360, 3870000414720, 3307326144512, 
    13263664316416, 30960003317760, 61920006635520, 123840013271040, 247680026542080, 495360053084160, 423337746497536,
    8899172237312, 22230750724096, 44461501448192, 88923002896384, 177846005792768, 355692011585536, 144036023238656,
];

// [square]
pub const DOUBLES: [u64; 49] = [
    115204, 246792, 510097, 1020194, 2040388, 1967112, 1837072,
    14746116, 31589384, 65292433, 130584866, 261169732, 251790344, 235145232, 
    1887502855, 4043441167, 8357431455, 16714862910, 33429725820, 32229164152, 30098589808, 
    241600365440, 517560469376, 1069751226240, 2139502452480, 4279004904960, 4125333011456, 3852619495424, 
    30924846776320, 66247740080128, 136928156958720, 273856313917440, 547712627834880, 528042625466368, 493135295414272, 
    17730713419776, 35461428936704, 75355534655488, 150711069310976, 301422138621952, 35461649137664, 70923029839872, 
    17731504046080, 35463276527616, 75359227740160, 150718455480320, 301436910960640, 35491462250496, 70948564762624,
];

// [color]
pub const ZOBRIST_COLOR: [u64; 2] = [4084159686542515422, 14666181926344167186];

// [color][square]
pub const ZOBRIST_TABLE: [[u64; 49]; 2] = [
    [17563444073393335390, 2846597195139569544, 7633681972977916996, 5469072607805991847, 7350754093880290883, 4291762070919452400, 8487945075206993651, 2382816939830228149, 2707499875389130715, 9906508899829372119, 7634255255865047281, 5160744990611578761, 14588605332432006716, 11542356621647364351, 9315612162286420415, 12682015151750608045, 9939721695426745422, 16463072884740326430, 14233545739975409863, 8588715167378814003, 15761215711043640581, 1138574715776301933, 3580337302502200713, 11437879599337395261, 1898357847822270669, 12254096487100070253, 9352688233920883427, 8919558613532176209, 13615627067972625395, 11041692407249251256, 16456548073144103579, 12205998344455191018, 11053474374972031772, 14378249035002220497, 17563365706878146660, 4471199980657422953, 8777955614717380774, 7836678263780956087, 2502862737529748745, 12390872547812399179, 17358954260330528290, 9246008147709809898, 16968016500155639424, 13892584517438209100, 181923185203793431, 10338611049567237536, 8551159264236839039, 6912631091244182420, 17295865159200245899],
    [2360250368738873274, 12681330155047466656, 3958444305827048595, 16214891748412538006, 13923701873813090480, 14001284309638179086, 7122613200649192092, 1664369051872308496, 14505594381162593033, 13029943882067926212, 10926800102998107133, 16994198603064308018, 11467548303800178734, 12074452811209642058, 14580109577657263026, 1909832834415471101, 4342144556585028182, 1313818423609091331, 5121944267380031616, 7399262579422610645, 2281098811351448471, 9467270851079376158, 11094227385174018218, 15165233144127257400, 12277478863606521539, 5666327351525767465, 5100754193725168702, 4174646403141519904, 14429013029486856002, 8994070408027292846, 8707043480544426421, 14489052731516803857, 4817190416287031698, 13014336581408647183, 8490557223642848867, 6583315161286975665, 1375705114315999423, 18212437245564466536, 1658420121511642312, 16534415663476583279, 16174548524638929290, 449806583819867897, 8489750979875960780, 14689503982528203887, 4192101430992261228, 1709661618577934593, 13454573818564563053, 11880173658315794738, 8081075546100166201],
];


