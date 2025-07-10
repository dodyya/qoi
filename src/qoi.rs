use std::iter::Peekable;
use std::slice::Chunks;
#[derive(Debug, PartialEq, Clone)]
enum Chunk {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Index { loc: usize },
    Diff { dr: i8, dg: i8, db: i8 },
    Luma { dg: i8, dr_dg: i8, db_dg: i8 },
    Run { length: u8 },
}

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

fn hash(c: [u8; 4]) -> usize {
    (c[0] as usize * 3 + c[1] as usize * 5 + c[2] as usize * 7 + c[3] as usize * 11) % 64
}

///Take in file data as an iterator and return (width, height, pixel data)
pub fn parse_img(data: impl Iterator<Item = u8>) -> (u32, u32, Vec<u8>) {
    let mut stream = data;

    assert_eq!(stream.take_array(), [b'q', b'o', b'i', b'f']);
    let width = u32::from_be_bytes(stream.take_array());
    let height = u32::from_be_bytes(stream.take_array());
    let channels: u8 = stream.next().unwrap();
    assert!(channels == 3 || channels == 4);
    let colorspace: u8 = stream.next().unwrap();
    assert!(colorspace == 0 || colorspace == 1);

    (
        width,
        height,
        stream
            .parse()
            .interpret((width * height) as usize)
            .flatten()
            .collect(),
    )
}

///Take in pixel and dimension data, return the .qoi file as a Vec<u8>
pub fn encode_img(width: u32, height: u32, pixels: Vec<u8>) -> Vec<u8> {
    let mut header = vec![b'q', b'o', b'i', b'f'];
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    if pixels.chunks(4).all(|slice| *slice.last().unwrap() == 255) {
        header.push(3); //RGB colorspace
    } else {
        header.push(4); //RGBA
    }
    header.push(1); // Not messing with sRGB yet

    let compressed: Compresser<Chunks<'_, u8>> = pixels.as_slice().compress();

    header
        .into_iter()
        .chain(compressed.assemble().flatten())
        .chain([0, 0, 0, 0, 0, 0, 0, 1])
        .collect()
}

//==============BOILERPLATE====================================//

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

trait TakeArray<T, const N: usize> {
    fn take_array(&mut self) -> [T; N];
}

impl<I, const N: usize> TakeArray<u8, N> for I
where
    I: Iterator<Item = u8>,
{
    fn take_array(&mut self) -> [u8; N] {
        self.by_ref()
            .take(N)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

//==========END BOILERPLATE====================================//

///Interpret chunks into pixel data
impl<I: Iterator<Item = Chunk>> Iterator for Interpreter<I> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pix_count >= self.max_pix {
            return None;
        }
        let mut out: Vec<u8> = vec![];
        match self.chunk_stream.next()? {
            Chunk::Rgb { r, g, b } => {
                self.pixel = [r, g, b, self.pixel[3]];
            }
            Chunk::Rgba { r, g, b, a } => {
                self.pixel = [r, g, b, a];
            }
            Chunk::Index { loc } => {
                self.pixel = self.seen[loc];
            }
            Chunk::Diff { dr, dg, db } => {
                self.pixel = [
                    self.pixel[0].wrapping_add_signed(dr),
                    self.pixel[1].wrapping_add_signed(dg),
                    self.pixel[2].wrapping_add_signed(db),
                    self.pixel[3],
                ];
            }
            Chunk::Luma { dg, dr_dg, db_dg } => {
                self.pixel = [
                    self.pixel[0].wrapping_add_signed(dr_dg + dg),
                    self.pixel[1].wrapping_add_signed(dg),
                    self.pixel[2].wrapping_add_signed(db_dg + dg),
                    self.pixel[3],
                ];
            }
            Chunk::Run { length } => {
                out.extend_from_slice(&self.pixel.repeat(length as usize - 1));
                self.pix_count += length as usize - 1;
            }
        }
        out.extend_from_slice(&self.pixel);
        self.pix_count += 1;
        self.seen[hash(self.pixel)] = self.pixel;
        Some(out)
    }
}

