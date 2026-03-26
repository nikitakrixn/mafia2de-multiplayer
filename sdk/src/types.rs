//! Базовые математические типы для SDK.
//!
//! Используются в структурах движка (`repr(C)`) и в game API.
//! `Vec3` — основной тип для позиций, направлений, скоростей.

use std::ops::{Add, Sub, Mul, Neg};

/// Трёхмерный вектор. Layout совместим с движком Illusion Engine.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };

    /// Конструктор.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// `true` если все компоненты конечны (не NaN, не Inf).
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Длина вектора (magnitude).
    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Квадрат длины (без sqrt — быстрее для сравнений).
    #[inline]
    pub fn length_sq(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Евклидово расстояние до другой точки.
    #[inline]
    pub fn distance_to(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }

    /// Квадрат расстояния (без sqrt).
    #[inline]
    pub fn distance_sq(&self, other: &Self) -> f32 {
        (*self - *other).length_sq()
    }

    /// Нормализованный вектор. `None` если длина ≈ 0.
    pub fn normalized(&self) -> Option<Self> {
        let len = self.length();
        if len > 1e-6 {
            Some(Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            })
        } else {
            None
        }
    }

    /// Скалярное произведение.
    #[inline]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Линейная интерполяция. `t=0` → `self`, `t=1` → `other`.
    #[inline]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }
}

impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}

impl Add for Vec3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// Кватернион ориентации. Layout совместим с движком.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    /// Конвертация в forward-вектор (направление взгляда).
    ///
    /// Формула подтверждена из IDA: `sub_140DA2440`.
    pub fn forward(&self) -> Vec3 {
        Vec3 {
            x: 2.0 * (self.x * self.y + self.w * self.z),
            y: 1.0 - 2.0 * (self.x * self.x + self.z * self.z),
            z: 2.0 * (self.y * self.z - self.w * self.x),
        }
    }
}