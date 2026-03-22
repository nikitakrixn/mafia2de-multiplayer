//! Доступ к DX11 render path игры.
//!
//! Через этот модуль injected-клиент получает:
//! - ID3D11Device* / ID3D11DeviceContext*
//! - IDXGISwapChain1* (для Present hook и overlay)
//! - ID3D11RenderTargetView* текущего backbuffer
//! - HWND и размеры рендера
//!
//! Все функции читают данные из глобального singleton'а
//! M2DE_C_RenderDeviceD3D11. Если рендер ещё не инициализирован —
//! возвращают None.

use common::logger;

use crate::{
    addresses, memory,
    structures::{CRenderDeviceD3D11, SwapChainManager, SwapChainWrapper},
};

use super::base;

/// Снимок текущего состояния рендера.
///
/// Удобно для логов, диагностики и проверки готовности overlay.
/// Все поля — копии на момент вызова, не живые указатели.
#[derive(Debug, Clone, Copy)]
pub struct RenderRuntimeInfo {
    pub render_device_ptr: usize,
    pub dxgi_factory_ptr: usize,
    pub d3d_device_ptr: usize,
    pub d3d_context_ptr: usize,
    pub swapchain_manager_ptr: usize,
    pub swapchain_wrapper_ptr: usize,
    pub swapchain_ptr: usize,
    pub hwnd: usize,
    pub width: u32,
    pub height: u32,
    pub feature_level: u32,
    pub adapter_flags: u32,
    pub tearing_supported: bool,
    pub dx_initialized: bool,
    pub rtv_ptr: usize,
    pub dsv_ptr: usize,
    pub back_buffer_ptr: usize,
}

// =============================================================================
//  Чтение основных объектов
// =============================================================================

/// Указатель на глобальный M2DE_C_RenderDeviceD3D11.
pub fn get_render_device_ptr() -> Option<usize> {
    unsafe { memory::read_ptr(base() + addresses::globals::RENDER_DEVICE) }
}

/// Прочитать копию структуры C_RenderDeviceD3D11.
/// Структура большая (~20KB) — не вызывать в горячем пути.
pub fn get_render_device() -> Option<CRenderDeviceD3D11> {
    let ptr = get_render_device_ptr()?;
    unsafe { memory::read_value::<CRenderDeviceD3D11>(ptr) }
}

/// Указатель на SwapChainManager.
pub fn get_swapchain_manager_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.swapchain_manager as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// Прочитать SwapChainManager.
pub fn get_swapchain_manager() -> Option<SwapChainManager> {
    let ptr = get_swapchain_manager_ptr()?;
    unsafe { memory::read_value::<SwapChainManager>(ptr) }
}

/// Указатель на текущий SwapChainWrapper.
pub fn get_swapchain_wrapper_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.current_swapchain as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// Прочитать текущий SwapChainWrapper.
pub fn get_swapchain_wrapper() -> Option<SwapChainWrapper> {
    let ptr = get_swapchain_wrapper_ptr()?;
    unsafe { memory::read_value::<SwapChainWrapper>(ptr) }
}

// =============================================================================
//  Отдельные указатели на DX-объекты
// =============================================================================

/// IDXGISwapChain1* — нужен для Present hook.
pub fn get_swapchain_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.swapchain as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// ID3D11Device*
pub fn get_d3d_device_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.d3d_device as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// ID3D11DeviceContext*
pub fn get_d3d_context_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.d3d_context as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// IDXGIFactory1*
pub fn get_dxgi_factory_ptr() -> Option<usize> {
    let rd = get_render_device()?;
    let ptr = rd.dxgi_factory as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// ID3D11RenderTargetView* текущего backbuffer.
pub fn get_backbuffer_rtv_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.rtv as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// ID3D11DepthStencilView*
pub fn get_backbuffer_dsv_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.dsv as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

/// ID3D11Texture2D* backbuffer.
pub fn get_backbuffer_texture_ptr() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    let ptr = sc.back_buffer as usize;
    memory::is_valid_ptr(ptr).then_some(ptr)
}

// =============================================================================
//  Параметры рендера
// =============================================================================

/// HWND окна игры.
pub fn get_hwnd() -> Option<usize> {
    let sc = get_swapchain_wrapper()?;
    (sc.hwnd != 0).then_some(sc.hwnd)
}

/// Размеры из текущего swapchain wrapper (backbuffer).
pub fn get_swapchain_size() -> Option<(u32, u32)> {
    let sc = get_swapchain_wrapper()?;
    Some((sc.width, sc.height))
}

/// Размеры из render init config (могут отличаться при resize).
pub fn get_render_size() -> Option<(u32, u32)> {
    let rd = get_render_device()?;
    Some((rd.render_init.width, rd.render_init.height))
}

/// Текущий D3D_FEATURE_LEVEL (0xB000 = 11.0, 0xB100 = 11.1).
pub fn get_feature_level() -> Option<u32> {
    let rd = get_render_device()?;
    Some(rd.feature_level)
}

/// Поддерживается ли tearing (DXGI_FEATURE_PRESENT_ALLOW_TEARING).
pub fn is_tearing_supported() -> Option<bool> {
    let mgr = get_swapchain_manager()?;
    Some(mgr.tearing_supported != 0)
}

// =============================================================================
//  Проверка готовности
// =============================================================================

/// Готов ли render path для overlay.
///
/// Проверяет всю цепочку: render device -> device -> context ->
/// -> swapchain -> RTV. Если хотя бы одно звено NULL — не готов.
pub fn is_overlay_ready() -> bool {
    get_render_device_ptr().is_some()
        && get_d3d_device_ptr().is_some()
        && get_d3d_context_ptr().is_some()
        && get_swapchain_ptr().is_some()
        && get_backbuffer_rtv_ptr().is_some()
}

/// D3D_FEATURE_LEVEL -> человекочитаемое имя.
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

/// Собрать полный снимок состояния рендера.
pub fn get_runtime_info() -> Option<RenderRuntimeInfo> {
    let render_device_ptr = get_render_device_ptr()?;
    let rd = get_render_device()?;
    let mgr = get_swapchain_manager()?;
    let sc = get_swapchain_wrapper()?;

    Some(RenderRuntimeInfo {
        render_device_ptr,
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
/// Удобно вызывать после инжекта и после загрузки мира —
/// сразу видно какой device, какой feature level,
/// работает ли tearing и т.д.
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
