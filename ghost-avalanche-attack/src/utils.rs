use serde::{Serialize};
use base58::{ToBase58};
use std::fmt;


#[derive(Hash, PartialEq, Default, Eq, Clone, PartialOrd, Ord, Serialize)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.0.to_base58())
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", &self.0.to_base58())
    }
}

pub trait Digestable {
    fn digest(&self) -> Digest;
}

