//! Доступ к DX11 render path игры.
//!
//! Через этот модуль injected-клиент получает:
//! - `ID3D11Device*` / `ID3D11DeviceContext*`
//! - `IDXGISwapChain1*` (для Present hook и overlay)
//! - `ID3D11RenderTargetView*` текущего backbuffer
//! - `HWND` и размеры рендера
//!
//! Все функции читают данные из глобального singleton'а
//! `M2DE_C_RenderDeviceD3D11`. Если рендер ещё не инициализирован —
//! возвращают `None`.

use common::logger;

use crate::{
    addresses, memory,
    memory::Ptr,
    structures::{CRenderDeviceD3D11, SwapChainManager, SwapChainWrapper},
};

use super::base;

/// Снимок текущего состояния рендера.
///
/// Это **копия только нужных полей**, а не копия всего render-device.
/// Удобно для логов, диагностики и проверки готовности overlay.
#[derive(Debug, Clone, Copy)]
pub struct RenderRuntimeInfo {
    /// Указатель на глобальный render-device singleton.
    pub render_device_ptr: usize,
    /// `IDXGIFactory1*`
    pub dxgi_factory_ptr: usize,
    /// `ID3D11Device*`
    pub d3d_device_ptr: usize,
    /// `ID3D11DeviceContext*`
    pub d3d_context_ptr: usize,
    /// `SwapChainManager*`
    pub swapchain_manager_ptr: usize,
    /// `SwapChainWrapper*`
    pub swapchain_wrapper_ptr: usize,
    /// `IDXGISwapChain1*`
    pub swapchain_ptr: usize,
    /// HWND окна игры.
    pub hwnd: usize,
    /// Ширина текущего backbuffer.
    pub width: u32,
    /// Высота текущего backbuffer.
    pub height: u32,
    /// Текущий `D3D_FEATURE_LEVEL`.
    pub feature_level: u32,
    /// Adapter/init flags render path.
    pub adapter_flags: u32,
    /// Поддерживается ли tearing.
    pub tearing_supported: bool,
    /// Завершена ли DX-инициализация.
    pub dx_initialized: bool,
    /// `ID3D11RenderTargetView*`
    pub rtv_ptr: usize,
    /// `ID3D11DepthStencilView*`
    pub dsv_ptr: usize,
    /// `ID3D11Texture2D*` backbuffer.
    pub back_buffer_ptr: usize,
}

// =============================================================================
//  Внутренние typed helpers
// =============================================================================

/// Получить typed reference по адресу.
///
/// Использует [`Ptr<T>`] из `memory.rs`, но возвращает ссылку
/// напрямую из raw pointer, не копируя структуру.
///
/// # Safety
///
/// - `addr` должен указывать на валидный живой объект типа `T`.
/// - Вызывающий код должен понимать lifetime объекта движка.
unsafe fn ref_from_addr<T>(addr: usize) -> Option<&'static T> {
    if !memory::is_valid_ptr(addr) {
        return None;
    }

    let ptr = Ptr::<T>::new(addr);
    debug_assert!(
        ptr.raw().is_aligned(),
        "unaligned ref_from_addr: 0x{addr:X}"
    );

    Some(unsafe { &*ptr.raw() })
}

// =============================================================================
//  Чтение основных объектов
// =============================================================================

/// Указатель на глобальный `M2DE_C_RenderDeviceD3D11`.
pub fn get_render_device_ptr() -> Option<usize> {
    unsafe { memory::read_ptr(base() + addresses::globals::RENDER_DEVICE) }
}

/// Typed reference на глобальный render-device.
///
/// ## Почему ссылка, а не копия
///
/// `CRenderDeviceD3D11` очень большой (`0x5118` байт), поэтому
/// для internal DLL правильнее читать его как `&CRenderDeviceD3D11`.
pub fn get_render_device() -> Option<&'static CRenderDeviceD3D11> {
    let ptr = get_render_device_ptr()?;
    unsafe { ref_from_addr::<CRenderDeviceD3D11>(ptr) }
}

/// Указатель на `SwapChainManager`.
pub fn get_swapchain_manager_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.swapchain_manager as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// Typed reference на `SwapChainManager`.
pub fn get_swapchain_manager() -> Option<&'static SwapChainManager> {
    let ptr = get_swapchain_manager_ptr()?;
    unsafe { ref_from_addr::<SwapChainManager>(ptr) }
}

/// Указатель на текущий `SwapChainWrapper`.
pub fn get_swapchain_wrapper_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.current_swapchain as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// Typed reference на текущий `SwapChainWrapper`.
pub fn get_swapchain_wrapper() -> Option<&'static SwapChainWrapper> {
    let ptr = get_swapchain_wrapper_ptr()?;
    unsafe { ref_from_addr::<SwapChainWrapper>(ptr) }
}

// =============================================================================
//  Отдельные указатели на DX-объекты
// =============================================================================

/// `IDXGISwapChain1*` — нужен для Present hook.
pub fn get_swapchain_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.swapchain as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// `ID3D11Device*`
pub fn get_d3d_device_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.d3d_device as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// `ID3D11DeviceContext*`
pub fn get_d3d_context_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.d3d_context as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// `IDXGIFactory1*`
pub fn get_dxgi_factory_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.dxgi_factory as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// `ID3D11RenderTargetView*` текущего backbuffer.
pub fn get_backbuffer_rtv_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.rtv as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// `ID3D11DepthStencilView*`
pub fn get_backbuffer_dsv_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.dsv as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// `ID3D11Texture2D*` backbuffer.
pub fn get_backbuffer_texture_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.back_buffer as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

