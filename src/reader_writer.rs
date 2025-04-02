use bitstream_io::{BitQueue, LittleEndian};
use std::io::{Bytes as FileBytes, Read};
use std::iter::Cycle;
use std::str::Bytes;

pub const HEADER_BYTES: u64 = 8;

pub trait BitFeed {
    fn done(&self) -> bool;
    fn get_bit(&mut self) -> u8;
}

struct BytesIntoBits {
    bytes_pushed: u64,
    bits_in_byte_consumed: u8,
    bytes_consumed: u64,
    bit_queue: BitQueue<LittleEndian, u128>,
}

impl BytesIntoBits {
    fn new(header_val: u64) -> BytesIntoBits {
        BytesIntoBits {
            bytes_pushed: 0,
            bits_in_byte_consumed: 0,
            bytes_consumed: 0,
            bit_queue: BitQueue::from_value(header_val as u128, 64),
        }
    }
    fn can_accept_byte(&self) -> bool {
        self.bit_queue.max_len() - self.bit_queue.len() >= 8
    }
    fn accept_byte(&mut self, byte: u8) {
        self.bit_queue.push(8, byte as u128);
        self.bytes_pushed += 1;
    }
    fn bytes_consumed(&self) -> u64 {
        self.bytes_consumed
    }
    fn get_bit(&mut self) -> u8 {
        let res = self.bit_queue.pop(1) as u8;
        self.bits_in_byte_consumed += 1;
        if self.bits_in_byte_consumed == 8 {
            self.bits_in_byte_consumed = 0;
            self.bytes_consumed += 1;
        }

        res
    }
}

pub struct StringEncoder<'a> {
    iter: Cycle<Bytes<'a>>,
    total_bytes: u64,
    feeder: BytesIntoBits,
}

impl<'a> StringEncoder<'a> {
    pub fn new(content: &'a String, times: &u64) -> StringEncoder<'a> {
        let encoded_bytes = content.len() as u64 * times;
        StringEncoder {
            iter: content.bytes().cycle(),
            total_bytes: HEADER_BYTES + encoded_bytes,
            feeder: BytesIntoBits::new(encoded_bytes),
        }
    }
}

impl<'a> BitFeed for StringEncoder<'a> {
    fn done(&self) -> bool {
        self.feeder.bytes_consumed() >= self.total_bytes
    }
    fn get_bit(&mut self) -> u8 {
        //keep pushing data into the queue, excess will be ignored
        if self.feeder.can_accept_byte() {
            self.feeder.accept_byte(self.iter.next().unwrap());
        }
        self.feeder.get_bit()
    }
}

pub trait ByteFeed {
    fn push_bit(&mut self, bit: u8);
    fn bytes_available(&self) -> u32;
    fn get_byte(&mut self) -> u8;
    fn get_header_bytes(&mut self) -> u64;
    fn header_was_read(&self) -> bool;
}

struct BitsIntoBytes {
    bytes_pushed: u64,
    bits_in_byte_pushed: u8,
    bytes_consumed: u64,
    bit_queue: BitQueue<LittleEndian, u128>,
}
impl BitsIntoBytes {
    fn new() -> BitsIntoBytes {
        BitsIntoBytes {
            bytes_pushed: 0,
            bits_in_byte_pushed: 0,
            bytes_consumed: 0,
            bit_queue: BitQueue::new(),
        }
    }
    fn can_accept_bit(&self) -> bool {
        self.bit_queue.max_len() - self.bit_queue.len() >= 1
    }
    fn accept_bit(&mut self, bit: u8) {
        self.bit_queue.push(1, bit as u128);
        self.bits_in_byte_pushed += 1;
        if self.bits_in_byte_pushed == 8 {
            self.bits_in_byte_pushed = 0;
            self.bytes_pushed += 1;
        }
    }
    fn bytes_consumed(&self) -> u64 {
        self.bytes_consumed
    }
    fn bytes_available(&self) -> u32 {
        self.bit_queue.len() / 8
    }
    fn get_byte(&mut self) -> u8 {
        let res = self.bit_queue.pop(8) as u8;
        self.bytes_consumed += 1;

        res
    }
    fn get_header_bytes(&mut self) -> u64 {
        let res = self.bit_queue.pop(64) as u64;
        self.bytes_consumed += 8;

        res
    }
}

pub struct StringDecoder {
    feeder: BitsIntoBytes,
}

impl StringDecoder {
    pub fn new() -> StringDecoder {
        StringDecoder {
            feeder: BitsIntoBytes::new(),
        }
    }
}

impl ByteFeed for StringDecoder {
    fn push_bit(&mut self, bit: u8) {
        assert!(self.feeder.can_accept_bit(), "bit feed full");
        self.feeder.accept_bit(bit);
    }
    fn bytes_available(&self) -> u32 {
        self.feeder.bytes_available()
    }
    fn get_byte(&mut self) -> u8 {
        self.feeder.get_byte()
    }
    fn get_header_bytes(&mut self) -> u64 {
        self.feeder.get_header_bytes()
    }
    fn header_was_read(&self) -> bool {
        self.feeder.bytes_consumed() >= 8
    }
}

pub struct BinaryEncoder<R: Read> {
    iter: FileBytes<R>,
    total_bytes: u64,
    feeder: BytesIntoBits,
}

impl<R: Read> BinaryEncoder<R> {
    pub fn new(file: R, filesize: u64) -> BinaryEncoder<R> {
        BinaryEncoder {
            iter: file.bytes(),
            total_bytes: HEADER_BYTES + filesize,
            feeder: BytesIntoBits::new(filesize),
        }
    }
}

impl<R: Read> BitFeed for BinaryEncoder<R> {
    fn done(&self) -> bool {
        self.feeder.bytes_consumed() >= self.total_bytes
    }
    fn get_bit(&mut self) -> u8 {
        //keep pushing data into the queue, excess will be ignored
        if self.feeder.can_accept_byte() {
            match self.iter.next() {
                Some(Ok(byte)) => {
                    self.feeder.accept_byte(byte);
                }
                Some(Err(e)) => {
                    panic!("Error reading byte: {}", e);
                }
                None => {
                    // EOF
                }
            }
        }
        self.feeder.get_bit()
    }
}

pub struct BinaryDecoder {
    feeder: BitsIntoBytes,
}

impl BinaryDecoder {
    pub fn new() -> BinaryDecoder {
        BinaryDecoder {
            feeder: BitsIntoBytes::new(),
        }
    }
}

impl ByteFeed for BinaryDecoder {
    fn push_bit(&mut self, bit: u8) {
        assert!(self.feeder.can_accept_bit(), "bit feed full");
        self.feeder.accept_bit(bit);
    }
    fn bytes_available(&self) -> u32 {
        self.feeder.bytes_available()
    }
    fn get_byte(&mut self) -> u8 {
        self.feeder.get_byte()
    }
    fn get_header_bytes(&mut self) -> u64 {
        self.feeder.get_header_bytes()
    }
    fn header_was_read(&self) -> bool {
        self.feeder.bytes_consumed() >= 8
    }
}
