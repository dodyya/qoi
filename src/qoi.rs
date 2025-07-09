#[derive(Debug, PartialEq)]
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

struct Compresser<I>
where
    I: Iterator<Item = u8>,
{
    pix_stream: I,
    pixel: [u8; 4],
    seen: [[u8; 4]; 64],
}

struct Assembler<I>
where
    I: Iterator<Item = Chunk>,
{
    chunk_stream: I,
}

pub fn parse_img(data: impl Iterator<Item = u8>) -> (u32, u32, Vec<u8>) {
    let mut stream = data.peekable();
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

fn hash(c: [u8; 4]) -> usize {
    (c[0] as usize * 3 + c[1] as usize * 5 + c[2] as usize * 7 + c[3] as usize * 11) % 64
}

//=============================================================//

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

//=============================================================//

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

//=============================================================//

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

pub fn encode_img(pixels: Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    let mut header = vec![b'q', b'o', b'i', b'f'];
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    if pixels.chunks(4).all(|slice| *slice.last().unwrap() == 255) {
        header.push(3); //RGB colorspace
    } else {
        header.push(4); //RGBA
    }
    header.push(1); // Not messing with sRGB yet

    header
        .into_iter()
        .chain(pixels.into_iter().compress().assemble().flatten())
        .chain([0, 0, 0, 0, 0, 0, 0, 1])
        .collect()
}

//=============================================================//

trait Compress
where
    Self: Sized,
    Self: Iterator<Item = u8>,
{
    fn compress(self) -> Compresser<Self>;
}

impl<I> Compress for I
where
    I: Sized,
    I: Iterator<Item = u8>,
{
    fn compress(self) -> Compresser<Self> {
        Compresser {
            pix_stream: self,
            pixel: [0, 0, 0, 255],
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

impl<I> Iterator for Assembler<I>
where
    I: Iterator<Item = Chunk>,
{
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        todo!();
    }
}

impl<I> Iterator for Compresser<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Chunk;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        todo!()
    }
}
