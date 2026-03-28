//! Транспорт.

use crate::game::car::Car;
use crate::memory;

use super::Player;

impl Player {
    pub fn is_in_vehicle(&self) -> Option<bool> {
        unsafe { self.human().map(|h| !h.actor.owner.is_null()) }
    }

    /// Указатель на vehicle entity из `CHuman.actor.owner` (+0x80).
    ///
    /// Проверяет что результат — валидный указатель на объект
    /// с читаемым CEntity заголовком (минимум 0x30 байт).
    /// Возвращает `None` если owner == NULL, невалиден или unreadable.
    pub fn get_vehicle_ptr(&self) -> Option<usize> {
        let owner = unsafe { self.human()?.actor.owner as usize };
        if !memory::is_valid_ptr(owner) {
            return None;
        }
        // Проверяем что по адресу реально можно прочитать хотя бы CEntity header
        if !memory::is_readable(owner, 0x30) {
            return None;
        }
        Some(owner)
    }

    /// Получить обёртку `Car` если игрок в машине типа C_Car (ft=0x12).
    ///
    /// Возвращает `None` если:
    /// - игрок не в транспорте
    /// - транспорт не C_Car (например C_CarVehicle ft=0x70)
    /// - указатель невалиден или память unreadable
    pub fn get_vehicle_as_car(&self) -> Option<Car> {
        Car::from_ptr(self.get_vehicle_ptr()?)
    }
}