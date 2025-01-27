
use std::{hash::{Hash, Hasher}, ops::Deref};

#[derive(Clone, Copy)]
pub struct PreHashedStr<'a> {
    str: &'a str,
    hash: u64
}

impl PreHashedStr<'_> {
    pub fn new<'a, H: Hasher>(str: &'a str, mut hasher: H) -> PreHashedStr<'a> {
        str.hash(&mut hasher);
        let hash = hasher.finish();

        PreHashedStr { str, hash }
    }
}

impl PartialEq for PreHashedStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}

impl Eq for PreHashedStr<'_> {}

impl Hash for PreHashedStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl Deref for PreHashedStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.str
    }
}