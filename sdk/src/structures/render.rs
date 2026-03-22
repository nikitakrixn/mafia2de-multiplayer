//! Структуры DX11 render path Mafia II: Definitive Edition.
//!
//! Нужны injected-клиенту для:
//! - ID3D11Device* / ID3D11DeviceContext*
//! - IDXGISwapChain1* (Present hook)
//! - ID3D11RenderTargetView* (overlay рендеринг)
//! - HWND и размеры backbuffer
//!
//! Главная цепочка:
//! RENDER_DEVICE -> current_swapchain -> swapchain (raw IDXGISwapChain1*)

use crate::macros::assert_layout;
use std::ffi::c_void;

/// HWND как usize — чтобы не тащить winapi типы в layout-структуры.
pub type HWND = usize;

/// Блок init-параметров рендера.
///
/// Копируется внутрь render-device при создании.
/// Расположен по смещению: `render_device + 0x2008`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RenderInitConfig {
    /// Engine-параметры первой части init-конфига.
    pub unknown_00: [u8; 0x18], // +0x00
    /// Текущая ширина рендера (render_device + 0x2020).
    pub width: u32, // +0x18
    /// Текущая высота рендера (render_device + 0x2024).
    pub height: u32, // +0x1C
    /// Указатель на window/config структуру.
    pub window_config_ptr: *mut c_void, // +0x20
}

/// Менеджер swapchain'ов.
///
/// Хранит wrapper'ы в RB-дереве по ключу HWND.
/// Для overlay обычно нужен только текущий wrapper.
/// Расположен по указателю: `render_device + 0x27A0`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SwapChainManager {
    /// Root/sentinel узел RB-дерева (ключ: HWND, значение: SwapChainWrapper*).
    pub tree_root: *mut c_void, // +0x00
    /// Количество элементов в дереве.
    pub tree_size: u64, // +0x08
    /// IDXGIFactory4* — через него создаётся raw swapchain.
    pub factory: *mut c_void, // +0x10
    /// ID3D11Device*
    pub device: *mut c_void, // +0x18
    /// ID3D11DeviceContext*
    pub context: *mut c_void, // +0x20
    /// Поддержка DXGI_FEATURE_PRESENT_ALLOW_TEARING.
    pub tearing_supported: u8, // +0x28
    /// Debug-mode флаг render path.
    pub debug_mode: u8, // +0x29
    pub _pad_2a: [u8; 6], // +0x2A
}

/// Wrapper над одним DXGI swapchain.
///
/// Создаётся для каждого HWND. Содержит:
/// - raw IDXGISwapChain1* (для Present hook)
/// - back buffer texture + RTV/DSV/SRV
/// - размеры и HWND окна
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SwapChainWrapper {
    /// Ширина backbuffer.
    pub width: u32, // +0x00
    /// Высота backbuffer.
    pub height: u32, // +0x04
    /// Внутренний режим создания swapchain.
    pub swapchain_mode: u32, // +0x08
    pub _pad_0c: u32, // +0x0C
    /// HWND окна, к которому привязан swapchain.
    pub hwnd: HWND, // +0x10
    /// IDXGISwapChain1* — для hook'ов Present/ResizeBuffers.
    pub swapchain: *mut c_void, // +0x18
    /// ID3D11Texture2D* back buffer.
    pub back_buffer: *mut c_void, // +0x20
    /// ID3D11Texture2D* depth texture.
    pub depth_texture: *mut c_void, // +0x28
    /// ID3D11DepthStencilView*
    pub dsv: *mut c_void, // +0x30
    /// ID3D11RenderTargetView* текущего backbuffer.
    pub rtv: *mut c_void, // +0x38
    /// ID3D11ShaderResourceView*
    pub srv: *mut c_void, // +0x40
}

