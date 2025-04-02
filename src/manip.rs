use std::collections::{HashMap, HashSet};
use std::iter::Cycle;
use std::panic;
use std::vec::IntoIter;

pub fn get_uniq_vertices_as_ordered_bits(t: &Vec<stl::Triangle>, m: &mut Vec<u128>) {
    let mut uniq_set = HashSet::<u128>::new();

    for tri in t.iter() {
        let v1full: u128 = get_vertex_as_bits(tri, 0);
        let v2full: u128 = get_vertex_as_bits(tri, 1);
        let v3full: u128 = get_vertex_as_bits(tri, 2);

        if !uniq_set.contains(&v1full) {
            uniq_set.insert(v1full.clone());
            m.push(v1full);
        }
        if !uniq_set.contains(&v2full) {
            uniq_set.insert(v2full.clone());
            m.push(v2full);
        }
        if !uniq_set.contains(&v3full) {
            uniq_set.insert(v3full.clone());
            m.push(v3full);
        }
    }
}

fn validate_bit_range(bits: u8) {
    assert!(bits > 0 && bits <= 32);
}

pub fn get_bit_shifts(bits: u8) -> Vec<u8> {
    validate_bit_range(bits);
    let mut target_bits = Vec::<u8>::new();
    //create a walkable list of which bits in a u128 to touch, in order
    let shifts: [u8; 3] = [96, 64, 32]; //x, y, z
    for vecshift in shifts.iter() {
        for b in (0..bits).rev() {
            target_bits.push(vecshift + b)
        }
    }

    let target_bits = target_bits;
    target_bits
}

pub fn get_keep_mask(bits: &u8) -> u128 {
    let mut mask: u128 = !0; //all bits set to 1 to start
    // mask off the bottom 32bits, unused
    mask <<= 32;
    let shifts = get_bit_shifts(*bits);
    for b in shifts.iter() {
        mask = mask & !(1 << b);
    }
    mask
}

pub fn get_vertex_as_bits(t: &stl::Triangle, i: u8) -> u128 {
    let vx: u128;
    let vy: u128;
    let vz: u128;

    match i {
        0 => {
            vx = t.v1[0].to_bits().into();
            vy = t.v1[1].to_bits().into();
            vz = t.v1[2].to_bits().into();
        }
        1 => {
            vx = t.v2[0].to_bits().into();
            vy = t.v2[1].to_bits().into();
            vz = t.v2[2].to_bits().into();
        }
        2 => {
            vx = t.v3[0].to_bits().into();
            vy = t.v3[1].to_bits().into();
            vz = t.v3[2].to_bits().into();
        }
        _ => panic!("vertex out of range"),
    }

    let mut bits: u128 = vx;
    bits <<= 32;
    bits |= vy;
    bits <<= 32;
    bits |= vz;
    bits <<= 32;

    bits
}

pub fn get_vertex_from_bits(bits: &u128) -> [f32; 3] {
    let vz: u32 = (bits >> 32 & 0xFFFFFFFF) as u32;
    let vy: u32 = (bits >> 64 & 0xFFFFFFFF) as u32;
    let vx: u32 = (bits >> 96 & 0xFFFFFFFF) as u32;

    return [f32::from_bits(vx), f32::from_bits(vy), f32::from_bits(vz)];
}

