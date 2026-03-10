//! RVA адресов строк в памяти игры.

pub mod class_names {
    /// "C_Human"
    pub const C_HUMAN: usize = 0x18E_9508;
    /// "C_Car"
    pub const C_CAR: usize = 0x18E_8930;
    /// "C_WrapperPlayer"
    pub const C_WRAPPER_PLAYER: usize = 0x189_6E10;
    /// "C_WrapperCar"
    pub const C_WRAPPER_CAR: usize = 0x189_6C88;
    /// "C_CarManager"
    pub const C_CAR_MANAGER: usize = 0x186_F768;
    /// "C_TrafficCar"
    pub const C_TRAFFIC_CAR: usize = 0x186_C378;
    /// "CrashObjMgr"
    pub const CRASH_OBJ_MGR: usize = 0x18E_91C0;
    /// "C_GarageManager"
    pub const C_GARAGE_MANAGER: usize = 0x18D_2B70;
    /// "C_Garage"
    pub const C_GARAGE: usize = 0x190_CF18;
    /// "GetActivePlayer"
    pub const GET_ACTIVE_PLAYER: usize = 0x18D_2B38;
}

pub mod table_paths {
    /// "/tables/vehicles.tbl"
    pub const VEHICLES: usize = 0x185_6010;
    /// "/tables/car_colors.tbl"
    pub const CAR_COLORS: usize = 0x185_6330;
    /// "/sds/cars/cars_universal.sds"
    pub const CARS_UNIVERSAL_SDS: usize = 0x185_4B80;
    /// "/tables/Tyres.bin"
    pub const TYRES: usize = 0x18E_91F0;
}

pub mod garage_strings {
    /// "garage"
    pub const GARAGE: usize = 0x190_D038;
    /// "Imperial"
    pub const IMPERIAL: usize = 0x190_D040;
    /// "Payback"
    pub const PAYBACK: usize = 0x190_D050;
    /// "SpikeStrip"
    pub const SPIKE_STRIP: usize = 0x190_D470;
}

pub mod render_strings {
    /// `"D3D11 Rendering Device"`
    ///
    /// IDA: `0x1418A3C88`
    pub const D3D11_RENDERING_DEVICE: usize = 0x18A_3C88;
}