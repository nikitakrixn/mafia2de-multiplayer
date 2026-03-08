//! Доступ к игровому Lua VM в Mafia II: DE.
//!
//! Важно:
//! - Lua VM не потокобезопасна.
//! - Эти вызовы корректнее выполнять из игрового потока.
//! - Для первичной smoke-проверки можно дёргать напрямую, но
//!   для нормальной консоли следующим этапом лучше сделать main-thread dispatcher.

use std::ffi::{CStr, CString};

use common::logger;

use crate::{addresses, memory};
use super::base;

const MANAGER_VECTOR_OFFSET: usize = 0x08;
const VECTOR_BEGIN_OFFSET: usize = 0x00;
const VECTOR_END_OFFSET: usize = 0x08;
const SCRIPT_MACHINE_LUA_STATE_OFFSET: usize = 0x70;

#[derive(Debug, Clone, Copy)]
pub struct LuaChainInfo {
    pub manager: usize,
    pub vector: usize,
    pub array: usize,
    pub machine: usize,
    pub lua_state: usize,
    pub machine_count: usize,
}

type LuaLoadBufferFn = unsafe extern "C" fn(
    usize,      // lua_State*
    *const i8,  // buffer
    usize,      // size
    *const i8,  // chunk name
    usize,      // extra/mode (в этой игре всегда 0)
) -> i32;

type LuaLoadStringFn = unsafe extern "C" fn(usize, *const i8) -> i32;
type LuaPcallFn = unsafe extern "C" fn(usize, i32, i32, i32) -> i32;
type LuaTolStringFn = unsafe extern "C" fn(usize, i32, *mut usize) -> *const i8;
type LuaSetTopFn = unsafe extern "C" fn(usize, i32);
type LuaGetTopFn = unsafe extern "C" fn(usize) -> i32;

fn lua_loadbuffer() -> LuaLoadBufferFn {
    unsafe { std::mem::transmute(base() + addresses::functions::lua::LOADBUFFER) }
}

fn lua_loadstring() -> LuaLoadStringFn {
    unsafe { std::mem::transmute(base() + addresses::functions::lua::LOADSTRING) }
}

fn lua_pcall() -> LuaPcallFn {
    unsafe { crate::memory::fn_at(base() + addresses::functions::lua::PCALL) }
}

fn lua_tolstring() -> LuaTolStringFn {
    unsafe { std::mem::transmute(base() + addresses::functions::lua::TOLSTRING) }
}

fn lua_settop() -> LuaSetTopFn {
    unsafe { std::mem::transmute(base() + addresses::functions::lua::SETTOP) }
}

fn lua_gettop() -> LuaGetTopFn {
    unsafe { std::mem::transmute(base() + addresses::functions::lua::GETTOP) }
}

struct LuaStackGuard {
    l: usize,
    top: i32,
}

impl Drop for LuaStackGuard {
    fn drop(&mut self) {
        unsafe { lua_settop()(self.l, self.top) };
    }
}

/// Возвращает полную цепочку до `ScriptMachine[index]`.
pub fn discover(index: usize) -> Option<LuaChainInfo> {
    unsafe {
        let manager = memory::read_ptr(base() + addresses::globals::SCRIPT_MACHINE_MANAGER)?;
        let vector = memory::read_ptr(manager + MANAGER_VECTOR_OFFSET)?;

        let begin = memory::read_ptr_raw(vector + VECTOR_BEGIN_OFFSET)?;
        let end = memory::read_ptr_raw(vector + VECTOR_END_OFFSET)?;

        if begin == 0 || end < begin {
            return None;
        }

        let count = (end - begin) / 8;
        if index >= count {
            return None;
        }

        let machine = memory::read_ptr(begin + index * 8)?;
        let lua_state = memory::read_ptr(machine + SCRIPT_MACHINE_LUA_STATE_OFFSET)?;

        Some(LuaChainInfo {
            manager,
            vector,
            array: begin,
            machine,
            lua_state,
            machine_count: count,
        })
    }
}

/// Главная script machine: `Main Game Script Machine`.
pub fn discover_main() -> Option<LuaChainInfo> {
    discover(0)
}

pub fn get_lua_state(index: usize) -> Option<usize> {
    discover(index).map(|x| x.lua_state)
}

pub fn get_main_lua_state() -> Option<usize> {
    discover_main().map(|x| x.lua_state)
}

pub fn is_ready() -> bool {
    get_main_lua_state().is_some()
}

pub fn log_chain() {
    match discover_main() {
        Some(info) => {
            logger::info(&format!(
                "Lua chain: manager=0x{:X}, vector=0x{:X}, array=0x{:X}, sm=0x{:X}, L=0x{:X}, count={}",
                info.manager,
                info.vector,
                info.array,
                info.machine,
                info.lua_state,
                info.machine_count,
            ));
        }
        None => logger::warn("Lua chain not ready"),
    }
}

