extern crate rand;
extern crate ff;

use std::{collections::HashMap, sync::Once, convert::TryInto};
use ff::*;
use pyo3::pyfunction;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(FrRepr);

#[allow(dead_code)]
const ROUND_PS: [usize; 14] = [51, 51, 52, 52, 52, 52, 53, 53, 53, 53, 53, 53, 53, 53];

pub trait Poseidon {
    fn restore_constants(&mut self, c_str: Vec<String>, m_str: Vec<Vec<String>>);

    fn hash(&self, state: &mut [Fr]) -> String;
}

pub trait ConstantsT<const T: usize> {
    const WIDTH: usize = T;

    const N_ROUND_F: usize = 6;
    const N_ROUND_P: usize = ROUND_PS[T-2];

    const ROUND_1: usize = Self::N_ROUND_F / 2;
    const ROUND_2: usize = Self::ROUND_1 + Self::N_ROUND_P;
    const ROUND_3: usize = Self::ROUND_2 + Self::ROUND_1;
}

struct Constants<const T: usize> {
    c: Vec<Fr>,
    m: Vec<Vec<Fr>>,
    // aux1: Vec<Fr>,
    // aux2: Vec<Fr>,
}

impl<const T: usize> ConstantsT<T> for Constants<T> { }

impl<const T: usize> Constants<T> {
    pub fn new() -> Self {
        Self { 
            c: vec![Fr::zero(); Self::ROUND_3],
            m: vec![vec![Fr::zero(); T]; T],
            // aux1: vec![Fr::zero(); Self::ROUND_3],
            // aux2: vec![Fr::zero(); Self::ROUND_3],
        }
    }

    fn sbox(&self, state: &mut [Fr; T], i: usize) {
        // let target = if i < Self::ROUND_1 || i >= Self::ROUND_2 {
        //     T
        // } else {
        //     1
        // };

        // let aux = vec![Fr::zero(); target];


        if i < Self::ROUND_1 || i >= Self::ROUND_2 {
            // for i in 0..T {
            //     let aux = state[i];
            //     state[i].square();
            //     state[i].square();
            //     state[i].mul_assign(&aux);
            // }
            state
                .iter_mut()
                .for_each(|s| {
                    let aux = s.clone();
                    s.square();
                    s.square();
                    s.mul_assign(&aux);
                })
        } else {
            let aux = state[0];
            state[0].square();
            state[0].square();
            state[0].mul_assign(&aux);

        }
    }

    fn mix(&self, state: &mut [Fr; T]) {
        // let mut new_state = vec![Fr::zero(); T];
        // for i in 0..T {
        //     for j in 0..T {
        //         let mut item = self.m[i][j].clone();
        //         item.mul_assign(&state[j]);
        //         new_state[i].add_assign(&item);
        //     }
        // }

        let new_state: Vec<Fr> = self.m
            .iter()
            .map(|ml| {
                let mut res = Fr::zero();
                ml.iter()
                    .zip(state.iter())
                    .for_each(|(a, b)| {
                        let mut item = a.clone();
                        item.mul_assign(b);
                        res.add_assign(&item);
                    });
                res
            })
            .collect();

        state.copy_from_slice(&new_state);
    }
}

impl<const T: usize> Poseidon for Constants<T> {
    fn restore_constants(&mut self, c_str: Vec<String>, m_str: Vec<Vec<String>>) {
        // println!("c str len: {}, m str len: {},{}", c_str.len(), m_str.len(), m_str[0].len());
        // println!("c len: {}, m len: {}, {}", self.c.len(), self.m.len(), self.m[0].len());
        // self.c = c_str.iter().map(|c| Fr::from_str(c).unwrap()).collect();
        // self.m = m_str.iter().map(|l|
        //     l.iter().map(|m| Fr::from_str(m).unwrap()).collect()
        // ).collect();
        self.c.iter_mut()
            .zip(c_str.iter())
            .for_each(|(c, s)| *c = Fr::from_str(s).unwrap());
        self.m.iter_mut()
            .zip(m_str.iter())
            .for_each(|(ml, sl)| {
                ml.iter_mut()
                    .zip(sl.iter())
                    .for_each(|(m, s)| *m = Fr::from_str(s).unwrap())
            });
    }

    fn hash(&self, state: &mut [Fr]) -> String {
        for i in 0..Self::ROUND_3 {
            for j in 0..T { state[j].add_assign(&self.c[i]); }
            // state.iter_mut().for_each(|s| s.add_assign(&self.c[i]));
            self.sbox(state.try_into().unwrap(), i);
            self.mix(state.try_into().unwrap());
        }

        state[0].to_string()
    }
}

static mut POSEIDON_ATTRS: Option<HashMap<usize, Box<dyn Poseidon>>> = None;
static INIT: Once = Once::new();

macro_rules! PARAMS_MAP {
    ( $( $t:literal ),* ) => {{
        let mut params_map: HashMap<usize, Box<dyn Poseidon>> = HashMap::new();
        $(
            let params = Box::new(Constants::<$t>::new());
            params_map.insert($t, params);
        )*

        Some(params_map)
    }}
}

#[pyfunction]
pub fn poseidon_params(t: usize, c_str: Vec<String>, m_str: Vec<Vec<String>>) {

    unsafe {
        INIT.call_once(|| {
            POSEIDON_ATTRS = PARAMS_MAP!(3,4,5,6,7,8,9,10);
        });
    }

    let attrs = unsafe { POSEIDON_ATTRS.as_mut() };
    let ins = attrs.unwrap().get_mut(&t)
        .expect("poseidon t: {t} not initialized");
    ins.restore_constants(c_str, m_str);
}

#[pyfunction]
pub fn poseidon_hash(t: usize, inp: Vec<String>) -> String {
    let attrs = unsafe { POSEIDON_ATTRS.as_mut() };
    let ins = attrs.unwrap().get(&t)
        .expect("poseidon t: {t} not initialized");

    let mut state = vec![Fr::zero(); t];

    inp.iter().zip(state.iter_mut())
        .for_each(|(i, s)| *s = Fr::from_str(i).unwrap());

    ins.hash(&mut state)
}