/// Главный DX11 renderer singleton.
///
/// Глобальный указатель: `addresses::globals::RENDER_DEVICE`
///
/// Через этот объект можно получить всё, что нужно для overlay:
/// device, context, swapchain, RTV, HWND, размеры.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CRenderDeviceD3D11 {
    /// VTable M2DE_C_RenderDeviceD3D11.
    pub vtable: *const c_void, // +0x0000
    _pad_0008: [u8; 0x2000], // +0x0008

    /// Скопированный init-конфиг (ширина, высота, window ptr).
    pub render_init: RenderInitConfig, // +0x2008
    _pad_2030: [u8; 0x2], // +0x2030

    /// Поддержка Feature Level 10.0.
    pub supports_fl_10_0: u8, // +0x2032
    pub supports_fl_10_0_dup: u8, // +0x2033
    /// Поддержка Feature Level 10.1.
    pub supports_fl_10_1: u8, // +0x2034
    /// DX-инициализация завершена (1 = да).
    pub dx_initialized: u8, // +0x2035
    _pad_2036: [u8; 0x6],         // +0x2036

    /// Количество выходов текущего адаптера.
    pub adapter_output_count: u32, // +0x203C
    /// Максимальный размер текстур.
    pub max_texture_size: u32, // +0x2040
    /// Настройка aniso-фильтрации.
    pub aniso_filter_setting: u32, // +0x2044
    _pad_2048: [u8; 0x8], // +0x2048

    /// Shader/resource cache.
    pub shader_cache: *mut c_void, // +0x2050
    _pad_2058: [u8; 0x18], // +0x2058

    /// Dynamic vertex buffer ("RenderDeviceBase::DynamicVB").
    pub dynamic_vb: *mut c_void, // +0x2070
    /// Dynamic index buffer ("RenderDeviceBase::DynamicIB").
    pub dynamic_ib: *mut c_void, // +0x2078
    _pad_2080: [u8; 0x128], // +0x2080

    /// Текущий режим/профиль рендера.
    pub current_state_mode: u32, // +0x21A8
    /// Вторичный state mode.
    pub current_state_mode_b: u32, // +0x21AC
    _pad_21b0: [u8; 0x5D0], // +0x21B0

    /// IDXGIFactory1*
    pub dxgi_factory: *mut c_void, // +0x2780
    /// D3D_FEATURE_LEVEL (0xB000 = 11_0, 0xB100 = 11_1).
    pub feature_level: u32, // +0x2788
    _pad_278c: [u8; 0x4], // +0x278C

    /// ID3D11Device*
    pub d3d_device: *mut c_void, // +0x2790
    /// ID3D11DeviceContext*
    pub d3d_context: *mut c_void, // +0x2798
    /// SwapChainManager* — менеджер всех swapchain'ов.
    pub swapchain_manager: *mut SwapChainManager, // +0x27A0
    /// Текущий активный SwapChainWrapper.
    pub current_swapchain: *mut SwapChainWrapper, // +0x27A8
    /// Дополнительный указатель на активный wrapper.
    pub active_swapchain: *mut SwapChainWrapper, // +0x27B0
    _pad_27b8: [u8; 0x2954], // +0x27B8

    /// Флаги адаптера / init-флаги render path.
    pub adapter_flags: u32, // +0x510C
    /// ID3DUserDefinedAnnotation* (NULL в retail).
    pub debug_annotation: *mut c_void, // +0x5110
}

// =============================================================================
//  Compile-time проверки layout'ов
// =============================================================================

assert_layout!(RenderInitConfig, size = 0x28, {
    width             == 0x18,
    height            == 0x1C,
    window_config_ptr == 0x20,
});

assert_layout!(SwapChainManager, size = 0x30, {
    factory           == 0x10,
    device            == 0x18,
    context           == 0x20,
    tearing_supported == 0x28,
});

assert_layout!(SwapChainWrapper, size = 0x48, {
    hwnd        == 0x10,
    swapchain   == 0x18,
    back_buffer == 0x20,
    dsv         == 0x30,
    rtv         == 0x38,
    srv         == 0x40,
});

assert_layout!(CRenderDeviceD3D11, size = 0x5118, {
    render_init        == 0x2008,
    shader_cache       == 0x2050,
    dynamic_vb         == 0x2070,
    dynamic_ib         == 0x2078,
    current_state_mode == 0x21A8,
    dxgi_factory       == 0x2780,
    feature_level      == 0x2788,
    d3d_device         == 0x2790,
    d3d_context        == 0x2798,
    swapchain_manager  == 0x27A0,
    current_swapchain  == 0x27A8,
    active_swapchain   == 0x27B0,
    adapter_flags      == 0x510C,
    debug_annotation   == 0x5110,
});
