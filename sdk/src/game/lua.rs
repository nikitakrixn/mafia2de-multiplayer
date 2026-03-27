//! Доступ к игровому Lua VM (Lua 5.1.2, модифицированный).
//!
//! Mafia II: DE использует Lua для скриптов миссий, UI и геймплея.
//! Этот модуль позволяет выполнять произвольный Lua-код через
//! внутренний Lua State игры.
//!
//! ВАЖНО:
//! - Lua VM не потокобезопасна
//! - Вызывать лучше из игрового потока (через main_thread dispatcher)
//! - Для smoke-тестов можно дёргать напрямую, но это рискованно

use std::ffi::{CStr, CString};

use common::logger;

use super::base;
use crate::addresses::fields;
use crate::{addresses, memory};

// =============================================================================
//  Типы Lua API
// =============================================================================

/// luaL_loadbuffer с 5-м параметром (особенность этой сборки).
/// Всегда передаём 0 в extra.
type LuaLoadBufferFn = unsafe extern "C" fn(
    usize,     // lua_State*
    *const i8, // buffer
    usize,     // size
    *const i8, // chunk name
    usize,     // extra (всегда 0)
) -> i32;

type LuaLoadStringFn = unsafe extern "C" fn(usize, *const i8) -> i32;
type LuaPcallFn = unsafe extern "C" fn(usize, i32, i32, i32) -> i32;
type LuaTolStringFn = unsafe extern "C" fn(usize, i32, *mut usize) -> *const i8;
type LuaSetTopFn = unsafe extern "C" fn(usize, i32);
type LuaGetTopFn = unsafe extern "C" fn(usize) -> i32;

// =============================================================================
//  Получение указателей на Lua API функции
// =============================================================================

fn lua_loadbuffer() -> LuaLoadBufferFn {
    unsafe { memory::fn_at(base() + addresses::functions::lua::LOADBUFFER) }
}

fn lua_loadstring() -> LuaLoadStringFn {
    unsafe { memory::fn_at(base() + addresses::functions::lua::LOADSTRING) }
}

fn lua_pcall() -> LuaPcallFn {
    unsafe { memory::fn_at(base() + addresses::functions::lua::PCALL) }
}

fn lua_tolstring() -> LuaTolStringFn {
    unsafe { memory::fn_at(base() + addresses::functions::lua::TOLSTRING) }
}

fn lua_settop() -> LuaSetTopFn {
    unsafe { memory::fn_at(base() + addresses::functions::lua::SETTOP) }
}

fn lua_gettop() -> LuaGetTopFn {
    unsafe { memory::fn_at(base() + addresses::functions::lua::GETTOP) }
}

// =============================================================================
//  Stack guard — автоматическое восстановление стека при выходе
// =============================================================================

/// Восстанавливает вершину Lua стека при выходе из scope.
/// Защищает от утечки элементов стека при ошибках.
struct LuaStackGuard {
    l: usize,
    top: i32,
}

impl Drop for LuaStackGuard {
    fn drop(&mut self) {
        unsafe { lua_settop()(self.l, self.top) };
    }
}

// =============================================================================
//  Обнаружение Lua State
// =============================================================================

/// Информация о цепочке до Lua State.
/// Полезна для диагностики — видно все промежуточные указатели.
#[derive(Debug, Clone, Copy)]
pub struct LuaChainInfo {
    pub manager: usize,
    pub vector: usize,
    pub array: usize,
    pub machine: usize,
    pub lua_state: usize,
    pub machine_count: usize,
}

