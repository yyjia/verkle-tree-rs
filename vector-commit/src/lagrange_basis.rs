use std::ops::{AddAssign, Index, Mul, MulAssign, Sub};

use ark_ff::PrimeField;
use ark_poly::{EvaluationDomain, Evaluations};
use thiserror::Error;

use crate::{
    precompute::PrecomputedLagrange,
    utils::{inner_product, max},
};

#[derive(Clone)]
pub struct LagrangeBasis<F: PrimeField, D: EvaluationDomain<F>> {
    /// The evaluations (data) stored in the vector
    evaluations: Evaluations<F, D>,

    /// The highest evaluation point
    max: usize,
}

impl<F: PrimeField, D: EvaluationDomain<F>> LagrangeBasis<F, D> {
    pub fn from_vec(data: Vec<F>) -> Self {
        let len = data.len();
        Self::from_vec_and_domain(data, D::new(len).unwrap())
    }

    pub fn from_vec_and_domain(data: Vec<F>, domain: D) -> Self {
        let max = data.len();
        Self {
            evaluations: Evaluations::from_vec_and_domain(data, domain),
            max,
        }
    }

    pub fn new_zero(size: usize) -> Self {
        Self {
            evaluations: Evaluations::from_vec_and_domain(vec![], D::new(size).unwrap()),
            max: 0,
        }
    }

    /// Returns the index of the highest evaluation point (can be smaller than the domain size)
    pub fn max(&self) -> usize {
        self.max - 1
    }

    pub fn elements(&self) -> impl Iterator<Item = &F> {
        self.evaluations.evals.iter()
    }

    pub fn elements_ref(&self) -> &[F] {
        &self.evaluations.evals
    }

    pub fn domain_size(&self) -> usize {
        self.evaluations.domain().size()
    }

    pub(crate) fn evaluate_outside_domain(
        &self,
        precompute: &PrecomputedLagrange<F>,
        point: F,
    ) -> F {
        inner_product(
            &self.evaluations.evals,
            &precompute.compute_barycentric_coefficients(point),
        )
    }

    /// Returns w^i
    pub fn index_to_point(&self, index: usize) -> F {
        self.evaluations.domain().element(index)
    }

    /// Compute the quotient polynomial q(x) = [f(X) - f(x_i)] / [X-x_i]
    pub(crate) fn divide_by_vanishing(
        &self,
        precompute: &PrecomputedLagrange<F>,
        index: usize,
    ) -> Vec<F> {
        let mut q = vec![F::zero(); self.max];
        let index_f = F::from(index as u64);
        let eval = self[index];

        for i in 0..self.max {
            if i == index {
                continue;
            }

            let denom = F::from(i as u64) - index_f;
            let sub = self[i] - eval;
            q[i] = sub / denom;
            q[index] += sub
                * denom
                * (index_f - F::from(i as u64))
                * precompute.vanishing_at(index)
                * precompute.vanishing_inverse_at(i);
        }

        q
    }
}

// TODO: These methods are less needed with LagrangeBasis - refactor
// impl<F: PrimeField, D: EvaluationDomain<F>> VCPreparedData for LagrangeBasis<F, D> {
//     type Item = F;
//     type Error = LagrangeError;

//     fn from_vec(data: Vec<Self::Item>) -> Self {
//         Self::from_vec(data)
//     }

//     fn get(&self, index: usize) -> Option<&Self::Item> {
//         Some(&self[index])
//     }

//     fn get_all(&self) -> Vec<(usize, Self::Item)> {
//         self.elements().enumerate().map(|(i, v)| (i, *v)).collect()
//     }

//     fn max_size(&self) -> usize {
//         self.evaluations.domain().size()
//     }
//     fn set_evaluation(&mut self, index: usize, value: Self::Item) -> Result<(), Self::Error> {
//         todo!()
//     }
// }

impl<F: PrimeField, D: EvaluationDomain<F>> Index<usize> for LagrangeBasis<F, D> {
    type Output = F;

    fn index(&self, index: usize) -> &Self::Output {
        &self.evaluations[index]
    }
}

impl<F: PrimeField, D: EvaluationDomain<F>> AddAssign<&Self> for LagrangeBasis<F, D> {
    fn add_assign(&mut self, rhs: &Self) {
        self.evaluations += &rhs.evaluations;
    }
}

impl<F: PrimeField, D: EvaluationDomain<F>> Sub for LagrangeBasis<F, D> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            evaluations: &self.evaluations - &rhs.evaluations,
            max: *max(&self.max, &rhs.max),
        }
    }
}

impl<F: PrimeField, D: EvaluationDomain<F>> Mul<F> for &LagrangeBasis<F, D> {
    type Output = LagrangeBasis<F, D>;
    fn mul(self, rhs: F) -> Self::Output {
        LagrangeBasis {
            evaluations: &self.evaluations * rhs,
            max: self.max,
        }
    }
}

impl<F: PrimeField, D: EvaluationDomain<F>> MulAssign<F> for LagrangeBasis<F, D> {
    fn mul_assign(&mut self, rhs: F) {
        self.evaluations = &self.evaluations * rhs;
    }
}

#[derive(Error, Clone, Debug)]
pub enum LagrangeError {}
