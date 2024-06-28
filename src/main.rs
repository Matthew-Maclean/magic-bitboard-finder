use rand::
{
    Rng,
    SeedableRng,

    rngs::SmallRng,
};
use clap::{Command, Arg};

use std::io::
{
    Write,

    stderr,
};

fn main()
{
    let matches = Command::new("magic-bitboard-finder")
        .version("1.0")
        .about("Finds magic bitboards for rooks & bishops")
        .arg(Arg::new("attempts")
            .short('a')
            .long("attempts")
            .value_name("ATTEMPTS")
            .value_parser(clap::value_parser!(u32))
            .default_value("100")
            .help("The number of attempts for each square (in millions)"))
        .arg(Arg::new("extra-bit")
            .short('e')
            .long("extra-bit")
            .value_name("EXTRA")
            .value_parser(clap::value_parser!(bool))
            .default_value("true")
            .help(concat!(
                "Whether to add an extra bit to a square's mask ",
                "once past 1/2 attempts with no magic found")))
        .get_matches();

    let attempts = *matches.get_one::<u32> ("attempts") .unwrap();
    let extra    = *matches.get_one::<bool>("extra-bit").unwrap();

    let attempts = attempts * 1_000_000;

    let mut stderr = stderr();

    let mut rng = SmallRng::from_entropy();

    let mut rook_magics = [0u64; 64];
    let mut bish_magics = [0u64; 64];

    let mut rook_shifts = [0u32; 64];
    let mut bish_shifts = [0u32; 64];

    let mut rook_masks  = [0u64; 64];
    let mut bish_masks  = [0u64; 64];

    let mut rook_attacks = vec_arr();
    let mut bish_attacks = vec_arr();

    'square: for square in 0..64
    {
        eprint!("rook {}..", square_name(square)); stderr.flush().unwrap();

        let occupancies = occupancies(rook_mask(square));
        let attacks = occupancies.clone().into_iter()
            .map(|occ| rook_attack(square, occ))
            .collect::<Vec<_>>();

        for i in 0..attempts
        {
            if i % (attempts / 10) == 0
                { eprint!("."); stderr.flush().unwrap(); }

            let magic = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();

            let rook = rook_mask(square);
            let bits = rook.count_ones();

            for shift in 0..(64 - bits)
            {
                let hash = hash(rook, magic, shift, bits);
                if hash.count_ones() >= bits
                {
                    if let Some(attacks) = check_rook_magic(
                        magic, shift, bits, &occupancies, &attacks)
                    {
                        eprintln!("found in {} attempts", i);
                        rook_magics [square as usize] = magic;
                        rook_shifts [square as usize] = shift;
                        rook_masks  [square as usize] = 2u64.pow(bits) - 1;
                        rook_attacks[square as usize] = attacks;
                        continue 'square;
                    }
                    if extra && i > attempts / 2
                    {
                        // try again with one more bit
                        let bits = bits + 1;
                        if let Some(attacks) = check_rook_magic(
                            magic, shift, bits, &occupancies, &attacks)
                        {
                            eprintln!("found (+1 bit) in {} attempts", i);
                            rook_magics [square as usize] = magic;
                            rook_shifts [square as usize] = shift;
                            rook_masks  [square as usize] = 2u64.pow(bits) - 1;
                            rook_attacks[square as usize] = attacks;
                            continue 'square;
                        }
                    }
                }
            }
        }
        eprintln!("failed");
        return;
    }
    'square: for square in 0..64
    {
        eprint!("bishop {}..", square_name(square)); stderr.flush().unwrap();

        let occupancies = occupancies(bish_mask(square));
        let attacks = occupancies.clone().into_iter()
            .map(|occ| bish_attack(square, occ))
            .collect::<Vec<_>>();

        for i in 0..attempts
        {
            if i % (attempts / 10) == 0
                { eprint!("."); stderr.flush().unwrap(); }
            let magic = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();

            let bish = bish_mask(square);
            let bits = bish.count_ones();

            for shift in 0..(64 - bits)
            {
                let hash = hash(bish, magic, shift, bits);
                if hash.count_ones() >= bits
                {
                    if let Some(attacks) = check_bish_magic(
                        magic, shift, bits, &occupancies, &attacks)
                    {
                        eprintln!("found in {} attempts", i);
                        bish_magics [square as usize] = magic;
                        bish_shifts [square as usize] = shift;
                        bish_masks  [square as usize] = 2u64.pow(bits) - 1;
                        bish_attacks[square as usize] = attacks;
                        continue 'square;
                    }
                    if extra && i > attempts / 2
                    {
                        // try again with one more bit
                        let bits = bits + 1;
                        if let Some(attacks) = check_bish_magic(
                            magic, shift, bits, &occupancies, &attacks)
                        {
                            eprintln!("found (+1 bit) in {} attempts", i);
                            bish_magics [square as usize] = magic;
                            bish_shifts [square as usize] = shift;
                            bish_masks  [square as usize] = 2u64.pow(bits) - 1;
                            bish_attacks[square as usize] = attacks;
                            continue 'square;
                        }
                    }
                }
            }
        }
        eprintln!("failed");
        return;
    }

    eprintln!("found all magics. writing to stdout now...");

    println!("// auto-generated");
    // rook magics, shifts, and masks
    println!("pub static ROOK_MAGICS: [u64; 64] = [");
    for square in 0..64
    {
        println!("    0x{:x},", rook_magics[square]);
    }
    println!("];");
    println!();
    println!("pub static ROOK_SHIFTS: [u64; 64] = [");
    for square in 0..64
    {
        println!("    {},", rook_shifts[square]);
    }
    println!("];");
    println!();
    println!("pub static ROOK_MASKS: [u64; 64] = [");
    for square in 0..64
    {
        println!("    0x{:x},", rook_masks[square]);
    }
    println!("];");
    println!();
    println!("pub static ROOK_ATTACKS: [&'static [u64]; 64] = [");
    for square in 0..64
    {
        println!("    &ROOK_{},", square_name_upper(square));
    }
    println!("];");
    println!();
    // bishop magics, shifts, and masks
    println!("pub static BISHOP_MAGICS: [u64; 64] = [");
    for square in 0..64
    {
        println!("    0x{:x},", bish_magics[square]);
    }
    println!("];");
    println!();
    println!("pub static BISHOP_SHIFTS: [u64; 64] = [");
    for square in 0..64
    {
        println!("    {},", bish_shifts[square]);
    }
    println!("];");
    println!();
    println!("pub static BISHOP_MASKS: [u64; 64] = [");
    for square in 0..64
    {
        println!("    0x{:x},", bish_masks[square]);
    }
    println!("];");
    println!();
    println!("pub static BISHOP_ATTACKS: [&'static [u64]; 64] = [");
    for square in 0..64
    {
        println!("    &BISHOP_{},", square_name_upper(square));
    }
    println!("];");
    println!();
    // rook & bishop attacks
    for square in 0..64
    {
        println!("pub static ROOK_{}: [u64; {}] = [",
            square_name_upper(square as u8), rook_attacks[square].len());
        for attack in rook_attacks[square].iter()
        {
            println!("    0x{:x},", attack);
        }
        println!("];");
    }
    println!();
    for square in 0..64
    {
        println!("pub static BISHOP_{}: [u64; {}] = [",
            square_name_upper(square as u8), bish_attacks[square].len());
        for attack in bish_attacks[square].iter()
        {
            println!("    0x{:x},", attack);
        }
        println!("];");
    }
}