///Parse file data into a stream of chunks
impl<I: Iterator<Item = u8>> Iterator for Parser<I> {
    type Item = Chunk;
    fn next(&mut self) -> Option<Self::Item> {
        // Implementation of chunking. Stepping along consuming bytes, yielding Chunk. Knows when to consume more bytes.
        let byte: u8 = self.byte_stream.next()?;

        match byte {
            0b1111_1110 => {
                return Some(Chunk::Rgb {
                    r: self.byte_stream.next()?,
                    g: self.byte_stream.next()?,
                    b: self.byte_stream.next()?,
                });
            }
            0b1111_1111 => {
                return Some(Chunk::Rgba {
                    r: self.byte_stream.next()?,
                    g: self.byte_stream.next()?,
                    b: self.byte_stream.next()?,
                    a: self.byte_stream.next()?,
                });
            }
            _ => {}
        }

        match byte >> 6 {
            0b00 => Some(Chunk::Index { loc: byte as usize }),
            0b01 => Some(Chunk::Diff {
                dr: ((byte >> 4 & 0b11) as i8 - 2),
                dg: ((byte >> 2 & 0b11) as i8 - 2),
                db: ((byte & 0b11) as i8 - 2),
            }),
            0b10 => {
                let next_byte = self.byte_stream.next()?;
                return Some(Chunk::Luma {
                    dg: (byte & 0b0011_1111) as i8 - 32,
                    dr_dg: (next_byte >> 4) as i8 - 8,
                    db_dg: (next_byte & 0b1111) as i8 - 8,
                });
            }
            0b11 => Some(Chunk::Run {
                length: (byte & 0b11_1111) + 1,
            }),
            _ => None,
        }
    }
}

///Parse a series of chunks into their byte representation
impl<I> Iterator for Assembler<I>
where
    I: Iterator<Item = Chunk>,
{
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.chunk_stream.next()? {
            Chunk::Rgb { r, g, b } => Some(vec![0b1111_1110, r, g, b]),
            Chunk::Rgba { r, g, b, a } => Some(vec![0b1111_1111, r, g, b, a]),
            Chunk::Index { loc } => Some(vec![loc as u8 & 0b0011_1111]),
            Chunk::Diff { dr, dg, db } => Some(vec![
                0b0100_0000 | ((dr + 2) as u8) << 4 | ((dg + 2) as u8) << 2 | (db + 2) as u8,
            ]),
            Chunk::Luma { dg, dr_dg, db_dg } => Some(vec![
                0b1000_0000 | (dg + 32) as u8,
                ((dr_dg + 8) as u8) << 4 | (db_dg + 8) as u8,
            ]),
            Chunk::Run { length } => Some(vec![0b1100_0000 | (length & 0b0011_1111) - 1]),
        }
    }
}

impl<'a, I> Iterator for Compresser<'a, I>
where
    I: Iterator<Item = &'a [u8]>,
{
    type Item = Chunk;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let pix: [u8; 4] = self.pix_stream.next()?.try_into().unwrap();

        if self.last_pix == pix {
            let mut length: u8 = 1;
            while let Some(&next_pix) = self.pix_stream.peek() {
                if next_pix == pix && length <= 61 {
                    length += 1;
                    self.pix_stream.next();
                } else {
                    break;
                }
            }
            if length > 1 {
                return Some(Chunk::Run { length });
            }
        }
        let (dr, dg, db) = dr_dg_db(pix, self.last_pix);

        if self.seen[hash(pix)] == pix {
            self.last_pix = pix;
            return Some(Chunk::Index { loc: hash(pix) });
        }

        self.seen[hash(pix)] = pix;

        if (-2..=1).contains(&dr) && (-2..=1).contains(&dg) && (-2..=1).contains(&db) {
            self.last_pix = pix;
            return Some(Chunk::Diff {
                dr: dr as i8,
                dg: dg as i8,
                db: db as i8,
            });
        }

        if (-32..=31).contains(&dg) && (-8..7).contains(&(dr - dg)) && (-8..7).contains(&(db - dg))
        {
            self.last_pix = pix;
            return Some(Chunk::Luma {
                dg: dg as i8,
                dr_dg: (dr - dg) as i8,
                db_dg: (db - dg) as i8,
            });
        }

        if pix[3] == self.last_pix[3] {
            self.last_pix = pix;
            return Some(Chunk::Rgb {
                r: pix[0],
                g: pix[1],
                b: pix[2],
            });
        }

        self.last_pix = pix;
        Some(Chunk::Rgba {
            r: pix[0],
            g: pix[1],
            b: pix[2],
            a: pix[3],
        })
    }
}

fn dr_dg_db(pix: [u8; 4], last_pix: [u8; 4]) -> (i16, i16, i16) {
    // println!("{:?}-{:?}", pix, last_pix);
    (
        ((pix[0] as i16) - (last_pix[0] as i16)),
        ((pix[1] as i16) - (last_pix[1] as i16)),
        ((pix[2] as i16) - (last_pix[2] as i16)),
    )
}
