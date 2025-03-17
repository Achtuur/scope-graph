#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub enum Color {
    #[default]
    Black,
    Red,
    Blue,
    Green,
    Purple,
    Orange,
    Yellow,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Black => "black",
            Self::Red => "red",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Purple => "purple",
            Self::Yellow => "yellow",
            Self::Orange => "orange",
        })
    }
}