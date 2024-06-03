#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Bit {
    L,
    H,
    X,
    Z,
}

impl From<bool> for Bit {
    fn from(b: bool) -> Self {
        if b { Bit::H } else { Bit::L }
    }
}

impl From<char> for Bit {
    fn from(c: char) -> Self {
        match c {
            '0' => Bit::L,
            '1' => Bit::H,
            'x' | 'X' => Bit::X,
            'z' | 'Z' | '?' => Bit::Z,
            _ => panic!("Invalid character for Bit"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct U64BasedBitVector {
    bits: u64,
    len: u8,
}

impl U64BasedBitVector {
    pub fn new(bits: u64, len: u8) -> Self {
        if len > 64 {
            panic!("BitVector length must be less than 64");
        }
        let bits = bits & ((1 << len) - 1);
        Self { bits, len }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn get_bit(&self, index: usize) -> Bit {
        if index >= self.len() {
            panic!("BitVector index out of bounds");
        }
        match (self.bits >> index) & 1 {
            0 => Bit::L,
            1 => Bit::H,
            _ => unreachable!(),
        }
    }

    pub fn get_bits(&self, range: std::ops::Range<usize>) -> Self {
        let start = range.start;
        let end = range.end;
        if start >= self.len() || end > self.len() {
            panic!("BitVector index out of bounds");
        }
        let len = end - start;
        let bits = (self.bits >> start) & ((1 << len) - 1);
        Self { bits, len: len as u8 }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
// lsb -> msb
pub struct BoxBasedBitVector {
    pub bits: Box<[Bit]>,
}

impl BoxBasedBitVector {
    pub fn len(&self) -> usize {
        self.bits.len()
    }
}

impl<I> From<I> for BoxBasedBitVector
where
    I: IntoIterator<Item = Bit>,
{
    fn from(iter: I) -> Self {
        Self { bits: iter.into_iter().collect() }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum BitVector {
    U64(U64BasedBitVector),
    Box(BoxBasedBitVector),
}

impl<I> From<I> for BitVector
where
    I: IntoIterator<Item = Bit>,
{
    fn from(iter: I) -> Self {
        let bits = iter.into_iter().collect::<Vec<_>>();
        let len = bits.len() as u8;
        if len <= 64 {
            BitVector::U64(U64BasedBitVector::new(
                bits.into_iter().fold(0, |acc, b| (acc << 1) | (b as u64)),
                len,
            ))
        } else {
            BitVector::Box(BoxBasedBitVector::from(bits))
        }
    }
}