/// Пройти цепочку до ScriptMachine[index].
///
/// Путь:
/// g_ScriptMachineManager -> vector (manager+0x08) ->
/// -> array[index] -> ScriptMachine -> lua_State (sm+0x70)
pub fn discover(index: usize) -> Option<LuaChainInfo> {
    unsafe {
        let manager =
            memory::read_validated_ptr(base() + addresses::globals::SCRIPT_MACHINE_MANAGER)?;

        let vector = memory::read_validated_ptr(manager + fields::script_machine_manager::VECTOR)?;

        let begin = memory::read::<usize>(vector + fields::std_vector::BEGIN)?;
        let end = memory::read::<usize>(vector + fields::std_vector::END)?;

        if begin == 0 || end < begin {
            return None;
        }

        let count = (end - begin) / 8;
        if index >= count {
            return None;
        }

        let machine = memory::read_validated_ptr(begin + index * 8)?;
        let lua_state = memory::read_validated_ptr(machine + fields::script_machine::LUA_STATE)?;

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

/// Главная script machine: "Main Game Script Machine" (index=0).
pub fn discover_main() -> Option<LuaChainInfo> {
    discover(0)
}

/// Получить lua_State* по индексу script machine.
pub fn get_lua_state(index: usize) -> Option<usize> {
    discover(index).map(|x| x.lua_state)
}

/// Получить lua_State* главной script machine.
pub fn get_main_lua_state() -> Option<usize> {
    discover_main().map(|x| x.lua_state)
}

/// Готова ли Lua VM к выполнению кода.
pub fn is_ready() -> bool {
    get_main_lua_state().is_some()
}

/// Вывести в лог цепочку указателей до Lua State.
pub fn log_chain() {
    match discover_main() {
        Some(info) => {
            logger::info(&format!(
                "Lua: manager=0x{:X} vector=0x{:X} sm=0x{:X} L=0x{:X} (machines: {})",
                info.manager, info.vector, info.machine, info.lua_state, info.machine_count,
            ));
        }
        None => logger::warn("Lua: цепочка не готова"),
    }
}

// =============================================================================
//  Выполнение Lua кода
// =============================================================================

/// Выполнить произвольный Lua chunk.
///
/// Использует luaL_loadbuffer + lua_pcall — это правильнее,
/// чем ScriptMachine::CallString, потому что мы контролируем
/// имя chunk'а и обработку ошибок.
pub fn exec(code: &str) -> Result<(), String> {
    exec_named(code, "=m2mp_console")
}

/// Выполнить chunk с указанным именем (для диагностики ошибок).
pub fn exec_named(code: &str, chunk_name: &str) -> Result<(), String> {
    let info = discover_main().ok_or_else(|| "Lua VM не готова".to_string())?;
    let l = info.lua_state;

    // Запоминаем вершину стека — восстановим при выходе
    let old_top = unsafe { lua_gettop()(l) };
    let _guard = LuaStackGuard { l, top: old_top };

    let chunk_name =
        CString::new(chunk_name).map_err(|_| "имя chunk'а содержит NUL-байт".to_string())?;

    // Загрузить chunk в стек
    let load_status = unsafe {
        lua_loadbuffer()(
            l,
            code.as_ptr() as *const i8,
            code.len(),
            chunk_name.as_ptr(),
            0, // extra — всегда 0 в этой сборке
        )
    };

    if load_status != 0 {
        return Err(format!(
            "luaL_loadbuffer({load_status}): {}",
            last_lua_error(l),
        ));
    }

    // Вызвать загруженный chunk
    let call_status = unsafe { lua_pcall()(l, 0, 0, 0) };
    if call_status != 0 {
        return Err(format!("lua_pcall({call_status}): {}", last_lua_error(l)));
    }

    Ok(())
}

/// Выполнить chunk и забрать один результат со стека.
///
/// Возвращает None если результат nil.
pub fn eval_chunk(code: &str) -> Result<Option<String>, String> {
    eval_chunk_named(code, "=m2mp_eval")
}

/// Выполнить chunk с именем и забрать один результат.
pub fn eval_chunk_named(code: &str, chunk_name: &str) -> Result<Option<String>, String> {
    let info = discover_main().ok_or_else(|| "Lua VM не готова".to_string())?;
    let l = info.lua_state;

    let old_top = unsafe { lua_gettop()(l) };
    let _guard = LuaStackGuard { l, top: old_top };

    let chunk_name =
        CString::new(chunk_name).map_err(|_| "имя chunk'а содержит NUL-байт".to_string())?;

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
            "luaL_loadbuffer({load_status}): {}",
            last_lua_error(l),
        ));
    }

    // nresults=1 — просим один результат
    let call_status = unsafe { lua_pcall()(l, 0, 1, 0) };
    if call_status != 0 {
        return Err(format!("lua_pcall({call_status}): {}", last_lua_error(l)));
    }

    // Читаем результат с вершины стека
    let result_ptr = unsafe { lua_tolstring()(l, -1, std::ptr::null_mut()) };
    if result_ptr.is_null() {
        return Ok(None);
    }

    let result = unsafe { CStr::from_ptr(result_ptr) }
        .to_string_lossy()
        .into_owned();

    Ok(Some(result))
}

/// Вычислить Lua-выражение и вернуть строковый результат.
///
/// Оборачивает выражение в `return tostring((...))`.
/// Удобно для консоли: `eval_expression("player:GetPos()")`.
pub fn eval_expression(expr: &str) -> Result<String, String> {
    let wrapped = format!("return tostring(({}))", expr);
    match eval_chunk_named(&wrapped, "=m2mp_expr")? {
        Some(s) => Ok(s),
        None => Ok("<nil>".to_string()),
    }
}

/// Fallback через `luaL_loadstring` (только для отладки).
///
/// Основной путь — `exec`/`exec_named` через `loadbuffer`.
/// Этот метод нужен если по какой-то причине `loadbuffer`
/// ведёт себя странно.
pub fn exec_via_loadstring(code: &str) -> Result<(), String> {
    let info = discover_main().ok_or_else(|| "Lua VM не готова".to_string())?;
    let l = info.lua_state;

    let old_top = unsafe { lua_gettop()(l) };
    let _guard = LuaStackGuard { l, top: old_top };

    let code = CString::new(code).map_err(|_| "код содержит NUL-байт".to_string())?;

    let load_status = unsafe { lua_loadstring()(l, code.as_ptr()) };
    if load_status != 0 {
        return Err(format!(
            "luaL_loadstring({load_status}): {}",
            last_lua_error(l),
        ));
    }

    let call_status = unsafe { lua_pcall()(l, 0, 0, 0) };
    if call_status != 0 {
        return Err(format!("lua_pcall({call_status}): {}", last_lua_error(l)));
    }

    Ok(())
}

// =============================================================================
//  Вспомогательные функции
// =============================================================================

/// Прочитать сообщение об ошибке с вершины Lua стека.
///
/// Lua кладёт ошибку на стек как строку при неудаче
/// `loadbuffer`/`pcall`. Если там не строка — вернём заглушку.
fn last_lua_error(l: usize) -> String {
    unsafe {
        let ptr = lua_tolstring()(l, -1, std::ptr::null_mut());
        if ptr.is_null() {
            "<ошибка Lua не является строкой>".to_string()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}
