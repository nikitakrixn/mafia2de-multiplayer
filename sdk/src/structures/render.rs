//! Структуры DX11 render path Mafia II: Definitive Edition.
//!
//! Этот модуль нужен injected-клиенту для:
//! - чтения `ID3D11Device*`
//! - чтения `ID3D11DeviceContext*`
//! - доступа к текущему `IDXGISwapChain1*`
//! - доступа к текущему `ID3D11RenderTargetView*`
//! - чтения HWND и размеров backbuffer
//!
//! Главная цепочка:
//! `M2DE_C_RenderDeviceD3D11 -> current_swapchain -> swapchain`

use std::ffi::c_void;

/// HWND в памяти игры.
///
/// Здесь оставлен как `usize`, чтобы не тащить winapi типы в layout-структуры.
pub type HWND = usize;

/// Блок init-параметров, который игра копирует внутрь render-device.
///
/// Он лежит по адресу:
/// `render_device + 0x2008`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RenderInitConfig {
    /// Первая часть init-конфига.
    ///
    /// Здесь лежат разные engine-параметры
    pub unknown_00: [u8; 0x18], // +0x00

    /// Текущая ширина рендера.
    ///
    /// Внутри `M2DE_C_RenderDeviceD3D11` это поле находится по смещению `+0x2020`.
    pub width: u32, // +0x18

    /// Текущая высота рендера.
    ///
    /// Внутри `M2DE_C_RenderDeviceD3D11` это поле находится по смещению `+0x2024`.
    pub height: u32, // +0x1C

    /// Указатель на связанную window/config структуру.
    ///
    /// В init path через эту цепочку игра получает HWND и дополнительные флаги.
    pub window_config_ptr: *mut c_void, // +0x20
}

/// Менеджер swapchain'ов.
///
/// Игра хранит несколько wrapper'ов в дереве по ключу `HWND`.
/// Для overlay обычно нужен только текущий wrapper.
///
/// Расположен по указателю:
/// `render_device + 0x27A0`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SwapChainManager {
    /// Root/sentinel узел RB-дерева.
    ///
    /// Ключ дерева: `HWND`
    /// Значение: `M2DE_SwapChainWrapper*`
    pub tree_root: *mut c_void, // +0x00

    /// Количество элементов в дереве.
    pub tree_size: u64, // +0x08

    /// `IDXGIFactory4*`
    ///
    /// Именно через этот factory создаётся raw swapchain.
    pub factory: *mut c_void, // +0x10

    /// `ID3D11Device*`
    pub device: *mut c_void, // +0x18

    /// `ID3D11DeviceContext*`
    pub context: *mut c_void, // +0x20

    /// Поддерживается ли tearing (`DXGI_FEATURE_PRESENT_ALLOW_TEARING`).
    pub tearing_supported: u8, // +0x28

    /// Debug-mode флаг render path.
    pub debug_mode: u8, // +0x29

    pub _pad_2a: [u8; 6], // +0x2A
}

/// Wrapper над одним DXGI swapchain и его основными view-объектами.
///
/// Этот wrapper создаётся для каждого окна, к которому привязывается swapchain.
/// - `swapchain` → raw `IDXGISwapChain1*`
/// - `rtv`       → текущий `ID3D11RenderTargetView*`
/// - `hwnd`      → окно игры
///
/// Расположен по указателю:
/// `render_device + 0x27A8`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SwapChainWrapper {
    /// Ширина backbuffer.
    pub width: u32, // +0x00

    /// Высота backbuffer.
    pub height: u32, // +0x04

    /// Внутренний режим swapchain.
    ///
    /// Используется движком при выборе path'а создания.
    pub swapchain_mode: u32, // +0x08

    pub _pad_0c: u32, // +0x0C

    /// HWND окна, к которому привязан swapchain.
    pub hwnd: HWND, // +0x10

    /// Raw `IDXGISwapChain1*`
    ///
    /// Это основной указатель, который нужен для hook'ов `Present`/`ResizeBuffers`.
    pub swapchain: *mut c_void, // +0x18

    /// `ID3D11Texture2D*` back buffer.
    pub back_buffer: *mut c_void, // +0x20

    /// `ID3D11Texture2D*` depth texture.
    pub depth_texture: *mut c_void, // +0x28

    /// `ID3D11DepthStencilView*`
    pub dsv: *mut c_void, // +0x30

    /// `ID3D11RenderTargetView*`
    ///
    /// Это готовый RTV текущего backbuffer.
    pub rtv: *mut c_void, // +0x38

    /// `ID3D11ShaderResourceView*`
    pub srv: *mut c_void, // +0x40
}

