use super::Pixel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::Right => Self::Left,
        }
    }

    pub fn can_turn(&self, other: &Self) -> bool {
        other != self && *other != self.opposite()
    }

    pub fn next_pixel(&self, mut p: Pixel, size: Pixel) -> Pixel {
        match self {
            Self::Left if p.x == 0 => p.x = size.x,
            Self::Left => p.x -= 1,
            Self::Down if p.y == size.y => p.y = 0,
            Self::Down => p.y += 1,
            Self::Up if p.y == 0 => p.y = size.y,
            Self::Up => p.y -= 1,
            Self::Right if p.x == size.x => p.x = 0,
            Self::Right => p.x += 1,
        }

        p
    }
}
