use std::collections::HashMap;
use std::sync::RwLock;

use ff::*;
use pyo3::prelude::*;
use rayon::prelude::*;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(FrRepr);

// #[derive(Debug)]
pub struct Constants {
    pub c: Vec<Fr>,
    pub m: Vec<Vec<Fr>>,
    pub width: usize,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,

    pub round1: usize,
    pub round2: usize,
    pub round3: usize,
}

lazy_static! {
    pub static ref POSEIDON_PARAMS: RwLock<HashMap<usize, Constants>> = RwLock::new(HashMap::new());
}

#[pyfunction]
pub fn poseidon_params(
    t: usize, n_rounds_f: usize, n_rounds_p: usize,
    c: Vec<String>, m: Vec<Vec<String>>,) -> usize {
    let params = Constants {
        c: c.iter().map(|c| Fr::from_str(c).unwrap()).collect(),
        m: m.iter().map(|l| 
            l.iter().map(|m| Fr::from_str(m).unwrap()).collect()
        ).collect(),
        width: t,
        n_rounds_f, n_rounds_p,
        round1: n_rounds_f / 2,
        round2: n_rounds_f / 2 + n_rounds_p,
        round3: n_rounds_f + n_rounds_p,
    };

    let mut params_map = POSEIDON_PARAMS.write().unwrap();
    params_map.insert(t, params);

    &params_map[&t] as *const Constants as usize
}

pub fn sbox(params: &Constants, state: &mut [Fr], aux: &mut [Fr], i: usize) {
    if i < params.round1 || i >= params.round2 {
        state.iter_mut()
            .zip(aux.iter_mut())
            .for_each(|(s, a)| {
                *a = *s;
                s.square();
                s.square();
                s.mul_assign(a);
            })
    } else {
        aux[0] = state[0];
        state[0].square();
        state[0].square();
        state[0].mul_assign(&aux[0]);
    }
}

pub fn mix(params: &Constants, state: &mut [Fr], aux: &mut [Fr], res: &mut [Fr]) {
    params.m
        .iter()
        .zip(res.iter_mut())
        .for_each(|(ml, res)| {
            *res = Fr::zero();
            ml.iter()
                .zip(state.iter())
                .zip(aux.iter_mut())
                .for_each(|((m, s), a)| {
                    *a = *s;
                    a.mul_assign(m);
                    res.add_assign(a);
                });

        });

    state.copy_from_slice(res);
}

pub fn hash(params: &Constants, state: &mut [Fr]) -> Fr{
    let mut aux1 = vec![Fr::zero(); params.width];
    let mut aux2 = vec![Fr::zero(); params.width];

    // avoid auto params c index assert.
    assert!(params.c.len() == params.round3);

    (0..params.round3)
        .for_each(|i| {
            state.iter_mut().for_each(|s| s.add_assign(&params.c[i]));
            sbox(params, state, &mut aux1, i);
            mix(params, state, &mut aux1, &mut aux2);
        });

    state[0]
}

#[pyfunction]
pub fn poseidon_hash(inp: Vec<String>, t: usize) -> String {
    let params_map = POSEIDON_PARAMS.read().unwrap();
    let params = params_map.get(&t)
        .expect("params t:{inp.len()} not initialized");

    let mut state = vec![Fr::zero(); params.width];

    inp.iter().zip(state.iter_mut())
        .for_each(|(i, s)| *s = Fr::from_str(i).unwrap());
    hash(params, &mut state).to_string()
}

#[pyfunction]
pub fn multi_poseidon_hash(inp: Vec<String>, t: usize) -> Vec<String> {
    let params_map = POSEIDON_PARAMS.read().unwrap();
    let params = params_map.get(&t)
        .expect("params t:{inp.len()} not initialized");

    let step = params.width - 1;
    let ostep = params.width;
    let mut output = vec![Fr::zero(); ostep * (inp.len() / step)];

    inp
        .par_chunks(step)
        .zip(output.par_chunks_mut(ostep))
        .map(|(arr, state)| {
            arr.iter().zip(state.iter_mut())
                .for_each(|(i, s)| *s = Fr::from_str(i).unwrap());

            hash(params, state).to_string()
        })
        .collect()
}