/// Главный DX11 renderer singleton.
///
/// Глобальный указатель:
/// `crate::addresses::globals::RENDER_DEVICE`
///
/// Основные поля, важные для overlay:
/// - `dxgi_factory`
/// - `d3d_device`
/// - `d3d_context`
/// - `swapchain_manager`
/// - `current_swapchain`
/// - `debug_annotation`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CRenderDeviceD3D11 {
    /// VTable `M2DE_C_RenderDeviceD3D11`.
    pub vtable: *const c_void, // +0x0000

    _pad_0008: [u8; 0x2000], // +0x0008

    /// Скопированный init-конфиг рендера.
    pub render_init: RenderInitConfig, // +0x2008

    _pad_2030: [u8; 0x2], // +0x2030

    /// Поддержка FL 10.0
    pub supports_fl_10_0: u8, // +0x2032

    /// Дополнительный флаг того же класса.
    pub supports_fl_10_0_dup: u8, // +0x2033

    /// Поддержка FL 10.1
    pub supports_fl_10_1: u8, // +0x2034

    /// Флаг завершённой DX-инициализации.
    pub dx_initialized: u8, // +0x2035

    _pad_2036: [u8; 0x6], // +0x2036

    /// Количество выходов текущего адаптера.
    pub adapter_output_count: u32, // +0x203C

    /// Максимальный размер текстур.
    pub max_texture_size: u32, // +0x2040

    /// Внутренний параметр фильтрации.
    pub aniso_filter_setting: u32, // +0x2044

    _pad_2048: [u8; 0x8], // +0x2048

    /// Базовый shader/resource cache.
    pub shader_cache: *mut c_void, // +0x2050

    _pad_2058: [u8; 0x18], // +0x2058

    /// DynamicVB resource.
    pub dynamic_vb: *mut c_void, // +0x2070

    /// DynamicIB resource.
    pub dynamic_ib: *mut c_void, // +0x2078

    _pad_2080: [u8; 0x128], // +0x2080

    /// Текущий режим рендера / state mode.
    pub current_state_mode: u32, // +0x21A8

    /// Вторичный state mode.
    pub current_state_mode_b: u32, // +0x21AC

    _pad_21b0: [u8; 0x5D0], // +0x21B0

    /// `IDXGIFactory1*`
    pub dxgi_factory: *mut c_void, // +0x2780

    /// `D3D_FEATURE_LEVEL`
    pub feature_level: u32, // +0x2788

    _pad_278c: [u8; 0x4], // +0x278C

    /// `ID3D11Device*`
    pub d3d_device: *mut c_void, // +0x2790

    /// `ID3D11DeviceContext*`
    pub d3d_context: *mut c_void, // +0x2798

    /// `M2DE_SwapChainManager*`
    pub swapchain_manager: *mut SwapChainManager, // +0x27A0

    /// Текущий swapchain wrapper.
    pub current_swapchain: *mut SwapChainWrapper, // +0x27A8

    /// Дополнительный указатель на активный wrapper.
    pub active_swapchain: *mut SwapChainWrapper, // +0x27B0

    _pad_27b8: [u8; 0x2954], // +0x27B8

    /// Флаги адаптера / init-флаги render path.
    pub adapter_flags: u32, // +0x510C

    /// `ID3DUserDefinedAnnotation*` или NULL.
    pub debug_annotation: *mut c_void, // +0x5110
}

const _: () = {
    use std::mem::{offset_of, size_of};

    assert!(size_of::<RenderInitConfig>() == 0x28);
    assert!(offset_of!(RenderInitConfig, width) == 0x18);
    assert!(offset_of!(RenderInitConfig, height) == 0x1C);
    assert!(offset_of!(RenderInitConfig, window_config_ptr) == 0x20);

    assert!(size_of::<SwapChainManager>() == 0x30);
    assert!(offset_of!(SwapChainManager, factory) == 0x10);
    assert!(offset_of!(SwapChainManager, device) == 0x18);
    assert!(offset_of!(SwapChainManager, context) == 0x20);
    assert!(offset_of!(SwapChainManager, tearing_supported) == 0x28);

    assert!(size_of::<SwapChainWrapper>() == 0x48);
    assert!(offset_of!(SwapChainWrapper, hwnd) == 0x10);
    assert!(offset_of!(SwapChainWrapper, swapchain) == 0x18);
    assert!(offset_of!(SwapChainWrapper, back_buffer) == 0x20);
    assert!(offset_of!(SwapChainWrapper, dsv) == 0x30);
    assert!(offset_of!(SwapChainWrapper, rtv) == 0x38);
    assert!(offset_of!(SwapChainWrapper, srv) == 0x40);

    assert!(size_of::<CRenderDeviceD3D11>() == 0x5118);
    assert!(offset_of!(CRenderDeviceD3D11, render_init) == 0x2008);
    assert!(offset_of!(CRenderDeviceD3D11, shader_cache) == 0x2050);
    assert!(offset_of!(CRenderDeviceD3D11, dynamic_vb) == 0x2070);
    assert!(offset_of!(CRenderDeviceD3D11, dynamic_ib) == 0x2078);
    assert!(offset_of!(CRenderDeviceD3D11, current_state_mode) == 0x21A8);
    assert!(offset_of!(CRenderDeviceD3D11, dxgi_factory) == 0x2780);
    assert!(offset_of!(CRenderDeviceD3D11, feature_level) == 0x2788);
    assert!(offset_of!(CRenderDeviceD3D11, d3d_device) == 0x2790);
    assert!(offset_of!(CRenderDeviceD3D11, d3d_context) == 0x2798);
    assert!(offset_of!(CRenderDeviceD3D11, swapchain_manager) == 0x27A0);
    assert!(offset_of!(CRenderDeviceD3D11, current_swapchain) == 0x27A8);
    assert!(offset_of!(CRenderDeviceD3D11, active_swapchain) == 0x27B0);
    assert!(offset_of!(CRenderDeviceD3D11, adapter_flags) == 0x510C);
    assert!(offset_of!(CRenderDeviceD3D11, debug_annotation) == 0x5110);
};