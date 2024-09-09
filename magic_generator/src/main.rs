#![allow(clippy::needless_range_loop)]

use std::{collections::HashSet, fs::File, io::Write};

use rand::Rng;

fn vaild_position(position: [i32; 2]) -> bool {
    (0..8).contains(&position[0]) && (0..8).contains(&position[1])
}

fn position_flatten(position: [i32; 2]) -> u8 {
    ((position[0] % 8) + position[1] * 8) as u8
}

fn calculate_sliding(direction: [i32; 2]) -> [u64; 64] {
    let mut table = [0; 64];
    for (position, sliding) in table.iter_mut().enumerate().take(64) {
        let mut bitmap = 0;

        let mut current_position = [position as i32 % 8, position as i32 / 8];

        loop {
            current_position[0] += direction[0];
            current_position[1] += direction[1];

            let is_vaild_position = vaild_position(current_position);

            if is_vaild_position {
                bitmap |= 1 << position_flatten(current_position);
            } else {
                break;
            }
        }

        *sliding = bitmap;
    }
    table
}

#[derive(Clone, Copy)]
pub struct RaycastTables {
    pub north_west: [u64; 64],
    pub north: [u64; 64],
    pub north_east: [u64; 64],

    pub west: [u64; 64],
    pub east: [u64; 64],

    pub south_west: [u64; 64],
    pub south: [u64; 64],
    pub south_east: [u64; 64],
}

impl Default for RaycastTables {
    fn default() -> RaycastTables {
        RaycastTables {
            north_west: calculate_sliding([1, 1]),
            north: calculate_sliding([0, 1]),
            north_east: calculate_sliding([-1, 1]),

            west: calculate_sliding([1, 0]),
            east: calculate_sliding([-1, 0]),

            south_west: calculate_sliding([1, -1]),
            south: calculate_sliding([0, -1]),
            south_east: calculate_sliding([-1, -1]),
        }
    }
}

const TOP: u64 = 0xff00000000000000;
const LEFT: u64 = 0x8080808080808080;
const RIGHT: u64 = 0x101010101010101;
const BOTTOM: u64 = 0xff;

fn generate_bishop_moves() -> [u64; 64] {
    let tables = RaycastTables::default();

    let mut north_west = tables.north_west;
    let north_east = tables.north_east;
    let south_west = tables.south_west;
    let south_east = tables.south_east;

    for position in 0..64 {
        north_west[position] |= north_east[position] | south_west[position] | south_east[position];
    }

    north_west
}

fn generate_bishop_moves_short() -> [u64; 64] {
    let mut rook_moves = generate_bishop_moves();

    for position in 0..64 {
        if (rook_moves[position] & TOP).count_ones() == 1 {
            rook_moves[position] &= !TOP;
        }
        if (rook_moves[position] & LEFT).count_ones() == 1 {
            rook_moves[position] &= !LEFT;
        }
        if (rook_moves[position] & RIGHT).count_ones() == 1 {
            rook_moves[position] &= !RIGHT;
        }
        if (rook_moves[position] & BOTTOM).count_ones() == 1 {
            rook_moves[position] &= !BOTTOM;
        }
    }

    rook_moves
}
fn generate_rook_moves() -> [u64; 64] {
    let tables = RaycastTables::default();

    let mut north = tables.north;
    let west = tables.west;
    let east = tables.east;
    let south = tables.south;

    for position in 0..64 {
        north[position] |= west[position] | east[position] | south[position];
    }

    north
}

fn generate_rook_moves_short() -> [u64; 64] {
    let mut rook_moves = generate_rook_moves();

    for position in 0..64 {
        if (rook_moves[position] & TOP).count_ones() == 1 {
            rook_moves[position] &= !TOP;
        }
        if (rook_moves[position] & LEFT).count_ones() == 1 {
            rook_moves[position] &= !LEFT;
        }
        if (rook_moves[position] & RIGHT).count_ones() == 1 {
            rook_moves[position] &= !RIGHT;
        }
        if (rook_moves[position] & BOTTOM).count_ones() == 1 {
            rook_moves[position] &= !BOTTOM;
        }
    }

    rook_moves
}

fn raycast_calculate(position: u8, occupancy: u64, tables: &RaycastTables) -> u64 {
    fn msb(x: u64) -> u32 {
        63 - x.leading_zeros()
    }
    fn lsb(x: u64) -> u32 {
        x.trailing_zeros()
    }
    let mut ray_cast_sum = 0;
    for (ray_id, ray_table) in [
        tables.north_west,
        tables.north,
        tables.north_east,
        tables.west,
        tables.east,
        tables.south_west,
        tables.south,
        tables.south_east,
    ]
    .iter()
    .enumerate()
    {
        let mut ray = occupancy & ray_table[position as usize];
        if ray != 0 {
            let r_pos = match ray_id {
                0..4 => lsb(ray),
                4..8 => msb(ray),
                _ => panic!(),
            };

            ray = ray_table[r_pos as usize] ^ ray_table[position as usize];
            ray_cast_sum |= ray;
        } else {
            ray_cast_sum |= ray_table[position as usize];
        }
    }

    ray_cast_sum
}