fn last_lua_error(l: usize) -> String {
    unsafe {
        let ptr = lua_tolstring()(l, -1, std::ptr::null_mut());
        if ptr.is_null() {
            "<non-string lua error>".to_string()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}

/// Выполнить произвольный Lua chunk.
///
/// Использует `luaL_loadbuffer + lua_pcall`.
///
/// Важно:
/// это правильнее, чем опираться на `ScriptMachine::CallString`.
pub fn exec(code: &str) -> Result<(), String> {
    exec_named(code, "=m2mp_console")
}

pub fn exec_named(code: &str, chunk_name: &str) -> Result<(), String> {
    let info = discover_main().ok_or_else(|| "Lua VM not ready".to_string())?;
    let l = info.lua_state;

    let old_top = unsafe { lua_gettop()(l) };
    let _guard = LuaStackGuard { l, top: old_top };

    let chunk_name =
        CString::new(chunk_name).map_err(|_| "chunk name contains interior NUL".to_string())?;

    let load_status = unsafe {
        lua_loadbuffer()(
            l,
            code.as_ptr() as *const i8,
            code.len(),
            chunk_name.as_ptr(),
            0,
        )
    };

    if load_status != 0 {
        return Err(format!(
            "luaL_loadbuffer failed ({load_status}): {}",
            last_lua_error(l)
        ));
    }

    let call_status = unsafe { lua_pcall()(l, 0, 0, 0) };
    if call_status != 0 {
        return Err(format!(
            "lua_pcall failed ({call_status}): {}",
            last_lua_error(l)
        ));
    }

    Ok(())
}

/// Выполнить chunk и забрать 1 результат со стека.
pub fn eval_chunk(code: &str) -> Result<Option<String>, String> {
    eval_chunk_named(code, "=m2mp_eval")
}

pub fn eval_chunk_named(code: &str, chunk_name: &str) -> Result<Option<String>, String> {
    let info = discover_main().ok_or_else(|| "Lua VM not ready".to_string())?;
    let l = info.lua_state;

    let old_top = unsafe { lua_gettop()(l) };
    let _guard = LuaStackGuard { l, top: old_top };

    let chunk_name =
        CString::new(chunk_name).map_err(|_| "chunk name contains interior NUL".to_string())?;

    let load_status = unsafe {
        lua_loadbuffer()(
            l,
            code.as_ptr() as *const i8,
            code.len(),
            chunk_name.as_ptr(),
            0,
        )
    };

    if load_status != 0 {
        return Err(format!(
            "luaL_loadbuffer failed ({load_status}): {}",
            last_lua_error(l)
        ));
    }

    let call_status = unsafe { lua_pcall()(l, 0, 1, 0) };
    if call_status != 0 {
        return Err(format!(
            "lua_pcall failed ({call_status}): {}",
            last_lua_error(l)
        ));
    }

    let result_ptr = unsafe { lua_tolstring()(l, -1, std::ptr::null_mut()) };
    if result_ptr.is_null() {
        return Ok(None);
    }

    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_string_lossy()
        .into_owned();

    Ok(Some(result))
}

/// Удобно для консоли: принимает выражение и всегда пытается вернуть строку.
pub fn eval_expression(expr: &str) -> Result<String, String> {
    let wrapped = format!("return tostring(({}))", expr);
    match eval_chunk_named(&wrapped, "=m2mp_expr")? {
        Some(s) => Ok(s),
        None => Ok("<nil>".to_string()),
    }
}

/// Временный fallback через `luaL_loadstring`.
///
/// Нужен только для отладки; основной путь — `exec/exec_named`.
pub fn exec_via_loadstring(code: &str) -> Result<(), String> {
    let info = discover_main().ok_or_else(|| "Lua VM not ready".to_string())?;
    let l = info.lua_state;

    let old_top = unsafe { lua_gettop()(l) };
    let _guard = LuaStackGuard { l, top: old_top };

    let code = CString::new(code).map_err(|_| "code contains interior NUL".to_string())?;

    let load_status = unsafe { lua_loadstring()(l, code.as_ptr()) };
    if load_status != 0 {
        return Err(format!(
            "luaL_loadstring failed ({load_status}): {}",
            last_lua_error(l)
        ));
    }

    let call_status = unsafe { lua_pcall()(l, 0, 0, 0) };
    if call_status != 0 {
        return Err(format!(
            "lua_pcall failed ({call_status}): {}",
            last_lua_error(l)
        ));
    }

    Ok(())
}