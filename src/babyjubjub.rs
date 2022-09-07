// BabyJubJub elliptic curve implementation in Rust.
// For LICENSE check https://github.com/arnaucube/babyjubjub-rs

use ff::*;
use pyo3::prelude::*;
use num_bigint::BigInt;

use crate::poseidon;

pub type Fr = poseidon::Fr; // alias

lazy_static! {
    static ref D: Fr = Fr::from_str("168696").unwrap();
    static ref A: Fr = Fr::from_str("168700").unwrap();
    // pub static ref Q: BigInt = BigInt::parse_bytes(
    //     b"21888242871839275222246405745257275088548364400416034343698204186575808495617",10
    // )
    //     .unwrap();
    static ref B8: Point = Point {
        x: Fr::from_str(
            "16540640123574156134436876038791482806971768689494387082833631921987005038935",
        ).unwrap(),
        y: Fr::from_str(
            "20819045374670962167435360035096875258406992893633759881276124905556507972311",
        ).unwrap(),
    };
}

#[derive(Clone, Debug)]
pub struct PointProjective {
    pub x: Fr,
    pub y: Fr,
    pub z: Fr,
}

impl PointProjective {
    pub fn affine(&self) -> Point {
        if self.z.is_zero() {
            return Point {
                x: Fr::zero(),
                y: Fr::zero(),
            };
        }

        let zinv = self.z.inverse().unwrap();
        let mut x = self.x;
        x.mul_assign(&zinv);
        let mut y = self.y;
        y.mul_assign(&zinv);

        Point { x, y }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn add(&self, q: &PointProjective) -> PointProjective {
        // add-2008-bbjlp https://hyperelliptic.org/EFD/g1p/auto-twisted-projective.html#doubling-dbl-2008-bbjlp
        let mut a = self.z;
        a.mul_assign(&q.z);
        let mut b = a;
        b.square();
        let mut c = self.x;
        c.mul_assign(&q.x);
        let mut d = self.y;
        d.mul_assign(&q.y);
        let mut e = *D;
        e.mul_assign(&c);
        e.mul_assign(&d);
        let mut f = b;
        f.sub_assign(&e);
        let mut g = b;
        g.add_assign(&e);
        let mut x1y1 = self.x;
        x1y1.add_assign(&self.y);
        let mut x2y2 = q.x;
        x2y2.add_assign(&q.y);
        let mut aux = x1y1;
        aux.mul_assign(&x2y2);
        aux.sub_assign(&c);
        aux.sub_assign(&d);
        let mut x3 = a;
        x3.mul_assign(&f);
        x3.mul_assign(&aux);
        let mut ac = *A;
        ac.mul_assign(&c);
        let mut dac = d;
        dac.sub_assign(&ac);
        let mut y3 = a;
        y3.mul_assign(&g);
        y3.mul_assign(&dac);
        let mut z3 = f;
        z3.mul_assign(&g);

        PointProjective {
            x: x3,
            y: y3,
            z: z3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Point {
    pub x: Fr,
    pub y: Fr,
}

impl Point {
    pub fn projective(&self) -> PointProjective {
        PointProjective {
            x: self.x,
            y: self.y,
            z: Fr::one(),
        }
    }

    pub fn mul_scalar(&self, n: &BigInt) -> Point {
        let mut r: PointProjective = PointProjective {
            x: Fr::zero(),
            y: Fr::one(),
            z: Fr::one(),
        };
        let mut exp: PointProjective = self.projective();
        let (_, b) = n.to_bytes_le();
        for i in 0..n.bits() {
            if test_bit(&b, i as usize) {
                r = r.add(&exp);
            }
            exp = exp.add(&exp);
        }
        r.affine()
    }

    pub fn equals(&self, p: Point) -> bool {
        self.x == p.x && self.y == p.y
    }
}

#[inline]
pub fn test_bit(b: &[u8], i: usize) -> bool {
    b[i / 8] & (1 << (i % 8)) != 0
}

#[pyfunction]
pub fn eddsa_verify(inps: [String; 6]) -> bool {
    let [x1, x2, rx, ry, ss, msg] = inps;
    let pk: Point = Point { 
        x: Fr::from_str(&x1).unwrap(), 
        y: Fr::from_str(&x2).unwrap() 
    };
    let r = Point {
        x: Fr::from_str(&rx).unwrap(),
        y: Fr::from_str(&ry).unwrap(),
    };
    let s: BigInt = BigInt::parse_bytes(ss.as_bytes(), 10).unwrap();
    let m = Fr::from_str(&msg).unwrap();

    let mut hm_input = vec![r.x, r.y, pk.x, pk.y, m, Fr::zero()];
    let params_map = poseidon::POSEIDON_PARAMS.read().unwrap();
    let params = params_map.get(&(hm_input.len()))
        .expect("params t:{inp.len()} not initialized");
    let hm = poseidon::hash(params, &mut hm_input);
    let hm_b = BigInt::parse_bytes(to_hex(&hm).as_bytes(), 16).unwrap();

    let lhs = B8.mul_scalar(&s);
    let rhs = r
        .projective()
        .add(&pk.mul_scalar(&hm_b).projective());
    // println!("lhs: {:#?}, rhs: {:#?}", lhs, rhs.affine());
    lhs.equals(rhs.affine())
}