pub fn generate_blocker_data(
    slider_data: [u64; 64],
    slider_data_full: [u64; 64],
) -> Vec<Vec<(u64, u64)>> {
    let mut output = Vec::new();
    let raycast_tables = RaycastTables::default();
    for position in 0..64 {
        let rook_movemask = slider_data[position];
        let mut bit_list: Vec<usize> = Vec::new();

        for bit in 0..64 {
            let bit_bitmask = 1 << bit;
            if rook_movemask & bit_bitmask != 0 {
                bit_list.push(bit);
            }
        }

        let max_number_from_bits: u64 = (1 << bit_list.len()) - 1;

        let mut position_all_possible_blockers = Vec::new();
        for blocker_index in 0..(max_number_from_bits + 1) {
            let mut blocker_pattern = 0;
            for bit in 0..bit_list.len() {
                let bitmask: u64 = 1 << bit;
                let bit_location: u64 = 1 << bit_list[bit];
                if bitmask & blocker_index != 0 {
                    blocker_pattern |= bit_location
                }
            }

            let sum_move = slider_data_full[position] & blocker_pattern;
            let reduced_move = slider_data_full[position]
                & raycast_calculate(position as u8, sum_move, &raycast_tables);

            position_all_possible_blockers.push((blocker_pattern, reduced_move));
        }
        output.push(position_all_possible_blockers)
    }

    output
}

fn random_u64() -> u64 {
    let mut rng = rand::thread_rng();

    rng.gen()
}

const MAGIC_SHIFT_BISHOP: usize = 64 - 13;
const MAGIC_SHIFT_ROOK: usize = 64 - 12;
fn main() {
    let data_rook = generate_blocker_data(generate_rook_moves_short(), generate_rook_moves()); // DO NOT LOOK AT THIS FUNCTION PLEASE
    let data_bishop = generate_blocker_data(generate_bishop_moves_short(), generate_bishop_moves());

    let (magic_arr_rook, magics_rook) = gen_magics(data_rook, MAGIC_SHIFT_ROOK);
    let (magic_arr_bishop, magics_bishop) = gen_magics(data_bishop, MAGIC_SHIFT_BISHOP);

    // Meta code generation
    let mut source_code: Vec<String> = Vec::new();
    source_code.push("// Autogenerated code, do not modify here.".to_string());
    source_code.push(format!(
        "pub const MAGIC_SHIFT_ROOK: usize = {};",
        MAGIC_SHIFT_ROOK
    ));
    source_code.push(format!(
        "pub const MAGICS_ROOK: [u64; 64] = {:?};",
        magic_arr_rook
    ));
    source_code.push(format!(
        "pub const LOOKUP_ROOK: [[u64; {}]; 64] = {:?};",
        magics_rook[0].len(),
        magics_rook
    ));
    source_code.push(format!(
        "pub const MAGIC_SHIFT_BISHOP: usize = {};",
        MAGIC_SHIFT_BISHOP,
    ));
    source_code.push(format!(
        "pub const MAGICS_BISHOP: [u64; 64] = {:?};",
        magic_arr_bishop,
    ));
    source_code.push(format!(
        "pub const LOOKUP_BISHOP: [[u64; {}]; 64] = {:?};",
        magics_bishop[0].len(),
        magics_bishop
    ));

    let mut data_file = File::create("../fchess/src/magics.rs").expect("creation failed"); // This should be done in a build script
    data_file
        .write_all(source_code.join("\n").as_bytes())
        .unwrap();
}

fn gen_magics(data: Vec<Vec<(u64, u64)>>, shift: usize) -> ([u64; 64], Vec<Vec<u64>>) {
    let mut magic_arr = [0u64; 64];
    let mut magics = Vec::new();
    for (chess_position_index, result) in data.iter().enumerate() {
        'outer: loop {
            let mut seen_indexes = HashSet::new();
            let max_index_lookup = 1 << ((65 - shift) - 1); // Considering N bits the max uint would be this.
            let mut lookup_table = vec![0; max_index_lookup];
            let magic = random_u64() & random_u64() & random_u64(); // Sparsely populated magics tend to be better.
            for position in result {
                let index = (position.0).wrapping_mul(magic) >> shift;

                if seen_indexes.contains(&index) {
                    continue 'outer; // Retry
                }
                seen_indexes.insert(index);

                lookup_table[index as usize] = position.1;
            }
            magics.push(lookup_table);
            magic_arr[chess_position_index] = magic;
            break;
        }
    }
    (magic_arr, magics)
}
