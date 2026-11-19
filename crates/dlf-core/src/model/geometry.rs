//! # Geometri dan Transformasi Visual (Geometry & Visual Transforms)
//!
//! Modul ini mendefinisikan tipe koordinat titik (`Point`) dan orientasi rotasi (`Rotation`)
//! untuk entitas-entitas grafis pada dokumen rangkaian (*Authoring Document*).

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Representasi koordinat titik 2D berbasis floating-point 64-bit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Koordinat sumbu X.
    pub x: f64,
    /// Koordinat sumbu Y.
    pub y: f64,
}

impl Point {
    /// Titik origin (0.0, 0.0).
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Membuat instance `Point` baru dari koordinat x dan y.
    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Menghitung jarak Euclidean kuadrat ke titik lain (tanpa akar kuadrat untuk efisiensi komparasi).
    #[inline]
    pub fn distance_squared_to(self, other: Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

/// Orientasi rotasi 90 derajat searah jarum jam untuk komponen digital.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rotation {
    /// 0 derajat (menghadap kanan / timur).
    #[default]
    R0 = 0,
    /// 90 derajat searah jarum jam (menghadap bawah / selatan).
    R90 = 1,
    /// 180 derajat (menghadap kiri / barat).
    R180 = 2,
    /// 270 derajat searah jarum jam (menghadap atas / utara).
    R270 = 3,
}

impl Serialize for Rotation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.to_degrees())
    }
}

impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RotationVisitor;

        impl<'de> serde::de::Visitor<'de> for RotationVisitor {
            type Value = Rotation;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("angka derajat (0, 90, 180, 270) atau kuadran (0..=3)")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    0 | 360 => Ok(Rotation::R0),
                    90 | 1 => Ok(Rotation::R90),
                    180 | 2 => Ok(Rotation::R180),
                    270 | 3 => Ok(Rotation::R270),
                    _ => Err(E::custom(format!("Nilai rotasi tidak valid: {}", v))),
                }
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v < 0 {
                    Err(E::custom("Nilai rotasi tidak boleh negatif"))
                } else {
                    self.visit_u64(v as u64)
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.to_ascii_uppercase().as_str() {
                    "0" | "R0" | "0 DEG" => Ok(Rotation::R0),
                    "90" | "R90" | "90 DEG" => Ok(Rotation::R90),
                    "180" | "R180" | "180 DEG" => Ok(Rotation::R180),
                    "270" | "R270" | "270 DEG" => Ok(Rotation::R270),
                    _ => Err(E::custom(format!("String rotasi tidak valid: '{}'", v))),
                }
            }
        }

        deserializer.deserialize_any(RotationVisitor)
    }
}

impl Rotation {
    /// Mengubah sudut diskrit (0, 90, 180, 270) menjadi nilai rotasi kuadran.
    pub fn from_degrees(degrees: u32) -> Self {
        match (degrees % 360) / 90 {
            0 => Self::R0,
            1 => Self::R90,
            2 => Self::R180,
            _ => Self::R270,
        }
    }

    /// Mengembalikan sudut dalam satuan derajat.
    pub const fn to_degrees(self) -> u32 {
        match self {
            Self::R0 => 0,
            Self::R90 => 90,
            Self::R180 => 180,
            Self::R270 => 270,
        }
    }
}

impl Display for Rotation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}°", self.to_degrees())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance_squared_to(p2), 25.0);
    }

    #[test]
    fn test_rotation_degrees() {
        assert_eq!(Rotation::from_degrees(0), Rotation::R0);
        assert_eq!(Rotation::from_degrees(90), Rotation::R90);
        assert_eq!(Rotation::from_degrees(180), Rotation::R180);
        assert_eq!(Rotation::from_degrees(270), Rotation::R270);
        assert_eq!(Rotation::from_degrees(360), Rotation::R0);
        assert_eq!(Rotation::from_degrees(450), Rotation::R90);
    }
}
