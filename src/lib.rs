extern crate rand;
extern crate ff;

use ff::*;
use pyo3::prelude::*;
use rayon::prelude::*;

#[derive(PrimeField)]
#[PrimeFieldModulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[PrimeFieldGenerator = "7"]
pub struct Fr(FrRepr);

// pub mod fast;
// mod constants;
// pub mod ark;

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

#[pyfunction]
pub fn poseidon_params(
    t: usize, n_rounds_f: usize, n_rounds_p: usize,
    c: Vec<String>, m: Vec<Vec<String>>,) -> usize {
    // if c.len() != n_rounds_f + n_rounds_p {
    //     panic!("parameter c length invalid, expect n_rounds_f + n_rounds_p = {}, but get {}",
    //            n_rounds_f + n_rounds_p, c.len())
    // }

    let params = Box::leak(Box::new(Constants {
        c: c.iter().map(|c| Fr::from_str(c).unwrap()).collect(),
        m: m.iter().map(|l| 
            l.iter().map(|m| Fr::from_str(m).unwrap()).collect()
        ).collect(),
        width: t,
        n_rounds_f, n_rounds_p,
        round1: n_rounds_f / 2,
        round2: n_rounds_f / 2 + n_rounds_p,
        round3: n_rounds_f + n_rounds_p,
    }));
    params as *const Constants as usize
}

fn as_constants(params_ptr: usize) -> &'static Constants {
    unsafe {
        &*(params_ptr as *const Constants)
    }
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

    // params.m
    //     .iter()
    //     .zip(aux.iter_mut())
    //     .for_each(|(ml, res)| {
    //         *res = Fr::zero();
    //         ml.iter()
    //             .zip(state.iter())
    //             .for_each(|(a, b)| {
    //                 let mut item = a.clone();
    //                 item.mul_assign(b);
    //                 res.add_assign(&item);
    //             });
    //     });

    state.copy_from_slice(res);
}

pub fn hash(params: &Constants, state: &mut [Fr], aux1: &mut [Fr], aux2: &mut [Fr]) -> String {
    // avoid auto params c index assert.
    assert!(params.c.len() == params.round3);

    (0..params.round3)
        .for_each(|i| {
            state.iter_mut().for_each(|s| s.add_assign(&params.c[i]));
            sbox(params, state, aux1, i);
            mix(params, state, aux1, aux2);
        });

    state[0].to_string()
}

#[pyfunction]
pub fn poseidon_hash(params_ptr: usize, inp: Vec<String>) -> String {
    let params = as_constants(params_ptr);
    let mut state = vec![Fr::zero(); params.width * 3];

    inp.iter().zip(state.iter_mut())
        .for_each(|(i, s)| *s = Fr::from_str(i).unwrap());

    let (state, aux) = state.split_at_mut(params.width);
    let (aux1, aux2) = aux.split_at_mut(params.width);
    hash(params, state, aux1, aux2)
}

#[pyfunction]
pub fn multi_poseidon_hash(params_ptr: usize, inp: Vec<String>) -> Vec<String> {
    let params = as_constants(params_ptr);

    let step = params.width - 1;
    let ostep = params.width * 3;
    let mut output = vec![Fr::zero(); ostep * (inp.len() / step)];

    inp
        .par_chunks(step)
        .zip(output.par_chunks_mut(ostep))
        .map(|(arr, out)| {
            let (state, aux) = out.split_at_mut(params.width);
            let (aux1, aux2) = aux.split_at_mut(params.width);
            arr.iter().zip(state.iter_mut())
                .for_each(|(i, s)| *s = Fr::from_str(i).unwrap());
            hash(params, state, aux1, aux2)
        })
        .collect()
}

#[pymodule]
fn poseidon_rs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(poseidon_params, m)?)?;
    m.add_function(wrap_pyfunction!(poseidon_hash, m)?)?;
    m.add_function(wrap_pyfunction!(multi_poseidon_hash, m)?)?;

    // Python::with_gil(|py| -> PyResult<()> {
    //     let fast = PyModule::new(py, "fast")?;
    //     fast.add_function(wrap_pyfunction!(fast::poseidon_params, m)?)?;
    //     fast.add_function(wrap_pyfunction!(fast::poseidon_hash, m)?)?;
    //     m.add_submodule(fast)?;
    //     Ok(())
    // })?;

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
