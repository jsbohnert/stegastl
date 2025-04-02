use clap::{Arg, ArgMatches, Command, value_parser};
use std::collections::HashMap;

use lib::reader_writer::BitFeed;
use lib::reader_writer::ByteFeed;
use lib::{manip, reader_writer, stlio};
use std::io;

fn main() -> std::io::Result<()> {
    let matches = Command::new("StegoSTL test tool: Text Embedding")
        .subcommand(
            Command::new("encode")
                .arg(Arg::new("in_file_path").required(true))
                .arg(Arg::new("out_file_path").required(true))
                .arg(Arg::new("text").required(true))
                .arg(
                    Arg::new("bits")
                        .required(true)
                        .value_parser(value_parser!(u8).range(1..=32)),
                )
                .arg(
                    Arg::new("times")
                        .long("times")
                        .value_parser(value_parser!(u64))
                        .default_value("1"),
                ),
        )
        .subcommand(
            Command::new("decode")
                .arg(Arg::new("in_file_path").required(true))
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
        _ => {
            println!("Unknown command");
            Err(io::Error::new(io::ErrorKind::Other, "Unknown command"))
        }
    }
}

fn handle_encode(args: &ArgMatches) -> std::io::Result<()> {
    let in_file_path: String = args.get_one::<String>("in_file_path").unwrap().clone();
    let out_file_path: String = args.get_one::<String>("out_file_path").unwrap().clone();
    let text: String = args.get_one::<String>("text").unwrap().clone();
    let bits: u8 = *args.get_one::<u8>("bits").unwrap();
    let times: u64 = *args.get_one::<u64>("times").unwrap();

    let (stl, uniq_vertices) = stlio::load_and_report(in_file_path);
    let available_bytes: u64 = manip::get_available_bits(bits, &uniq_vertices) / 8;

    let expected_header_val: u64 = text.len() as u64 * (times);
    let expected_total_write_bytes: u64 = reader_writer::HEADER_BYTES + expected_header_val;
    println!(
        "{}-bit encoding `{}` {} times for {} bytes of text incl header",
        bits, text, times, expected_total_write_bytes
    );
    println!(
        "{} bits of storage provides {} bytes of stored data incl header",
        bits, available_bytes
    );
    assert!(
        expected_total_write_bytes <= available_bytes,
        "Insufficient bytes available to encode text"
    );

    let mut encoder = reader_writer::StringEncoder::new(&text, &times);
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

    write_file(&out_file_path, &stl, &vertex_map);
    Ok(())
}

fn handle_decode(args: &ArgMatches) -> std::io::Result<()> {
    let in_file_path: String = args.get_one::<String>("in_file_path").unwrap().clone();
    let bits: u8 = *args.get_one::<u8>("bits").unwrap();
    let debug = false;

    let (_stl, uniq_vertices) = stlio::load_and_report(in_file_path);

    let mut vman = manip::VertexManipulator::new(manip::ManipulatorMode::READ, uniq_vertices, bits);
    let mut decoder: reader_writer::StringDecoder = reader_writer::StringDecoder::new();

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

    println!("======== BEGIN ENCODED MESSAGE ========");
    println!("{}", String::from_utf8(output).expect("Valid UTF-8"));
    println!("======== END ENCODED MESSAGE ========");
    Ok(())
}

fn write_file(out_file_path: &String, orig_stl: &stl::BinaryStlFile, vmap: &HashMap<u128, u128>) {
    let outstl = manip::generate_transformed_stl(orig_stl, vmap);
    println!("Writing file {}", out_file_path);
    stlio::write_stl(out_file_path.to_string(), outstl);
}