fn check_rook_magic(
    magic: u64, shift: u32, bits: u32,
    occupancies: &[u64], attacks: &[u64])
    -> Option<Vec<u64>>
{
    let mut table: Vec<u64> = vec![!0; 2usize.pow(bits)];

    for (occupancy, attack) in occupancies.iter().zip(attacks.iter())
    {
        let hash = hash(*occupancy, magic, shift, bits) as usize;

        match table[hash]
        {
            val if val == !0      => { table[hash] = *attack; },
            val if val == *attack => { },
            _                     => return None,
        }
    }

    Some(table)
}

fn check_bish_magic(
    magic: u64, shift: u32, bits: u32,
    occupancies: &[u64], attacks: &[u64])
    -> Option<Vec<u64>>
{
    let mut table: Vec<u64> = vec![!0; 2usize.pow(bits)];

    for (occupancy, attack) in occupancies.iter().zip(attacks.iter())
    {
        let hash = hash(*occupancy, magic, shift, bits) as usize;

        match table[hash]
        {
            val if val == !0      => { table[hash] = *attack; },
            val if val == *attack => { },
            _                     => return None,
        }
    }

    Some(table)
}

fn rook_mask(square: u8) -> u64
{
    let rank = square / 8;
    let file = square % 8;

    let mut mask = 0u64;

    for r in (rank + 1)..7 { mask |= 1 << (r * 8 + file); }
    for f in (file + 1)..7 { mask |= 1 << (rank * 8 + f); }
    for r in 1..rank       { mask |= 1 << (r * 8 + file); }
    for f in 1..file       { mask |= 1 << (rank * 8 + f); }

    mask
}

