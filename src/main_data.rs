use clap::{Arg, ArgMatches, Command, value_parser};
use lib::reader_writer::BitFeed;
use lib::reader_writer::ByteFeed;
use lib::{manip, reader_writer, stlio};
use std::collections::HashMap;
use std::fs::File;
use std::io;
//use std::io::{Read, Seek, SeekFrom};

fn main() -> std::io::Result<()> {
    let matches = Command::new("StegaSTL test tool: Data Embedding")
        .subcommand(
            Command::new("encode")
                .arg(Arg::new("in_file_path").required(true))
                .arg(Arg::new("out_file_path").required(true))
                .arg(Arg::new("data_file_path").required(true))
                .arg(
                    Arg::new("bits")
                        .required(true)
                        .value_parser(value_parser!(u8).range(1..=32)),
                ),
        )
        .subcommand(
            Command::new("decode")
                .arg(Arg::new("in_file_path").required(true))
                .arg(Arg::new("out_file_path").required(true))
                .arg(
                    Arg::new("bits")
                        .required(true)
                        .value_parser(value_parser!(u8).range(1..=32)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("encode", sub_m)) => handle_encode(sub_m),
        Some(("decode", sub_m)) => handle_decode(sub_m),
        _ => Err(io::Error::new(io::ErrorKind::Other, "Unknown command")),
    }
}

fn handle_encode(args: &ArgMatches) -> std::io::Result<()> {
    let in_file_path: String = args.get_one::<String>("in_file_path").unwrap().clone();
    let out_file_path: String = args.get_one::<String>("out_file_path").unwrap().clone();
    let data_file_path: String = args.get_one::<String>("data_file_path").unwrap().clone();
    let bits: u8 = *args.get_one::<u8>("bits").unwrap();

    let (stl, uniq_vertices) = stlio::load_and_report(in_file_path);
    let available_bytes: u64 = manip::get_available_bits(bits, &uniq_vertices) / 8;

    //inspect the input data file
    let mut file = File::open(data_file_path)?;

    let expected_header_val: u64 = file.metadata().expect("metadata").len();
    let expected_total_write_bytes: u64 = reader_writer::HEADER_BYTES + expected_header_val;
    println!(
        "{}-bit encoding {} bytes of data incl header",
        bits, expected_total_write_bytes
    );
    println!(
        "{} bits of storage provides {} bytes of stored data incl header",
        bits, available_bytes
    );
    assert!(
        expected_total_write_bytes <= available_bytes,
        "Insufficient bytes available to encode data"
    );

    let mut encoder = reader_writer::BinaryEncoder::new(&mut file, expected_header_val);
    let mut vman =
        manip::VertexManipulator::new(manip::ManipulatorMode::WRITE, uniq_vertices, bits);
    let mut vertex_map = HashMap::<u128, u128>::new();

    loop {
        if encoder.done() {
            break;
        }

        let next_bit: u8 = encoder.get_bit();
        let (vertex_write, vertex_orig, vertex_changed) = vman.set_next_bit_in_vertex(next_bit);

        //if just wrote the last bit in this vertex(or we're done), map it and move forward
        if vertex_write || encoder.done() {
            vertex_map.insert(vertex_orig, vertex_changed);
        }
    }

    write_encoded_file(&out_file_path, &stl, &vertex_map);
    Ok(())
}

fn handle_decode(args: &ArgMatches) -> std::io::Result<()> {
    let in_file_path: String = args.get_one::<String>("in_file_path").unwrap().clone();
    let out_file_path: String = args.get_one::<String>("out_file_path").unwrap().clone();
    let bits: u8 = *args.get_one::<u8>("bits").unwrap();
    let debug = false;

    let (_stl, uniq_vertices) = stlio::load_and_report(in_file_path);

    let mut vman = manip::VertexManipulator::new(manip::ManipulatorMode::READ, uniq_vertices, bits);
    let mut decoder: reader_writer::BinaryDecoder = reader_writer::BinaryDecoder::new();

    let mut remaining_bytes = 0;

    let mut output = Vec::<u8>::new();

    if debug {
        vman.print_masked_bits()
    };

    while !decoder.header_was_read() {
        if decoder.bytes_available() >= reader_writer::HEADER_BYTES as u32 {
            remaining_bytes = decoder.get_header_bytes();
            break;
        }

        let next_bit = vman.next_bit_from_vertex();

        decoder.push_bit(next_bit);
    }
    println!("Header read, payload bytes: {}", remaining_bytes);

    loop {
        let next_bit = vman.next_bit_from_vertex();

        decoder.push_bit(next_bit);
        if decoder.bytes_available() > 0 {
            output.push(decoder.get_byte());
            remaining_bytes -= 1;
        }
        if remaining_bytes == 0 {
            break;
        }
    }
    let output = output;

    write_decoded_file(&out_file_path, output);
    println!("Decode complete.");
    Ok(())
}

fn write_decoded_file(out_file_path: &String, output: Vec<u8>) {
    println!(
        "Writing {} bytes of data to output file {}",
        output.len(),
        out_file_path
    );
    stlio::write_binary_file(out_file_path.to_string(), output);
}

fn write_encoded_file(
    out_file_path: &String,
    orig_stl: &stl::BinaryStlFile,
    vmap: &HashMap<u128, u128>,
) {
    let outstl = manip::generate_transformed_stl(orig_stl, vmap);
    println!("Writing file {}", out_file_path);
    stlio::write_stl(out_file_path.to_string(), outstl);
}
