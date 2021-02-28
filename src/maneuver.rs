pub enum Move {
    Forward,
    Backward,
    Up,
    Down,
    Left,
    Right
}

impl Move {
    pub fn code(&self) -> &str {
        match self {
            Move::Forward => "f",
            Move::Backward => "b",
            Move::Up => "u",
            Move::Down => "d",
            Move::Left => "l",
            Move::Right => "r",
        }
    }
}
