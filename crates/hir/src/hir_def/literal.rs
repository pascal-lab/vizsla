use super::bit::{Bit, BoxBasedBitVector};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Literal {
    // 2 states
    Int(i64),
    Float(FloatTypeWrapper),
    // TODO: optimization for U64BasedBitVector
    // 4 states
    Vector {
        bits: BoxBasedBitVector,
        signed: bool,
        base: Base,
    },
    Time {
        // TODO: support float time
        v: usize,
        unit: TimeUnit,
    },
    Str(Box<str>),
    UnbasedUnsized(Bit),
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum TimeUnit {
    S,
    Ms,
    Us,
    Ns,
    Ps,
    Fs,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Base {
    Bin,
    Oct,
    Dec,
    Hex,
}

// FloatTypeWrapper is a wrapper around f64 to allow it to be used in a HashMap.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct FloatTypeWrapper(u64);

impl From<f64> for FloatTypeWrapper {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<f32> for FloatTypeWrapper {
    fn from(value: f32) -> Self {
        Self::new(value as f64)
    }
}

impl Into<f64> for FloatTypeWrapper {
    fn into(self) -> f64 {
        f64::from_bits(self.0)
    }
}

impl Into<f32> for FloatTypeWrapper {
    fn into(self) -> f32 {
        f64::from_bits(self.0) as f32
    }
}

impl fmt::Display for FloatTypeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", f64::from_bits(self.0))
    }
}