fn bish_mask(square: u8) -> u64
{
    let rank = square / 8;
    let file = square % 8;

    let mut mask = 0u64;

    for (r, f) in ((rank + 1)..7).zip((file + 1)..7)
        { mask |= 1 << (r * 8 + f); }
    for (r, f) in ((rank + 1)..7).zip((1..file).rev())
        { mask |= 1 << (r * 8 + f); }
    for (r, f) in (1..rank).rev().zip((file + 1)..7)
        { mask |= 1 << (r * 8 + f); }
    for (r, f) in (1..rank).rev().zip((1..file).rev())
        { mask |= 1 << (r * 8 + f); }

    mask
}

fn occupancies(mask: u64) -> Vec<u64>
{
    let bits = mask.count_ones();
    let len = 2usize.pow(bits) - 1;

    let mut occupancies = Vec::with_capacity(len);

    for pat in 0..len
    {
        let mut board = 0u64;

        let mut pat_idx = 0;
        for mask_idx in 0..64
        {
            if mask & (1 << mask_idx) != 0
            {
                if pat & (1 << pat_idx) != 0
                {
                    board |= 1 << mask_idx;
                }

                pat_idx += 1;
            }
        }

        occupancies.push(board);
    }

    occupancies
}

fn rook_attack(square: u8, occupancy: u64) -> u64
{
    let rank = square / 8;
    let file = square % 8;

    let mut mask = 0u64;

    for r in (rank + 1)..8
    {
        mask |= 1 << (r * 8 + file);
        if occupancy & (1 << (r * 8 + file)) != 0 { break; }
    }
    for f in (file + 1)..8
    {
        mask |= 1 << (rank * 8 + f);
        if occupancy & (1 << (rank * 8 + f)) != 0 { break; }
    }
    for r in (0..rank).rev()
    {
        mask |= 1 << (r * 8 + file);
        if occupancy & (1 << (r * 8 + file)) != 0 { break; }
    }
    for f in (0..file).rev()
    {
        mask |= 1 << (rank * 8 + f);
        if occupancy & (1 << (rank * 8 + f)) != 0 { break; }
    }

    mask
}

fn bish_attack(square: u8, occupancy: u64) -> u64
{
    let rank = square / 8;
    let file = square % 8;

    let mut mask = 0u64;

    for (r, f) in ((rank + 1)..8).zip((file + 1)..8)
    {
        mask |= 1 << (r * 8 + f);
        if occupancy & (1 << (r* 8 + f)) != 0 { break; }
    }
    for (r, f) in ((rank + 1)..8).zip((0..file).rev())
    {
        mask |= 1 << (r * 8 + f);
        if occupancy & (1 << (r* 8 + f)) != 0 { break; }
    }
    for (r, f) in (0..rank).rev().zip((file + 1)..8)
    {
        mask |= 1 << (r * 8 + f);
        if occupancy & (1 << (r* 8 + f)) != 0 { break; }
    }
    for (r, f) in (0..rank).rev().zip((0..file).rev())
    {
        mask |= 1 << (r * 8 + f);
        if occupancy & (1 << (r* 8 + f)) != 0 { break; }
    }

    mask
}

fn hash(board: u64, magic: u64, shift: u32, bits: u32) -> u64
{
    use std::num::Wrapping;

    ((Wrapping(board) * Wrapping(magic)).0 >> shift) & (2u64.pow(bits) - 1)
}

fn vec_arr() -> [Vec<u64>; 64]
{
    [
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
        Vec::new(), Vec::new(), Vec::new(), Vec::new(),
    ]
}

fn square_name(square: u8) -> String
{
    let square = square as usize;

    let file = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'][square % 8];
    let rank = ['1', '2', '3', '4', '5', '6', '7', '8'][square / 8];

    String::from_iter([file, rank])
}

fn square_name_upper(square: u8) -> String
{
    let square = square as usize;

    let file = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'][square % 8];
    let rank = ['1', '2', '3', '4', '5', '6', '7', '8'][square / 8];

    String::from_iter([file, rank])
}
