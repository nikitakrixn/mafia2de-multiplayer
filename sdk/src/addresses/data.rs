//! RVA адресов важных data/IID объектов в памяти игры.

pub mod render_iids {
    /// `IID_IDXGIFactory4`
    ///
    /// Используется в `M2DE_CreateSwapChain`.
    ///
    /// IDA: `0x1418A38A0`
    pub const IDXGI_FACTORY4: usize = 0x18A_38A0;

    /// `IID_ID3D11Texture2D`
    ///
    /// Используется в `IDXGISwapChain1::GetBuffer(0, IID_ID3D11Texture2D, ...)`.
    ///
    /// IDA: `0x1418A38B0`
    pub const ID3D11_TEXTURE2D: usize = 0x18A_38B0;

    /// `IID_IDXGIFactory1`
    ///
    /// Используется в `CreateDXGIFactory1` и `GetParent`.
    ///
    /// IDA: `0x1418A4638`
    pub const IDXGI_FACTORY1: usize = 0x18A_4638;

    /// `IID_ID3DUserDefinedAnnotation`
    ///
    /// Используется для query у immediate context.
    ///
    /// IDA: `0x1418A4648`
    pub const ID3D_USER_DEFINED_ANNOTATION: usize = 0x18A_4648;

    /// `IID_IDXGIDevice`
    ///
    /// IDA: `0x1418A4658`
    pub const IDXGI_DEVICE: usize = 0x18A_4658;

    /// `IID_IDXGIAdapter1`
    ///
    /// IDA: `0x1418A4668`
    pub const IDXGI_ADAPTER1: usize = 0x18A_4668;

    /// `IID_IDXGIFactory5`
    ///
    /// Нужен для `CheckFeatureSupport(DXGI_FEATURE_PRESENT_ALLOW_TEARING)`.
    ///
    /// IDA: `0x1418A4AF8`
    pub const IDXGI_FACTORY5: usize = 0x18A_4AF8;
}

pub mod render_data {
    /// Значение blend-factor, используемое при initial setup.
    ///
    /// IDA: `0x141856A80`
    pub const DEFAULT_BLEND_FACTOR: usize = 0x185_6A80;
}
