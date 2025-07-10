use crate::util::{TakeArray, TakeVec};
use std::iter::Peekable;

use core::convert::TryInto;
use crc::{CRC_32_ISO_HDLC, Crc};
use std::fmt::{Debug, Display};
use std::slice::Chunks;
use std::{
    fmt,
    str::{FromStr, Utf8Error, from_utf8},
    string::FromUtf8Error,
};
pub type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkType {
    data: [u8; 4],
}
impl TryFrom<[u8; 4]> for ChunkType {
    type Error = Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        if is_valid_chunk_type(value) {
            Ok(ChunkType { data: value })
        } else {
            Err(format!("Invalid chunk data: {:?}", value).into())
        }
    }
}
impl FromStr for ChunkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err("Invalid chunk type length".into());
        }

        let bytes: [u8; 4] = s.as_bytes().try_into()?;

        if is_valid_chunk_type(bytes) {
            return Ok(ChunkType { data: bytes });
        }
        Err("Invalid chunk data".into())
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = from_utf8(&self.data);
        match result {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.data
    }
    pub fn is_critical(&self) -> bool {
        self.data[0].is_ascii_uppercase()
    }
    pub fn is_public(&self) -> bool {
        self.data[1].is_ascii_uppercase()
    }
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.data[2].is_ascii_uppercase()
    }
    pub fn is_safe_to_copy(&self) -> bool {
        self.data[3].is_ascii_uppercase()
    }

    pub fn is_valid(&self) -> bool {
        is_valid_chunk_type(self.data) && self.is_reserved_bit_valid()
    }
}

fn is_valid_chunk_type(data: [u8; 4]) -> bool {
    for byte in data {
        if !byte.is_ascii_alphabetic() {
            return false;
        }
    }
    return true;
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let mut message: Vec<u8> = chunk_type.bytes().to_vec();
        message.extend_from_slice(data.as_slice());

        Self {
            length: data.len() as u32,
            chunk_type,
            data,
            crc: Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&message),
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.length.to_be_bytes().to_vec();
        bytes.extend_from_slice(&self.chunk_type.bytes()[..]);
        bytes.extend_from_slice(self.data());
        bytes.extend_from_slice(&self.crc.to_be_bytes()[..]);
        bytes
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-byte chunk of type {} with data {:X?} and crc {}",
            self.length, self.chunk_type, self.data, self.crc
        )
    }
}

pub const STANDARD_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

pub fn as_bytes(chunks: Vec<Chunk>) -> Vec<u8> {
    STANDARD_HEADER
        .iter()
        .copied()
        .chain(chunks.iter().flat_map(|chunk| chunk.as_bytes()))
        .collect()
}

pub fn parse_img(data: impl Iterator<Item = u8>) -> (u32, u32, Vec<u8>) {
    let mut stream = data.peekable();
    assert_eq!(stream.take_array().unwrap(), STANDARD_HEADER);

    let mut chunks: Vec<Chunk> = stream.parse().collect(); //CRC validation all happens here
    for c in &chunks {
        println!("{}", c);
    }

    let header_chunk = chunks.first().unwrap();
    assert_eq!(header_chunk.chunk_type().to_string(), "IHDR");
    assert!(&chunks.iter().any(|c| c.chunk_type().to_string() == "IDAT"));
    assert_eq!(&chunks.last().unwrap().chunk_type().to_string(), "IEND");
    let width = u32::from_be_bytes(header_chunk.data()[0..4].try_into().unwrap());
    let height = u32::from_be_bytes(header_chunk.data()[4..8].try_into().unwrap());
    let bit_depth = header_chunk.data()[8];
    let color_type = header_chunk.data()[9];
    let compression_method = header_chunk.data()[10];
    let filter_method = header_chunk.data()[11];
    let interlace_method = header_chunk.data()[12];

    println!("width: {}", width);
    println!("height: {}", height);
    println!("bit depth: {}", bit_depth); // Number of bits per pallette index OR bits in color value
    println!("color type: {}", color_type); // 
    println!("compression method: {}", compression_method);
    println!("filter method: {}", filter_method);
    println!("interlace method: {}", interlace_method);

    (width, height, vec![])
}

