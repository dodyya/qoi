pub fn parse_img(data: impl Iterator<Item = u8>) -> (u32, u32, Vec<u8>) {
    let mut stream = data.peekable();
    assert_eq!(stream.take_array(), [b'q', b'o', b'i', b'f']);
    let width = u32::from_be_bytes(stream.take_array());
    let height = u32::from_be_bytes(stream.take_array());
    let channels: u8 = stream.next().unwrap();
    assert!(channels == 3 || channels == 4);
    let colorspace: u8 = stream.next().unwrap();
    assert!(colorspace == 0 || colorspace == 1);

    let chunk_stream = stream.parse();

    (
        width,
        height,
        interpret(chunk_stream, (width * height * 4) as usize),
    )
}

fn hash(c: [u8; 4]) -> usize {
    (c[0] as usize * 3 + c[1] as usize * 5 + c[2] as usize * 7 + c[3] as usize * 11) % 64
}

fn interpret(chunk_stream: impl Iterator<Item = Chunk>, len: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut last_color = [0, 0, 0, 255];
    let mut seen: [[u8; 4]; 64] = [[0, 0, 0, 0]; 64];
    for chunk in chunk_stream {
        if out.len() >= len {
            break;
        }
        match chunk {
            Chunk::Rgb { r, g, b } => {
                last_color = [r, g, b, last_color[3]];
            }
            Chunk::Rgba { r, g, b, a } => {
                last_color = [r, g, b, a];
            }
            Chunk::Index { loc } => {
                last_color = seen[loc];
            }
            Chunk::Diff { dr, dg, db } => {
                last_color = [
                    last_color[0].wrapping_add_signed(dr),
                    last_color[1].wrapping_add_signed(dg),
                    last_color[2].wrapping_add_signed(db),
                    last_color[3],
                ];
            }
            Chunk::Luma { dg, dr_dg, db_dg } => {
                last_color = [
                    last_color[0].wrapping_add_signed(dr_dg + dg),
                    last_color[1].wrapping_add_signed(dg),
                    last_color[2].wrapping_add_signed(db_dg + dg),
                    last_color[3],
                ];
            }
            Chunk::Run { length } => {
                for _ in 0..length - 1 {
                    out.extend_from_slice(&last_color);
                }
            }
        }
        out.extend_from_slice(&last_color);
        seen[hash(last_color)] = last_color;
    }
    return out;
}
//=============================================================//
struct Parser<I>
// Declaration of what a Parser is. Need to do this because it dynamically consumes byte_stream.
// If it didn't, would just map or whatever
where
    I: Iterator<Item = u8>,
{
    byte_stream: I,
}

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

#[derive(Debug, PartialEq)]
enum Chunk {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Index { loc: usize },
    Diff { dr: i8, dg: i8, db: i8 },
    Luma { dg: i8, dr_dg: i8, db_dg: i8 },
    Run { length: u8 },
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

// pub fn encode_img(pixel_data:&[u8], )
