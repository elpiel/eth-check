#![cfg_attr(any(test, bench), feature(test))]

static PREFIX: &str = "0x";

#[derive(Debug)]
pub enum Error<'a> {
    Length { expected_either: [usize; 2], actual: usize },
    Prefix { expected: &'a str, actual: &'a str },
}

pub struct Checksum {}

impl Checksum {
    pub fn checksum(input: &str) -> Result<String, Error> {
        match input.len() {
            40 => Ok(to_checksum_address(input)),
            42 => {
                let prefix = &input[..2];

                if &prefix != &PREFIX {
                    return Err(Error::Prefix {
                        expected: PREFIX,
                        actual: prefix,
                    });
                }

                let hash = &input[2..];

                let checksummed = to_checksum_address(hash);

                Ok(format!("{}{}", prefix, checksummed))
            }
            actual => Err(Error::Length {
                expected_either: [40, 42],
                actual,
            }),
        }
    }
}

fn to_checksum_address(address_string: &str) -> String {
    let address_string = address_string.to_lowercase();
    let hash = keccak256_hash(&address_string);

    address_string
        .char_indices()
        .fold(String::with_capacity(40), |mut result, (i, a_char)| {
            let new_char = match a_char {
                '0'..='9' => a_char,
                a_char if get_half_byte_at(&hash, i) >= 8 => a_char.to_uppercase().next().unwrap(),
                _ => a_char,
            };

            result.push(new_char);
            result
        })
}

fn keccak256_hash<T: AsRef<[u8]>>(address: T) -> [u8; 40] {
    use tiny_keccak::{Keccak, Hasher};

    let mut hasher = Keccak::v256();

    let mut result: [u8; 40] = [0_u8; 40];
    hasher.update(address.as_ref());

    hasher.finalize(&mut result);
    result
}

fn get_half_byte_at(array: &[u8; 40], i: usize) -> u8 {
    if i & 1 == 0 {
        unsafe { array.get_unchecked(i / 2) >> 4 }
    } else {
        unsafe { array.get_unchecked(i / 2) & 0x0f }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    #[test]
    fn prefixed_hash() {
        let hash = "0xe0fc04fa2d34a66b779fd5cee748268032a146c0";
        let result = Checksum::checksum(hash).expect("Is valid");

        assert_eq!(PREFIX, &result[..2]);
    }

    #[test]
    fn test_checksum() {
        let addr_lowercase = "0xe0fc04fa2d34a66b779fd5cee748268032a146c0";
        let checksummed = Checksum::checksum(addr_lowercase).expect("Should be valid String!");
        assert_eq!(checksummed, "0xe0FC04FA2d34a66B779fd5CEe748268032a146c0");

        let addr_uppercase = "0xE0FC04FA2D34A66B779FD5CEE748268032A146C0";
        let checksummed = Checksum::checksum(addr_uppercase).expect("Should be valid String!");
        assert_eq!(checksummed, "0xe0FC04FA2d34a66B779fd5CEe748268032a146c0");
    }

    #[bench]
    fn bench_checksum(b: &mut Bencher) {
        b.iter(|| {
            let address = test::black_box("0xe0fc04fa2d34a66b779fd5cee748268032a146c0");

            for _ in 0..20_000 {
                Checksum::checksum(address).unwrap();
            }
        })
    }
}