///Parse file data into a stream of chunks
impl<I: Iterator<Item = u8>> Iterator for Parser<I> {
    type Item = Chunk;
    fn next(&mut self) -> Option<Self::Item> {
        let data_length: usize = u32::from_be_bytes(self.byte_stream.take_array()?) as usize; // Length of data + 4 bytes for type, CRC and length
        let chunk_type = ChunkType::try_from(self.byte_stream.take_array()?).ok()?;
        let chunk_data = self.byte_stream.take_vec(data_length);
        let chunk_crc = u32::from_be_bytes(self.byte_stream.take_array()?);
        let trial_chunk = Chunk::new(chunk_type, chunk_data);

        assert_eq!(trial_chunk.crc(), chunk_crc);
        assert_eq!(trial_chunk.length(), data_length as u32);

        Some(trial_chunk)
    }
}

///Interpret chunks into pixel data
impl<I: Iterator<Item = Chunk>> Iterator for Interpreter<I> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

///Parse a series of chunks into their byte representation
impl<I> Iterator for Assembler<I>
where
    I: Iterator<Item = Chunk>,
{
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        todo!()
    }
}

impl<'a, I> Iterator for Compresser<'a, I>
where
    I: Iterator<Item = &'a [u8]>,
{
    type Item = Chunk;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        todo!()
    }
}
//==============BOILERPLATE====================================//

struct Interpreter<I>
where
    I: Iterator<Item = Chunk>,
{
    max_pix: usize,
    pix_count: usize,
    chunk_stream: I,
    pixel: [u8; 4],
    seen: [[u8; 4]; 64],
}

struct Parser<I>
where
    I: Iterator<Item = u8>,
{
    byte_stream: I,
}

struct Compresser<'a, I>
where
    I: Iterator<Item = &'a [u8]>,
{
    pix_stream: Peekable<I>,
    last_pix: [u8; 4],
    seen: [[u8; 4]; 64],
}

struct Assembler<I>
where
    I: Iterator<Item = Chunk>,
{
    chunk_stream: I,
}

///Construct an Interpreter
trait Interpret {
    fn interpret(self, max_pix: usize) -> Interpreter<Self>
    where
        Self: Sized,
        Self: Iterator<Item = Chunk>; // Can only call .interpret() on chunk iters
}

impl<I> Interpret for I
where
    I: Iterator<Item = Chunk>,
{
    fn interpret(self, max_len: usize) -> Interpreter<I> {
        Interpreter {
            max_pix: max_len,
            pix_count: 0,
            chunk_stream: self,
            pixel: [0, 0, 0, 255],
            seen: [[0; 4]; 64],
        } //Once called, create an Interpreter with all related state
    }
}

///Construct a Parser
trait Parse {
    fn parse(self) -> Parser<Self>
    where
        Self: Sized,
        Self: Iterator<Item = u8>; // Can only call .parse() on u8 iters
}

impl<I> Parse for I
where
    I: Iterator<Item = u8>,
{
    fn parse(self) -> Parser<I> {
        Parser { byte_stream: self } //Once called, create a Parser with byte_stream as its only field
    }
}

trait Compress<'a, I>
where
    Self: Sized,
    I: Iterator<Item = &'a [u8]>,
{
    fn compress(self) -> Compresser<'a, I>;
}

impl<'a, I> Compress<'a, I> for &'a [u8]
where
    I: Iterator<Item = &'a [u8]> + std::convert::From<std::slice::Chunks<'a, u8>>,
{
    fn compress(self) -> Compresser<'a, I> {
        Compresser {
            pix_stream: <Chunks<'_, u8> as Into<I>>::into(self.chunks(4)).peekable(),
            last_pix: [0, 0, 0, 255],
            seen: [[0; 4]; 64],
        }
    }
}

trait Assemble<I>
where
    I: Iterator<Item = Chunk>,
{
    fn assemble(self) -> Assembler<I>;
}

impl<I> Assemble<I> for I
where
    I: Iterator<Item = Chunk>,
{
    fn assemble(self) -> Assembler<I> {
        Assembler { chunk_stream: self }
    }
}

//==========END BOILERPLATE====================================//
