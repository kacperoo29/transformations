use std::ops::{Add, Div, Mul, Sub};

use num::Num;
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};

pub type Vector2f = Vector<f32, 2>;

#[derive(Debug, Clone, Copy)]
pub struct Vector<T: Num + Copy, const N: usize> {
    data: [T; N],
}

impl<T: Num + Copy> Vector<T, 2> {
    pub fn new_with_data(x: T, y: T) -> Self {
        Self { data: [x, y] }
    }

    pub fn x(&self) -> T {
        self.data[0]
    }

    pub fn y(&self) -> T {
        self.data[1]
    }

    pub fn set_x(&mut self, x: T) {
        self.data[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.data[1] = y;
    }
}

impl Vector<f32, 2> {
    pub fn distance_to(&self, other: Vector<f32, 2>) -> f32 {
        let x = self.x() - other.x();
        let y = self.y() - other.y();
        (x * x + y * y).sqrt()
    }
}

impl<T: Num + Copy> Add<Self> for Vector<T, 2> {
    type Output = Self;

    fn add(self, other: Vector<T, 2>) -> Self {
        Self {
            data: [self.data[0] + other.data[0], self.data[1] + other.data[1]],
        }
    }
}

impl<T: Num + Copy> Sub<Self> for Vector<T, 2> {
    type Output = Self;

    fn sub(self, other: Vector<T, 2>) -> Self {
        Self {
            data: [self.data[0] - other.data[0], self.data[1] - other.data[1]],
        }
    }
}

impl<T: Num + Copy> Mul<T> for Vector<T, 2> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
        Self {
            data: [self.data[0] * other, self.data[1] * other],
        }
    }
}

impl<T: Num + Copy> Div<T> for Vector<T, 2> {
    type Output = Self;

    fn div(self, other: T) -> Self {
        Self {
            data: [self.data[0] / other, self.data[1] / other],
        }
    }
}

impl Serialize for Vector2f {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.x())?;
        seq.serialize_element(&self.y())?;
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Vector2f {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        struct Vector2fVisitor;

        impl<'de> serde::de::Visitor<'de> for Vector2fVisitor {
            type Value = Vector2f;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a 2D vector")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let x = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let y = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                Ok(Vector2f::new_with_data(x, y))
            }
        }

        deserializer.deserialize_seq(Vector2fVisitor)
    }
}

