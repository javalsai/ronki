pub mod units {
    use std::ops::*;

    pub mod sizes {
        use super::*;
        use std::fmt::Display;

        pub fn bytes_to_metric<T: Normalizable + From<u16> + Display>(amount: T) -> String {
            let (amount, magnitude_idx) =
                normalize_magnitude_metric(amount, MAGNITUDE_PREFIX_METRIC.len() - 1);
            format!("{amount} {}B", MAGNITUDE_PREFIX_METRIC[magnitude_idx])
        }

        pub fn bytes_to_binary<T: Normalizable + From<u16> + Display>(amount: T) -> String {
            let (amount, magnitude_idx) =
                normalize_magnitude_binary(amount, MAGNITUDE_PREFIX_BINARY.len() - 1);
            format!("{amount} {}B", MAGNITUDE_PREFIX_BINARY[magnitude_idx])
        }
    }

    pub trait Normalizable: DivAssign + PartialOrd + Copy {}
    impl<T: DivAssign + PartialOrd + Copy> Normalizable for T {}

    pub static MAGNITUDE_PREFIX_METRIC: &[&str] = &["", "K", "M", "G", "T", "P", "E", "Z"];
    pub static MAGNITUDE_PREFIX_BINARY: &[&str] = &["", "Ki", "Mi", "Gi", "Ti", "Pi", "Ei", "Zi"];

    pub fn normalize_magnitude_metric<T: Normalizable + From<u16>>(amount: T, max_idx: usize) -> (T, usize) {
        normalize_magnitude(amount, T::from(1000), max_idx)
    }

    pub fn normalize_magnitude_binary<T: Normalizable + From<u16>>(amount: T, max_idx: usize) -> (T, usize) {
        normalize_magnitude(amount, T::from(1024), max_idx)
    }

    pub fn normalize_magnitude<T: Normalizable>(
        mut amount: T,
        radix: T,
        mut max_idx: usize,
    ) -> (T, usize) {
        if max_idx == 0 {
            max_idx = usize::MAX;
        }
        let mut unit_idx = 0;

        while amount >= radix && unit_idx < max_idx {
            amount /= radix;
            unit_idx += 1;
        }

        (amount, unit_idx)
    }
}