pub fn generate_transformed_stl(
    orig_stl: &stl::BinaryStlFile,
    vert_trans_map: &HashMap<u128, u128>,
) -> stl::BinaryStlFile {
    let outheader: stl::BinaryStlHeader = stl::BinaryStlHeader {
        header: orig_stl.header.header,
        num_triangles: orig_stl.header.num_triangles,
    };
    let mut tris: Vec<stl::Triangle> = Vec::<stl::Triangle>::new();
    for src_tri in orig_stl.triangles.iter() {
        let v1bits: u128 = get_vertex_as_bits(src_tri, 0);
        let v2bits: u128 = get_vertex_as_bits(src_tri, 1);
        let v3bits: u128 = get_vertex_as_bits(src_tri, 2);
        let newv1bits: u128 = *vert_trans_map.get(&v1bits).unwrap_or(&v1bits);
        let newv2bits: u128 = *vert_trans_map.get(&v2bits).unwrap_or(&v2bits);
        let newv3bits: u128 = *vert_trans_map.get(&v3bits).unwrap_or(&v3bits);
        let new_tri = stl::Triangle {
            normal: src_tri.normal,
            attr_byte_count: src_tri.attr_byte_count,
            v1: get_vertex_from_bits(&newv1bits),
            v2: get_vertex_from_bits(&newv2bits),
            v3: get_vertex_from_bits(&newv3bits),
        };
        tris.push(new_tri);
    }
    let outstl: stl::BinaryStlFile = stl::BinaryStlFile {
        header: outheader,
        triangles: tris,
    };

    outstl
}

pub fn get_available_bits(bits: u8, uniq_vert: &Vec<u128>) -> u64 {
    (uniq_vert.len() * (bits as usize) * 3) as u64
}

#[derive(PartialEq)]
pub enum ManipulatorMode {
    READ,
    WRITE,
}

pub struct VertexManipulator {
    mode: ManipulatorMode,
    bits_encoding: u8,
    vertex_iter: IntoIter<u128>,
    shift_iter: Cycle<IntoIter<u8>>,
    last_shift: u8,
    current_vertex: u128,
    current_vertex_src: u128,
}

impl VertexManipulator {
    pub fn new(mode: ManipulatorMode, vertices: Vec<u128>, bits: u8) -> VertexManipulator {
        let sb = get_bit_shifts(bits);
        let last_shift_bit = *sb.last().unwrap();
        let sbi = sb.into_iter().cycle();
        let mut vi = vertices.into_iter();
        let nv = vi.next().unwrap();
        VertexManipulator {
            mode: mode,
            bits_encoding: bits,
            vertex_iter: vi,
            shift_iter: sbi,
            last_shift: last_shift_bit,
            current_vertex: nv,
            current_vertex_src: nv.clone(),
        }
    }

    pub fn next_bit_from_vertex(&mut self) -> u8 {
        if self.mode != ManipulatorMode::READ {
            return 0;
        }
        let next_shift = self.shift_iter.next().unwrap();
        let next_bit: u8 = (self.current_vertex >> next_shift & 1) as u8;
        let is_last_vertex_bit = next_shift == self.last_shift;
        if is_last_vertex_bit {
            self.current_vertex = self.vertex_iter.next().unwrap_or(0 as u128);
            self.current_vertex_src = self.current_vertex.clone();
        }
        next_bit
    }

    pub fn set_next_bit_in_vertex(&mut self, bit: u8) -> (bool, u128, u128) {
        if self.mode != ManipulatorMode::WRITE {
            return (false, 0, 0);
        }
        let next_shift = self.shift_iter.next().unwrap();
        let mut next_bit: u128 = bit as u128;
        next_bit <<= next_shift;
        let next_bit = next_bit;
        //mask off the current vertex and write the bit
        let mask = 1 << next_shift;
        self.current_vertex = self.current_vertex & !mask | next_bit;

        //if just wrote the last bit in this vertex(or we're done), send the original and the
        //result back to calling code for mapping
        let mut is_last_vertex_bit = false;
        let vertex_result_src: u128 = self.current_vertex_src.clone();
        let vertex_result_res: u128 = self.current_vertex.clone();

        if next_shift == self.last_shift {
            is_last_vertex_bit = true;
            self.current_vertex = self.vertex_iter.next().unwrap_or(0 as u128);
            self.current_vertex_src = self.current_vertex.clone();
        }

        (is_last_vertex_bit, vertex_result_src, vertex_result_res)
    }

