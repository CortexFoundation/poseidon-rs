extern crate rand;
extern crate ff;

use ff::*;
use pyo3::prelude::*;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(FrRepr);

mod constants;

#[derive(Debug)]
pub struct Constants {
    pub c: Vec<Fr>,
    pub m: Vec<Vec<Fr>>,
    pub width: usize,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,
}

pub fn sbox(params: &Constants, state: &mut Vec<Fr>, i: usize) {
    let half_f = params.n_rounds_f / 2;
    if i < half_f || i >= half_f + params.n_rounds_p {
        state.iter_mut()
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

pub fn mix(state: &Vec<Fr>, param_m: &Vec<Vec<Fr>>) -> Vec<Fr> {
    param_m.iter()
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
        .collect()
}

pub fn hash(params: &Constants, inp: Vec<Fr>) -> Result<Fr, String> {
    if inp.len() + 1 != params.width {
        return Err(String::from("Wrong input length"));
    }

    let mut state = vec![Fr::zero(); params.width];
    state[..inp.len()].clone_from_slice(&inp);

    (0..(params.n_rounds_f + params.n_rounds_p))
        .for_each(|i| {
            state.iter_mut().for_each(|s| s.add_assign(&params.c[i]));
            sbox(params, &mut state, i);
            state = mix(&state, &params.m);
        });

    Ok(state[0])
}

#[pyfunction]
pub fn poseidon_params(
    t: usize, n_rounds_f: usize, n_rounds_p: usize,
    c: Vec<String>, m: Vec<Vec<String>>,) -> usize {
    if c.len() != n_rounds_f + n_rounds_p {
        panic!("parameter c length invalid, expect n_rounds_f + n_rounds_p = {}, but get {}",
               n_rounds_f + n_rounds_p, c.len())
    }

    let params = Box::leak(Box::new(Constants {
        c: c.iter().map(|c| Fr::from_str(c).unwrap()).collect(),
        m: m.iter().map(|l| 
            l.iter().map(|m| Fr::from_str(m).unwrap()).collect()
        ).collect(),
        width: t,
        n_rounds_f, n_rounds_p,
    }));
    params as *const Constants as usize
}

fn as_constants(params_ptr: usize) -> &'static Constants {
    unsafe {
        &*(params_ptr as *const Constants)
    }
}

#[pyfunction]
pub fn poseidon_hash(params_ptr: usize, inp: Vec<String>) -> String {
    hash(as_constants(params_ptr),
         inp.iter().map(|s| Fr::from_str(s).unwrap()).collect())
        .unwrap_or_else(|s| {
            println!("poseidon hash error: {s}");
            Fr::zero()
        }).to_string()
}

#[pyfunction]
pub fn multi_poseidon_hash(params_ptr: usize, inp: Vec<String>) -> Vec<String> {
    let params = as_constants(params_ptr);
    inp
        .chunks(params.width-1)
        .map(|arr| arr.iter().map(|s| Fr::from_str(s).unwrap()).collect())
        .map(|arr| hash(params, arr).unwrap_or_else(|s| {
            println!("poseidon hash error: {s}");
            Fr::zero()
        }).to_string())
        .collect()
}

#[pymodule]
fn poseidon_rs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(poseidon_params, m)?)?;
    m.add_function(wrap_pyfunction!(poseidon_hash, m)?)?;
    m.add_function(wrap_pyfunction!(multi_poseidon_hash, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ff() {
        let a = Fr::from_repr(FrRepr::from(2)).unwrap();
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000002",
            to_hex(&a)
        );

        let b: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495619",
        )
        .unwrap();
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000002",
            to_hex(&b)
        );
        assert_eq!(&a, &b);
    }
}
