pub trait TakeArray<T, const N: usize> {
    fn take_array(&mut self) -> Option<[T; N]>;
}

impl<I, const N: usize> TakeArray<u8, N> for I
where
    I: Iterator<Item = u8>,
{
    fn take_array(&mut self) -> Option<[u8; N]> {
        self.by_ref().take(N).collect::<Vec<_>>().try_into().ok()
    }
}

pub trait TakeVec<T> {
    fn take_vec(&mut self, n: usize) -> Vec<T>;
}

impl<I> TakeVec<u8> for I
where
    I: Iterator<Item = u8>,
{
    fn take_vec(&mut self, n: usize) -> Vec<u8> {
        self.by_ref().take(n).collect()
    }
}
