pub struct Histogram {
    left: f64,
    right: f64,
    bins: usize,
    counts: Vec<usize>,
}

impl Histogram {
    pub fn bin(left: f64, right: f64, bins: usize, data: Box<dyn Iterator<Item = f64>>) -> Self {
        let width = (right - left) / (bins as f64);
        let mut counts = vec![0; bins];
        data.into_iter()
            .map(|val| val - left)
            .filter(|&val| val >= 0f64 && val < (right - left))
            .map(|val| val / width)
            .map(|val| val as usize) // watch out for failure here
            .for_each(|i| counts[i] += 1);

        Histogram {
            left,
            right,
            bins,
            counts,
        }
    }

    #[allow(dead_code)]
    pub fn edges(&self) -> Vec<f64> {
        let width = self.width();
        (0..=self.bins)
            .map(|i| self.left + (i as f64) * width)
            .collect()
    }

    pub fn centres(&self) -> Vec<f64> {
        let width = self.width();
        (0..self.bins)
            .map(|i| self.left + (i as f64 + 0.5) * width)
            .collect()
    }

    pub fn width(&self) -> f64 {
        (self.right - self.left) / (self.bins as f64)
    }

    pub fn counts(&self) -> Vec<usize> {
        self.counts.to_owned()
    }
}
