use std::iter::Peekable;
fn consume_ascii_whitespace(stream: &mut std::iter::Peekable<impl Iterator<Item = u8>>) {
    while stream
        .peek()
        .map_or(false, |&byte| byte.is_ascii_whitespace())
    {
        stream.next();
    }
}

fn consume_ascii_dec(stream: &mut std::iter::Peekable<impl Iterator<Item = u8>>) -> u32 {
    let mut buffer = 0;
    while stream.peek().unwrap().is_ascii_digit() {
        let digit = stream.next().unwrap();
        buffer = buffer * 10 + (digit - b'0') as u32;
    }
    buffer
}

pub fn parse_img(data: impl Iterator<Item = u8>) -> (u32, u32, Vec<u8>) {
    let mut stream = data.peekable();
    assert_eq!(stream.next(), Some(b'P'));
    assert_eq!(stream.next(), Some(b'6'));

    consume_ascii_whitespace(&mut stream);

    while stream.peek().map_or(false, |&byte| byte == b'#') {
        // Repeat for any number of comment lines
        while let Some(b) = stream.next() {
            if b == b'\n' {
                break;
            }
        }
    }

    consume_ascii_whitespace(&mut stream);
    let width = consume_ascii_dec(&mut stream);
    consume_ascii_whitespace(&mut stream);
    let height = consume_ascii_dec(&mut stream);
    consume_ascii_whitespace(&mut stream);
    assert_eq!(255, consume_ascii_dec(&mut stream)); //Only adding support for 8-bit images
    assert_eq!(Some(b'\n'), stream.next());
    //Stream should now be at the start of the image data

    let pixel_buf = stream.space_n(255, 3);
    (width, height, pixel_buf.collect())
}

struct SpaceN<I, T: Clone>
where
    I: Iterator<Item = T>,
{
    stream: I,
    count: usize,
    period: usize,
    spacer: T,
}

impl<I, T: Clone> Iterator for SpaceN<I, T>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.period {
            self.count = 0;
            Some(self.spacer.clone())
        } else {
            self.count += 1;
            self.stream.next()
        }
    }
}

trait Spaceable<I, T: Clone>
where
    I: Iterator<Item = T>,
{
    fn space_n(self, spacer: T, period: usize) -> SpaceN<Self, T>
    where
        Self: Sized,
        Self: Iterator<Item = T>,
    {
        SpaceN {
            stream: self,
            count: 0,
            period,
            spacer,
        }
    }
}

impl<I, T: Clone> Spaceable<I, T> for I
where
    I: Iterator<Item = T>,
{
    fn space_n(self, spacer: T, period: usize) -> SpaceN<Self, T>
    where
        Self: Sized,
        Self: Iterator<Item = T>,
    {
        SpaceN {
            stream: self,
            count: 0,
            period,
            spacer,
        }
    }
}