    pub fn print_masked_bits(&self) {
        let vbits = format!("{:0128b}", self.current_vertex);
        let mask = get_keep_mask(&self.bits_encoding);
        let masked_bits: String = vbits
            .chars()
            .enumerate()
            .map(|(i, c)| if (mask >> (127 - i)) & 1 == 1 { ' ' } else { c })
            .collect();
        let guide: String = (0..128)
            .map(|i| {
                if i == 32 || i == 64 || i == 96 {
                    '_'
                } else {
                    ' '
                }
            })
            .collect();

        println!("{}", guide);
        println!("{}", vbits);
        println!("{}", masked_bits);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bit_shifts() {
        let _result = panic::catch_unwind(|| {
            get_bit_shifts(0);
        });
        let _result = panic::catch_unwind(|| {
            get_bit_shifts(33);
        });
        let result = get_bit_shifts(1);
        assert_eq!(result, vec![96, 64, 32]);
        let result = get_bit_shifts(3);
        assert_eq!(result, vec![98, 97, 96, 66, 65, 64, 34, 33, 32]);
    }

    #[test]
    fn test_get_keep_mask() {
        let result = get_keep_mask(&(1 as u8));
        let maskfor32: u128 = 0b11111111_11111111_11111111_11111110;
        assert_eq!(
            result,
            (0 as u128 | maskfor32 << 96 | maskfor32 << 64 | maskfor32 << 32 | maskfor32) << 32
        );

        let result = get_keep_mask(&(17 as u8));
        let maskfor32: u128 = 0b11111111_11111110_00000000_00000000;
        assert_eq!(
            result,
            (0 as u128 | maskfor32 << 96 | maskfor32 << 64 | maskfor32 << 32 | maskfor32) << 32
        );
    }

    #[test]
    fn test_vertex_transform() {
        let norm: [f32; 3] = [0.0, 0.0, 0.0];
        let v1: [f32; 3] = [1.0, 2.0, 3.0];
        let v2: [f32; 3] = [4.0, 5.0, 6.0];
        let v3: [f32; 3] = [7.0, 8.0, 9.0];
        let tri = stl::Triangle {
            normal: norm,
            attr_byte_count: 0,
            v1: v1,
            v2: v2,
            v3: v3,
        };

        let result = get_vertex_as_bits(&tri, 0);
        let result = get_vertex_from_bits(&result);
        assert_eq!(result, v1);
        let result = get_vertex_as_bits(&tri, 1);
        let result = get_vertex_from_bits(&result);
        assert_eq!(result, v2);
        let result = get_vertex_as_bits(&tri, 2);
        let result = get_vertex_from_bits(&result);
        assert_eq!(result, v3);
    }

    #[test]
    fn test_get_unique_vertices() {
        let norm: [f32; 3] = [0.0, 0.0, 0.0];
        let v1: [f32; 3] = [1.0, 2.0, 3.0];
        let v2: [f32; 3] = [4.0, 5.0, 6.0];
        let v3: [f32; 3] = [7.0, 8.0, 9.0];

        let v4: [f32; 3] = [10.0, 11.0, 12.0];
        let v5: [f32; 3] = v2;
        let v6: [f32; 3] = [12.0, 13.0, 14.0];

        let v7: [f32; 3] = v4;
        let v8: [f32; 3] = v6;
        let v9: [f32; 3] = [15.0, 16.0, 17.0];

        let mut tris: Vec<stl::Triangle> = vec![
            stl::Triangle {
                normal: norm,
                attr_byte_count: 0,
                v1: v1,
                v2: v2,
                v3: v3,
            },
            stl::Triangle {
                normal: norm,
                attr_byte_count: 0,
                v1: v4,
                v2: v5,
                v3: v6,
            },
            stl::Triangle {
                normal: norm,
                attr_byte_count: 0,
                v1: v7,
                v2: v8,
                v3: v9,
            },
        ];

        let mut result: Vec<u128> = vec![];
        get_uniq_vertices_as_ordered_bits(&mut tris, &mut result);
        let expected: Vec<u128> = vec![
            get_vertex_as_bits(&tris[0], 0),
            get_vertex_as_bits(&tris[0], 1),
            get_vertex_as_bits(&tris[0], 2),
            get_vertex_as_bits(&tris[1], 0),
            get_vertex_as_bits(&tris[1], 2),
            get_vertex_as_bits(&tris[2], 2),
        ];

        assert_eq!(result, expected);
    }
}
