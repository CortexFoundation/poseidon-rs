use ark_ff::{Fp256, Fp256Parameters, FftParameters, FpParameters, BigInteger256 as BigInteger};


pub type Fp = Fp256<Fpp>;

pub struct Fpp;

impl Fp256Parameters for Fpp {}
impl FftParameters for Fpp {
    type BigInt = BigInteger;

    const TWO_ADICITY: u32 = 32;

    const TWO_ADIC_ROOT_OF_UNITY: Self::BigInt = BigInteger([
        0xd34f1ed960c37c9c,
        0x3215cf6dd39329c8,
        0x98865ea93dd31f74,
        0x03ddb9f5166d18b7,
    ]);
}

impl FpParameters for Fpp {
    /// MODULUS = 21888242871839275222246405745257275088548364400416034343698204186575808495617
    const MODULUS: Self::BigInt = BigInteger([
        0x43e1f593f0000001,
        0x2833e84879b97091,
        0xb85045b68181585d,
        0x30644e72e131a029,
    ]);

    const MODULUS_BITS: u32 = 32;

    const REPR_SHAVE_BITS: u32 = 1;

    const R: Self::BigInt = BigInteger([
        0xac96341c4ffffffb,
        0x36fc76959f60cd29,
        0x666ea36f7879462e,
        0x0e0a77c19a07df2f,
    ]);

    const R2: Self::BigInt = BigInteger([
        0x1bb8e645ae216da7,
        0x53fe3ab1e35c59e3,
        0x8c49833d53bb8085,
        0x0216d0b17f4e44a5
    ]);

    const INV: u64 = 0xffffffffefffffff;

    const GENERATOR: Self::BigInt = BigInteger([
        0x3057819e4fffffdb,
        0x307f6d866832bb01,
        0x5c65ec9f484e3a89,
        0x0180a96573d3d9f8
    ]);

    const CAPACITY: u32 = Self::MODULUS_BITS - 1;

    const T: Self::BigInt = BigInteger([
        0x79b9709143e1f593,
        0x8181585d2833e848,
        0xe131a029b85045b6,
        0x0000000030644e72
    ]);

    const T_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger([
        0x3cdcb848a1f0fac9,
        0x40c0ac2e9419f424,
        0x7098d014dc2822db,
        0x0000000018322739
    ]);

    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt = BigInteger([
        0xa1f0fac9f8000000,
        0x9419f4243cdcb848,
        0xdc2822db40c0ac2e,
        0x183227397098d014
    ]);
}

// impl PrimeField for Fp {
//     type Params;

//     type BigInt;

//     fn from_repr(repr: Self::BigInt) -> Option<Self> {
//         todo!()
//     }

//     fn into_repr(&self) -> Self::BigInt {
//         todo!()
//     }
// }
