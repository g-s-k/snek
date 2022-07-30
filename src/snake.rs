use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use super::Direction;

type Value = u16;
pub type Raster = HashMap<Value, BTreeSet<Value>>;

const INITIAL_LENGTH: Value = 4;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Pixel {
    pub x: Value,
    pub y: Value,
}

impl Pixel {
    pub fn r#in(&self, m: &Raster) -> bool {
        if let Some(v) = m.get(&self.y) {
            return v.get(&self.x).is_some();
        }

        false
    }

    /// get random(-ish) pixel based on system time
    pub fn randomize(size: Self) -> Self {
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        Pixel {
            x: (seed.as_nanos() % (size.x as u128)) as u16,
            y: (seed.as_millis() % (size.y as u128)) as u16,
        }
    }
}

#[derive(Debug)]
pub struct Snake {
    head: Pixel,
    facing: Direction,
    to_first_bend: Value,
    bends: VecDeque<(Value, Direction)>,
}

impl Snake {
    pub fn new(head: Pixel) -> Self {
        Self {
            head,
            facing: Direction::Right,
            to_first_bend: INITIAL_LENGTH,
            bends: VecDeque::new(),
        }
    }

    pub fn head(&self) -> Pixel {
        self.head
    }

    pub fn rasterize(&self, term_size: Pixel) -> Raster {
        let mut current_direction = self.facing.opposite();
        let mut bend_queue = self.bends.iter();

        let mut current_pixel = current_direction.next_pixel(self.head, term_size);
        let mut current_length = self.to_first_bend - 1;

        let mut out: Raster = HashMap::new();
        loop {
            for _ in 0..current_length {
                if let Some(v) = out.get_mut(&current_pixel.y) {
                    v.insert(current_pixel.x);
                } else {
                    let mut set = BTreeSet::new();
                    set.insert(current_pixel.x);
                    out.insert(current_pixel.y, set);
                }

                current_pixel = current_direction.next_pixel(current_pixel, term_size);
            }

            match bend_queue.next() {
                None => return out,
                Some((length, direction)) => {
                    current_length = *length;
                    current_direction = *direction;
                }
            }
        }
    }

    pub fn turn(&mut self, direction: Direction) {
        if !self.facing.can_turn(&direction) {
            return;
        }

        self.bends
            .push_front((self.to_first_bend, self.facing.opposite()));
        self.facing = direction;
        self.to_first_bend = 0;
    }

    pub fn advance(&mut self, term_size: Pixel) {
        self.head = self.facing.next_pixel(self.head, term_size);

        if let Some(b) = self.bends.back_mut() {
            self.to_first_bend += 1;

            if b.0 == 1 {
                self.bends.pop_back();
            } else {
                b.0 -= 1;
            }
        }
    }

    pub fn grow(&mut self) {
        if let Some(b) = self.bends.back_mut() {
            b.0 += 1;
        } else {
            self.to_first_bend += 1;
        }
    }

    pub fn len(&self) -> Value {
        self.bends
            .iter()
            .fold(self.to_first_bend, |sum, (l, _)| sum + l)
            - INITIAL_LENGTH
    }
}