// =============================================================================
//  Параметры рендера
// =============================================================================

/// `HWND` окна игры.
pub fn get_hwnd() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    (sc.hwnd != 0).then_some(sc.hwnd)
}

/// Размеры из текущего swapchain wrapper (backbuffer).
pub fn get_swapchain_size() -> Option<(u32, u32)> {
    let sc = get_swapchain_wrapper()?;
    Some((sc.width, sc.height))
}

/// Размеры из render init config.
///
/// В некоторых случаях могут отличаться от текущего swapchain
/// (например, сразу после resize, до полного reinit).
pub fn get_render_size() -> Option<(u32, u32)> {
    let rd = get_render_device()?;
    Some((rd.render_init.width, rd.render_init.height))
}

/// Текущий `D3D_FEATURE_LEVEL`.
///
/// Примеры:
/// - `0xB000` = 11.0
/// - `0xB100` = 11.1
pub fn get_feature_level() -> Option<u32> {
    let rd = get_render_device()?;
    Some(rd.feature_level)
}

/// Поддерживается ли tearing (`DXGI_FEATURE_PRESENT_ALLOW_TEARING`).
pub fn is_tearing_supported() -> Option<bool> {
    let mgr = get_swapchain_manager()?;
    Some(mgr.tearing_supported != 0)
}

// =============================================================================
//  Проверка готовности
// =============================================================================

/// Готов ли render path для overlay.
///
/// Проверяет всю цепочку:
/// `render device -> device -> context -> swapchain -> RTV`
pub fn is_overlay_ready() -> bool {
    get_render_device_ptr().is_some()
        && get_d3d_device_ptr().is_some()
        && get_d3d_context_ptr().is_some()
        && get_swapchain_ptr().is_some()
        && get_backbuffer_rtv_ptr().is_some()
}

/// `D3D_FEATURE_LEVEL` -> человекочитаемое имя.
pub fn feature_level_name(level: u32) -> &'static str {
    match level {
        0x9100 => "9.1",
        0x9200 => "9.2",
        0x9300 => "9.3",
        0xA000 => "10.0",
        0xA100 => "10.1",
        0xB000 => "11.0",
        0xB100 => "11.1",
        _ => "неизвестно",
    }
}

// =============================================================================
//  Диагностика
// =============================================================================

/// Собрать снимок текущего состояния рендера.
///
/// Здесь мы **осознанно** копируем только небольшой набор полей,
/// чтобы безопасно логировать и передавать состояние наружу.
pub fn get_runtime_info() -> Option<RenderRuntimeInfo> {
    let rd = get_render_device()?;
    let mgr = get_swapchain_manager()?;
    let sc = get_swapchain_wrapper()?;

    Some(RenderRuntimeInfo {
        render_device_ptr: rd as *const CRenderDeviceD3D11 as usize,
        dxgi_factory_ptr: rd.dxgi_factory as usize,
        d3d_device_ptr: rd.d3d_device as usize,
        d3d_context_ptr: rd.d3d_context as usize,
        swapchain_manager_ptr: rd.swapchain_manager as usize,
        swapchain_wrapper_ptr: rd.current_swapchain as usize,
        swapchain_ptr: sc.swapchain as usize,
        hwnd: sc.hwnd,
        width: sc.width,
        height: sc.height,
        feature_level: rd.feature_level,
        adapter_flags: rd.adapter_flags,
        tearing_supported: mgr.tearing_supported != 0,
        dx_initialized: rd.dx_initialized != 0,
        rtv_ptr: sc.rtv as usize,
        dsv_ptr: sc.dsv as usize,
        back_buffer_ptr: sc.back_buffer as usize,
    })
}

/// Вывести в лог состояние рендера.
///
/// Удобно вызывать после инжекта и после загрузки мира.
pub fn dump_runtime_info() {
    let Some(info) = get_runtime_info() else {
        logger::warn("[render] render path ещё не готов");
        return;
    };

    logger::info(&format!(
        "[render] device=0x{:X} factory=0x{:X} d3d_device=0x{:X} context=0x{:X}",
        info.render_device_ptr, info.dxgi_factory_ptr, info.d3d_device_ptr, info.d3d_context_ptr,
    ));

    logger::info(&format!(
        "[render] wrapper=0x{:X} swapchain=0x{:X} hwnd=0x{:X}",
        info.swapchain_wrapper_ptr, info.swapchain_ptr, info.hwnd,
    ));

    logger::info(&format!(
        "[render] {}x{} rtv=0x{:X} dsv=0x{:X} backbuffer=0x{:X}",
        info.width, info.height, info.rtv_ptr, info.dsv_ptr, info.back_buffer_ptr,
    ));

    logger::info(&format!(
        "[render] feature_level={} flags=0x{:X} tearing={} dx_init={}",
        feature_level_name(info.feature_level),
        info.adapter_flags,
        info.tearing_supported,
        info.dx_initialized,
    ));
}
