//! Адреса виртуальных таблиц (RVA).

pub mod human {
    /// `C_Human` vtable.
    /// IDA: `0x1418_E94E0`
    pub const MAIN: usize = 0x18E_94E0;
}

pub mod car {
    /// `C_Car` vtable (множественное наследование — 3 vtable).
    /// IDA: `0x1418E8908`
    pub const MAIN: usize = 0x18E_8908;
    /// IDA: `0x1418_E8938`
    pub const BASE_2: usize = 0x18E_8938;
    /// IDA: `0x1418E8950`
    pub const BASE_3: usize = 0x18E_8950;
}

pub mod car_manager {
    /// IDA: `0x14186F708`
    pub const MAIN: usize = 0x186_F708;
    /// IDA: `0x14186F778`
    pub const ALT: usize = 0x186_F778;
}

pub mod garage {
    /// `C_GarageManager` vtable (базовый класс).
    /// IDA: `0x14190CD38`
    pub const GARAGE_MANAGER: usize = 0x190_CD38;

    /// `C_Garage` vtables (наследует C_GarageManager, 5 vtable).
    pub const GARAGE_1: usize = 0x190_CD78;
    pub const GARAGE_2: usize = 0x190_CF28;
    pub const GARAGE_3: usize = 0x190_CF78;
    pub const GARAGE_4: usize = 0x190_CFC8;
    pub const GARAGE_5: usize = 0x190_D2C0;

    /// `VehicleWrapper` vtable.
    /// IDA: `0x1419_0D480`
    pub const VEHICLE_WRAPPER: usize = 0x190_D480;
}

pub mod player {
    /// `C_WrapperPlayer` vtable.
    /// IDA: `0x141C33648`
    pub const WRAPPER: usize = 0x1C3_3648;
}

pub mod callbacks {
    /// `GameCallbackManager` vtable.
    /// IDA: `0x14186D208`
    pub const GAME_CALLBACK_MANAGER: usize = 0x186_D208;
}

pub mod misc {
    /// `CrashObjMgr` vtable.
    /// IDA: `0x1418E9198`
    pub const CRASH_OBJ_MGR: usize = 0x18E_9198;
}