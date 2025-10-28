#[derive(Clone, Debug)]
pub struct Stats {
    data_points: Vec<f32>,
}

macro_rules! impl_traits {
    ($($t:ty),+) => {
        $(
            impl FromIterator<$t> for Stats
            {
                fn from_iter<I: IntoIterator<Item = $t>>(iter: I) -> Self {
                    Self::new(
                        iter.into_iter().map(|x| x as f32).collect::<Vec<f32>>()
                    )
                }
            }

            impl From<Vec<$t>> for Stats
            {
                fn from(data_points: Vec<$t>) -> Self {
                    data_points.into_iter().collect::<Self>()
                }
            }
        )+
    }
}

impl_traits!(f32, usize, u32, u64, i32, i64);

impl Stats {
    pub fn new(mut data_points: Vec<f32>) -> Self {
        data_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self { data_points }
    }

    pub fn avg(&self) -> f32 {
        if self.data_points.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.data_points.iter().sum();
        sum / self.data_points.len() as f32
    }

    pub fn min(&self) -> f32 {
        *self.data_points.first().unwrap_or(&0.0)
    }

    pub fn max(&self) -> f32 {
        *self.data_points.last().unwrap_or(&0.0)
    }

    pub fn median(&self) -> f32 {
        if self.data_points.is_empty() {
            return 0.0;
        }

        let mid = self.data_points.len() / 2;
        if self.data_points.len().is_multiple_of(2) {
            (self.data_points[mid - 1] + self.data_points[mid]) / 2.0
        } else {
            self.data_points[mid]
        }
    }

    pub fn to_latex_table(&self, name: &str) -> String {
        format!(
            "{} & {} & {:.2} & {} & {} & {} \\\\",
            name,
            self.data_points.len(),
            self.avg(),
            self.median() as u32,
            self.min() as u32,
            self.max() as u32,
        )
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stats {{count: {}, avg: {:.2}, median: {:.2}, min: {:.2}, max: {:.2}}}",
            self.data_points.len(),
            self.avg(),
            self.median(),
            self.min(),
            self.max()
        )
    }
}
