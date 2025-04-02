use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

use crate::manip;

pub fn load_stl(file: &mut File) -> stl::BinaryStlFile {
    let stl = stl::read_stl(file).unwrap();
    stl
}

pub fn load_and_report(file_path: String) -> (stl::BinaryStlFile, Vec<u128>) {
    println!("File: {}", file_path);
    let mut file = OpenOptions::new().read(true).open(file_path).unwrap();
    let mut uniq_vertices = Vec::<u128>::new();
    let stl = load_stl(&mut file);

    manip::get_uniq_vertices_as_ordered_bits(&stl.triangles, &mut uniq_vertices);
    let num_vert = uniq_vertices.len();
    println!("Tris: {}", stl.triangles.len());
    println!("Vertices: {}", num_vert);

    (stl, uniq_vertices)
}

pub fn write_stl(filename: String, stl: stl::BinaryStlFile) {
    let file = File::create(filename).unwrap();
    let mut writer = BufWriter::new(file);
    stl::write_stl(&mut writer, &stl).expect("Error writing output file");
}

pub fn write_binary_file(filename: String, bytes: Vec<u8>) {
    let file = File::create(filename).unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(&bytes).expect("Error writing output file");
    writer.flush().expect("Error flushing output file");
}
