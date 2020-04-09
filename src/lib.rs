#![cfg_attr(any(test, bench), feature(test))]

static PREFIX: &str = "0x";

pub use error::Error;
pub use try_checksum::*;

pub struct Checksum {}

impl Checksum {
    pub fn from_str(input: &str) -> Result<String, Error> {
        match input.len() {
            40 => to_checksum_address(input),
            42 => {
                let prefix = &input[..2];

                if &prefix != &PREFIX {
                    return Err(Error::Prefix {
                        expected: PREFIX,
                        actual: prefix,
                    });
                }

                let hash = &input[2..];

                let checksummed = to_checksum_address(hash)?;

                Ok(format!("{}{}", prefix, checksummed))
            }
            actual => Err(Error::Length {
                expected_either: [40, 42],
                actual,
            }),
        }
    }
}

mod error {
    use std::str::Utf8Error;

    #[derive(Debug, PartialEq, Eq)]
    pub enum Error<'a> {
        Length {
            expected_either: [usize; 2],
            actual: usize,
        },
        Prefix {
            expected: &'a str,
            actual: &'a str,
        },
        Utf8(Utf8Error),
        /// Invalid Hex character
        HexChar {
            value: char,
            index: usize,
        },
    }

    impl<'a> From<Utf8Error> for Error<'a> {
        fn from(e: Utf8Error) -> Self {
            Self::Utf8(e)
        }
    }
}

mod try_checksum {
    use super::*;

    pub trait TryChecksum {
        fn try_checksum<'a>(&'a self) -> Result<String, Error<'a>>;
    }

    impl TryChecksum for str {
        fn try_checksum<'a>(&'a self) -> Result<String, Error<'a>> {
            Checksum::from_str(self)
        }
    }

    impl TryChecksum for String {
        fn try_checksum<'a>(&'a self) -> Result<String, Error<'a>> {
            Checksum::from_str(self)
        }
    }

    impl TryChecksum for [u8; 40] {
        fn try_checksum<'a>(&'a self) -> Result<String, Error<'a>> {
            let string = std::str::from_utf8(self)?;
            Checksum::from_str(string)
        }
    }
}

fn to_checksum_address(address_string: &str) -> Result<String, Error> {
    let address_string = address_string.to_lowercase();
    let hash = keccak256_hash(&address_string);

    address_string
        .char_indices()
        .try_fold(String::with_capacity(40), |mut result, (i, a_char)| {
            let new_char = match a_char {
                a_char @ '0'..='9' => a_char,
                a_char @ 'a'..='f' => {
                    if should_be_uppercased(&hash, i) {
                        a_char.to_uppercase().next().unwrap()
                    } else {
                        a_char
                    }
                },
                a_char => {
                    // fail as soon as possible
                    // On the first invalid char
                    return Err(Error::HexChar {
                        value: a_char,
                        index: i,
                    })
                },
            };

            result.push(new_char);
            Ok(result)
        })
}

fn keccak256_hash<T: AsRef<[u8]>>(address: T) -> [u8; 40] {
    use tiny_keccak::{Hasher, Keccak};

    let mut hasher = Keccak::v256();

    let mut result: [u8; 40] = [0_u8; 40];
    hasher.update(address.as_ref());

    hasher.finalize(&mut result);
    result
}

fn should_be_uppercased(array: &[u8; 40], i: usize) -> bool {
    let half_byte_at: u8 = if i & 1 == 0 {
        array[i / 2] >> 4
    } else {
        array[i / 2] & 0x0f
    };

    half_byte_at >= 8
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    #[test]
    fn test_checksum_from_str() {
        let prefixed_checksum = "0xe0FC04FA2d34a66B779fd5CEe748268032a146c0";

        let addr_lowercase = "0xe0fc04fa2d34a66b779fd5cee748268032a146c0";
        let checksummed = Checksum::from_str(&addr_lowercase).expect("Should be valid String!");

        assert_eq!(PREFIX, &checksummed[..2]);
        assert_eq!(checksummed, prefixed_checksum);

        let addr_uppercase = "0xE0FC04FA2D34A66B779FD5CEE748268032A146C0";
        let checksummed = Checksum::from_str(&addr_uppercase).expect("Should be valid String!");

        assert_eq!(PREFIX, &checksummed[..2]);
        assert_eq!(checksummed, prefixed_checksum);
    }

    #[test]
    fn test_invalid_hex_char() {
        let hex_char = "eqfc04fa2d34a66b779fd5cee748268032a146c0";

        let expected_err = Error::HexChar {
            value: 'q',
            index: 1,
        };
        assert_eq!(Err(expected_err), Checksum::from_str(hex_char));

    }
    #[bench]
    fn bench_checksum(b: &mut Bencher) {
        b.iter(|| {
            let address = test::black_box("0xe0fc04fa2d34a66b779fd5cee748268032a146c0");

            for _ in 0..20_000 {
                Checksum::from_str(address).unwrap();
            }
        })
    }
}
