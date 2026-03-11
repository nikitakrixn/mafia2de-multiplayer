//! Обработка HUMAN / ENTITY сообщений локального игрока.
//!
//! Модуль питается от hook'а на `M2DE_EntityMessageRegistry_Broadcast`.
//! Мы фильтруем общий поток сообщений до локального игрока и превращаем
//! интересные `message_id` в `PlayerEvent`.

use sdk::{
    addresses::constants::human_messages as hm,
    game::Player,
    structures::EntityMessageHeader,
};

use crate::{
    player_events::{self, PlayerEvent},
    state::{self, GameSessionState},
};

fn is_interesting_message(id: u32) -> bool {
    matches!(
        id,
        hm::ENTER_VEHICLE
            | hm::LEAVE_VEHICLE
            | hm::ENTER_VEHICLE_DONE
            | hm::LEAVE_VEHICLE_DONE
            | hm::DAMAGE
            | hm::DEATH
            | hm::ANIM_NOTIFY
            | hm::PLAYER_WEAPON_SELECT
            | hm::PLAYER_WEAPON_HIDE
            | hm::SHOT
            | hm::WEAPON_DRAW
            | hm::WEAPON_HOLSTER
    )
}

fn is_spam_message(id: u32) -> bool {
    matches!(
        id,
        hm::HUMAN_MODE_CHANGE
            | hm::HUMAN_TICK
            | hm::HUMAN_SETTLED
    ) || (id >> 16) == 5
      || (id >> 16) == 18
}

#[allow(dead_code)]
pub fn message_name(id: u32) -> &'static str {
    match id {
        hm::ENTER_VEHICLE => "ENTER_VEHICLE",
        hm::LEAVE_VEHICLE => "LEAVE_VEHICLE",
        hm::ENTER_VEHICLE_DONE => "ENTER_VEHICLE_DONE",
        hm::LEAVE_VEHICLE_DONE => "LEAVE_VEHICLE_DONE",
        hm::DAMAGE => "DAMAGE",
        hm::DEATH => "DEATH",
        hm::ANIM_NOTIFY => "ANIM_NOTIFY",
        hm::PLAYER_WEAPON_SELECT => "PLAYER_WEAPON_SELECT",
        hm::PLAYER_WEAPON_HIDE => "PLAYER_WEAPON_HIDE",
        hm::SHOT => "SHOT",
        hm::WEAPON_DRAW => "WEAPON_DRAW",
        hm::WEAPON_HOLSTER => "WEAPON_HOLSTER",
        hm::HEAD_DAMAGE => "HEAD_DAMAGE",
        hm::BODY_DAMAGE => "BODY_DAMAGE",
        hm::KILL_DAMAGE => "KILL_DAMAGE",
        hm::HUMAN_MODE_CHANGE => "HUMAN_MODE_CHANGE",
        hm::HUMAN_TICK => "HUMAN_TICK",
        hm::HUMAN_SETTLED => "HUMAN_SETTLED",
        _ => "UNKNOWN",
    }
}

/// Обработать одно broadcast-сообщение.
///
/// Вызывается из detour'а на EntityMessageRegistry_Broadcast.
/// entity_ptr и msg_ptr приходят из движка — обычно валидны,
/// но лучше перестраховаться, чем ловить access violation в хуке.
pub fn process_broadcast(entity_ptr: usize, msg_ptr: usize) {
    // Быстрая отсечка по состоянию сессии — не тратим время
    // на разбор сообщений в меню/загрузке
    if !matches!(state::get(), GameSessionState::InGame | GameSessionState::Paused) {
        return;
    }

    // Проверяем оба указателя перед разыменованием.
    // В теории движок не должен давать мусор сюда,
    // но мы в чужом процессе — осторожность не помешает.
    // К тому же, если msg_ptr невалидный, то разыменование
    if !sdk::memory::is_valid_ptr(msg_ptr) || !sdk::memory::is_valid_ptr(entity_ptr) {
        return;
    }

    let header = unsafe { &*(msg_ptr as *const EntityMessageHeader) };
    let id = header.message_id;

    if is_spam_message(id) {
        return;
    }

    if !is_interesting_message(id) {
        return;
    }

    let Some(player) = Player::get_active() else {
        return;
    };

    if entity_ptr != player.as_ptr() {
        return;
    }

    let event = match id {
        hm::ENTER_VEHICLE => PlayerEvent::EnterVehicle,
        hm::LEAVE_VEHICLE => PlayerEvent::LeaveVehicle,
        hm::ENTER_VEHICLE_DONE => PlayerEvent::EnterVehicleDone,
        hm::LEAVE_VEHICLE_DONE => PlayerEvent::LeaveVehicleDone,
        hm::DAMAGE => PlayerEvent::Damage,
        hm::DEATH => PlayerEvent::Death,
        hm::ANIM_NOTIFY => PlayerEvent::AnimNotify,
        hm::PLAYER_WEAPON_SELECT => PlayerEvent::WeaponSelect,
        hm::PLAYER_WEAPON_HIDE => PlayerEvent::WeaponHide,
        hm::SHOT => PlayerEvent::Shot,
        hm::WEAPON_DRAW => PlayerEvent::WeaponSelect,
        hm::WEAPON_HOLSTER => PlayerEvent::WeaponHide,
        _ => return,
    };

    player_events::push(event);
}