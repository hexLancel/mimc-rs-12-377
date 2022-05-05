extern crate rand;
#[macro_use]
extern crate ff;
use ff::*;

#[derive(PrimeField)]
#[PrimeFieldModulus = "8444461749428370424248824938781546531375899335154063827935233455917409239041"]
#[PrimeFieldGenerator = "21"]
pub struct Fr(FrRepr);

extern crate num;
extern crate num_bigint;
use num_bigint::{BigInt, Sign};
use tiny_keccak::Keccak;

const SEED: &str = "mimc";

pub struct Constants {
    n_rounds: usize,
    cts: Vec<Fr>,
}

pub fn generate_constants(n_rounds: usize) -> Constants {
    let cts = get_constants(SEED, n_rounds);

    Constants {
        n_rounds: n_rounds,
        cts: cts,
    }
}

pub fn get_constants(seed: &str, n_rounds: usize) -> Vec<Fr> {
    let mut cts: Vec<Fr> = Vec::new();
    cts.push(Fr::zero());

    let mut keccak = Keccak::new_keccak256();
    let mut h = [0u8; 32];
    keccak.update(seed.as_bytes());
    keccak.finalize(&mut h);

    let r: BigInt = BigInt::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
        10,
    )
    .unwrap();

    let mut c = BigInt::from_bytes_be(Sign::Plus, &h);
    for _ in 1..n_rounds {
        let (_, c_bytes) = c.to_bytes_be();
        let mut c_bytes32: [u8; 32] = [0; 32];
        let diff = c_bytes32.len() - c_bytes.len();
        c_bytes32[diff..].copy_from_slice(&c_bytes[..]);

        let mut keccak = Keccak::new_keccak256();
        let mut h = [0u8; 32];
        keccak.update(&c_bytes[..]);
        keccak.finalize(&mut h);
        c = BigInt::from_bytes_be(Sign::Plus, &h);

        let n = modulus(&c, &r);
        cts.push(Fr::from_str(&n.to_string()).unwrap());
    }
    cts
}

pub fn modulus(a: &BigInt, m: &BigInt) -> BigInt {
    ((a % m) + m) % m
}

pub struct Mimc7 {
    constants: Constants,
}

impl Mimc7 {
    pub fn new(n_rounds: usize) -> Mimc7 {
        Mimc7 {
            constants: generate_constants(n_rounds),
        }
    }

    pub fn hash(&self, x_in: &Fr, k: &Fr) -> Fr {
        let mut h: Fr = Fr::zero();
        for i in 0..self.constants.n_rounds {
            let mut t: Fr;
            if i == 0 {
                t = x_in.clone();
                t.add_assign(k);
            } else {
                t = h.clone();
                t.add_assign(&k);
                t.add_assign(&self.constants.cts[i]);
            }
            let mut t2 = t.clone();
            t2.square();
            let mut t7 = t2.clone();
            t7.square();
            t7.mul_assign(&t2);
            t7.mul_assign(&t);
            h = t7.clone();
        }
        h.add_assign(&k);
        h
    }

    pub fn multi_hash(&self, arr: Vec<Fr>, key: &Fr) -> Fr {
        let mut r = key.clone();
        for i in 0..arr.len() {
            let h = self.hash(&arr[i], &r);
            r.add_assign(&arr[i]);
            r.add_assign(&h);
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_constants() {
        let constants = generate_constants(91);
        println!("Constants: {}", constants.cts[1].to_string());
        assert_eq!(
            "Fr(0x08d7f0f443d020b71726275c38280ad6d86ce6ed6d921ddb1a956cb7e70a98e3)",
            constants.cts[1].to_string()
        );
    }
}
