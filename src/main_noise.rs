use clap::{Arg, Command, value_parser};
use std::collections::HashMap;

use lib::{manip, stlio};

fn main() {
    let matches = Command::new("STL Stego Noise Eval Test Tool")
        .about("Fuzzes STL file with random data in desired encode bit ranges, outputs sample files for inspection")
        .arg(Arg::new("file").required(true).value_parser(value_parser!(String)))
        .arg(Arg::new("prefix").required(true).value_parser(value_parser!(String)))
        .arg(Arg::new("max_bits").required(true).value_parser(value_parser!(u8).range(1..=32)))
        .get_matches();

    let file_path: String = matches.get_one::<String>("file").unwrap().clone();
    let output_prefix: String = matches.get_one::<String>("prefix").unwrap().clone();
    let max_bits: u8 = *matches.get_one::<u8>("max_bits").unwrap();

    let (stl, uniq_vertices) = stlio::load_and_report(file_path);
    let mut vertex_map = HashMap::<u128, u128>::new();

    for i in 1..=max_bits {
        fuzz_vertices(&i, &uniq_vertices, &mut vertex_map);
        write_file(&output_prefix, &i, &stl, &vertex_map);
    }
}

fn write_file(
    prefix: &String,
    bits: &u8,
    orig_stl: &stl::BinaryStlFile,
    vmap: &HashMap<u128, u128>,
) {
    let outstl = manip::generate_transformed_stl(orig_stl, vmap);
    let filename: String = format!("{}_{}.stl", prefix, bits);
    println!("Writing file {} for {} encoded bits", filename, bits);
    stlio::write_stl(filename, outstl);
}

fn fuzz_vertices(bits: &u8, source: &Vec<u128>, dest: &mut HashMap<u128, u128>) {
    /*
     *  Randomizes the content of the desired bits on every vertex.
     *  Resulting STL can be used as a test sample for the chosen bit depth:
     *   - to get a visual idea of how affected the functional model will be by encoded data
     *   - to check how it will slice
     */
    dest.clear();
    let mask = manip::get_keep_mask(bits);
    for v in source.iter() {
        if !dest.contains_key(v) {
            let random_bits: u128 = rand::random();
            let fuzzed = (random_bits & !mask) | (v & mask);
            dest.insert(v.clone(), fuzzed);
        }
    }
}
