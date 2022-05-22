use super::*;
use itertools::Itertools;
use std::cmp::Ordering;

// y = ax +b
// [a, b]

pub fn linear_regression(data: &Vec<(f64, f64)>) -> Result<Array<f64, Ix1>, String> {
    let dim = data.len();
    let x: Vec<f64> = data.iter().map(|i| i.0).collect();
    let y: Vec<f64> = data.iter().map(|i| i.1).collect();

    let a: Array<f64, Ix2> = match Array::from_shape_vec((dim, 1), x) {
        Ok(result) => concatenate!(Axis(1), result, Array::ones((dim, 1))),
        Err(result) => {
            return Err(error_message!(
                "failed to create matrix from vector!\ndetails : {:?}",
                result
            ));
        }
    };
    let b = Array::from_vec(y);
    let x = match (a.t().dot(&a)).solve_into(a.t().dot(&b)) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to solve Ax=b!\ndetails : {:?}",
                result
            ));
        }
    };
    Ok(x)
}

pub fn get_cross_point(
    a_1: &Array<f64, Ix1>,
    a_2: &Array<f64, Ix1>,
) -> Result<Array<f64, Ix1>, String> {
    let a = stack!(Axis(0), *a_1, *a_2);
    let (a, mut b) = a.view().split_at(Axis(1), 1);
    let a = concatenate!(Axis(1), a, Array::from_elem((a.ndim(), 1), -1.0));
    b.swap_axes(1, 0);
    let b: Array<f64, Ix1> = b.iter().map(|&x| -x).collect();
    let x = match a.solve_into(b) {
        Ok(result) => result,
        Err(result) => {
            return Err(error_message!(
                "failed to solve Ax=b!\ndetails : {:?}",
                result
            ));
        }
    };
    Ok(x)
}

pub fn fast_fourier_transform(input: Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let mut input = input;
    let fft = FftPlanner::new().plan_fft_forward(input.len());
    fft.process(&mut input);
    input
}

pub fn analyze_spectrum(sampling_frequency: f64, input: Vec<Complex<f64>>) -> Vec<(f64, f64)> {
    let mut ffted = fast_fourier_transform(input);
    let length = ffted.len();
    for b in &mut ffted {
        *b = Complex {
            re: 2.0 * b.re / length as f64,
            im: 2.0 * b.im / length as f64,
        };
    }
    let mut spectrum = Vec::new();
    let mut counter = 0.0;
    for c in &ffted {
        spectrum.push((
            counter * sampling_frequency / length as f64,
            (c.re.powi(2) + c.im.powi(2)).sqrt(),
        ));
        counter += 1.0;
    }
    spectrum
}

pub trait ProbabilityEvent {
    fn get_index(&self) -> usize;
    fn get_max_event_size() -> usize;
}

impl ProbabilityEvent for Ordering {
    fn get_index(&self) -> usize {
        match self {
            Ordering::Equal => 0,
            Ordering::Greater => 1,
            Ordering::Less => 2,
        }
    }
    fn get_max_event_size() -> usize {
        3
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Probability<X> {
    p_matrix: Array4<f64>,
    events: Vec<X>,
}

impl<X: ProbabilityEvent> Probability<X> {
    pub fn new(probability_size: usize) -> Self {
        Probability {
            p_matrix: Array4::zeros((
                probability_size,
                probability_size,
                X::get_max_event_size(),
                X::get_max_event_size(),
            )),
            events: Vec::new(),
        }
    }

    pub fn p(&self, x: usize, y: usize, prob_value_x: X, prob_value_y: X) -> f64 {
        self.p_matrix[[x, y, prob_value_x.get_index(), prob_value_y.get_index()]]
    }

    pub fn update(&mut self, probability_values: Vec<X>) {
        self.events = probability_values;
        for v in self
            .events
            .iter()
            .enumerate()
            .combinations_with_replacement(2)
        {
            if v[0].0 != v[1].0 {
                self.p_matrix[[v[0].0, v[1].0, v[0].1.get_index(), v[1].1.get_index()]] += 1.0f64;
                self.p_matrix[[v[1].0, v[0].0, v[0].1.get_index(), v[1].1.get_index()]] += 1.0f64;
            } else {
                self.p_matrix[[v[0].0, v[1].0, v[0].1.get_index(), v[1].1.get_index()]] += 1.0f64;
            }
        }
    }

    pub fn normalize(&mut self) {
        for mut p in self.p_matrix.exact_chunks_mut((1, 1, 3, 3)) {
            let sum = p.sum();
            for i in p.iter_mut() {
                *i /= sum;
            }
        }
    }
}
