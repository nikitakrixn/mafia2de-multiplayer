/// Двумерный вектор
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Трехмерный вектор
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// Четырехмерный вектор
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
    }
}

/// Матрица 4x4
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Matrix {
    pub row1: Vector4,
    pub row2: Vector4,
    pub row3: Vector4,
    pub row4: Vector4,
}

impl Matrix {
    /// Создает единичную матрицу
    pub fn identity() -> Self {
        Self {
            row1: Vector4::new(1.0, 0.0, 0.0, 0.0),
            row2: Vector4::new(0.0, 1.0, 0.0, 0.0),
            row3: Vector4::new(0.0, 0.0, 1.0, 0.0),
            row4: Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// Структура трансформации
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector3,  // Позиция объекта
    pub rotation: Vector3,  // Поворот объекта
    pub scale: Vector3,     // Масштаб объекта
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

/// Кватернион для вращения
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32, 
    pub w: f32,
}

impl Quaternion {
    /// Создает кватернион без вращения
    pub fn identity() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
    
    /// Создает кватернион из углов Эйлера (в радианах)
    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        let cx = (x * 0.5).cos();
        let sx = (x * 0.5).sin();
        let cy = (y * 0.5).cos();
        let sy = (y * 0.5).sin();
        let cz = (z * 0.5).cos();
        let sz = (z * 0.5).sin();
        
        Self {
            x: sx * cy * cz - cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
            w: cx * cy * cz + sx * sy * sz,
        }
    }
} 