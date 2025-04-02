use clap::{Arg, Command, value_parser};
use std::collections::HashSet;

use lib::{manip, stlio};

fn main() {
    let matches = Command::new("STL Inspect")
        .about("Evaluates STL model files for viable bit-encoded capacity")
        .arg(
            Arg::new("file")
                .required(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("max_bits")
                .required(true)
                .value_parser(value_parser!(u8).range(1..=32)),
        )
        .get_matches();

    let file_path: String = matches.get_one::<String>("file").unwrap().clone();
    let max_bits: u8 = *matches.get_one::<u8>("max_bits").unwrap();

    let (_stl, uniq_vertices) = stlio::load_and_report(file_path);

    println!("Encoding bits check:");
    println!(
        "{:9}{:>9}{:>15}{:>9}",
        "Bits", "Safe", "Encodable Bits", "(Bytes)"
    );

    for i in 1..=max_bits {
        let safe: bool = test_zeroed_bits(&i, &uniq_vertices);
        let bits_available = manip::get_available_bits(i, &uniq_vertices);
        println!(
            "{:<9}{:>9}{:>15}{:>9}",
            i,
            safe,
            bits_available,
            bits_available / 8
        );
    }
}

fn test_zeroed_bits(bits: &u8, source: &Vec<u128>) -> bool {
    /*
     * Reports back a validity check (true/false) based on a somewhat arbitrary and unscientific
     * test condition:
     *  - If the requested bits were zeroed off from all vertices on all triangles,
     *  - would any originally unique vertices collapse to share a point?
     *  - If so, invalid
     *
     *  Rationale:
     *  - When data is encoded onto the bits, the model must physically change shape
     *  - But if we can effectively discard n bits, and the model still seems to be made
     *    of unique triangles that are still described by their own geometry,
     *  - then it is likely (but not guaranteed) that the model can hold data without being
     *    irredeemable damaged.
     *
     *  This will probably prove to be a poor analysis under intense or academic scrutiny,
     *    but it works on my test cases close enough to be a poor-mans's heuristic.
     */
    let mut uniq_set = HashSet::<u128>::new();
    let mask = manip::get_keep_mask(bits);

    for v in source.iter() {
        let masked = v & mask;
        if !uniq_set.contains(&masked) {
            uniq_set.insert(masked.clone());
        }
    }

    uniq_set.len() == source.len()
}
