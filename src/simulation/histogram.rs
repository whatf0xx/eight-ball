pub struct BinEdges<T> {
    left: T,
    right: T,
    number: usize,
    width: T,
}

impl BinEdges<f64> {
    pub fn new(left: f64, right: f64, number: usize) -> Self {
        let width = (right - left) / (number as f64);
        BinEdges {
            left,
            right,
            number,
            width,
        }
    }

    pub fn edges(&self) -> Vec<f64> {
        (0..self.number)
            .map(|i| self.left + (i as f64) * self.width)
            .collect()
    }
}

pub trait Bin<T> {
    /// Bin the data contained within `self` based on a passed slice of bin
    /// edges. the data falls entirely within the given bin edges, and the
    /// `bin_edges` array should capture the left- AND right-most edge of the
    /// bins. As such, the output array should have one element fewer than
    /// `bin_edges`.
    fn bin(&self, bin_edges: &BinEdges<T>) -> Vec<usize>;
    // TODO: also add the ability to add data to an existing histogram:
    // fn bin_into<T>(&self, histogram: Histogram<T>);
}

impl Bin<f64> for &[f64] {
    fn bin(&self, bin_edges: &BinEdges<f64>) -> Vec<usize> {
        let left = bin_edges.left;
        let right = bin_edges.right;
        let width = bin_edges.width;
        let mut counts = vec![0; bin_edges.number];
        self.iter()
            .map(|val| val - left)
            .filter(|&val| val >= 0f64 && val < (right - left))
            .map(|val| val / width)
            .map(|val| val as usize) // watch out for failure here
            .for_each(|i| counts[i] += 1);
        counts
    }
}
