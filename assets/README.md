# assets/ — дополнительные файлы для дистрибуции

Содержимое этой папки **копируется в `binary/`** при `cargo dist`.

## Что должно лежать здесь

### `steam_api64.dll` (обязательно)

Копия из папки игры:
```
C:\Program Files (x86)\Steam\steamapps\common\Mafia II Definitive Edition\pc\steam_api64.dll
```

У меня был этот путь, у вас может быть другой..

Этот DLL нужен для лаунчера:
- `SteamAPI_InitSafe()` — безопасный init без mutex
- получения SteamID игрока
- проверки владения игрой через `BIsAppInstalled`

### Что НЕ нужно класть

- `steam_appid.txt` — генерируется автоматически в `binary/` (содержит `1030830`)
- `m2mp_*.dll` / `*.exe` — собираются Cargo
- `*.bat` файлы — генерируются `xtask::dist`

## Почему отдельно от игры

Чтобы лаунчер и игра использовали **один и тот же** Steam API context
(один процесс инициализирует, второй наследует), DLL должен лежать там же
где запускается launcher. Хотя сама игра имеет такую же DLL в `pc/`

## Быстрая команда для копирования

```cmd
copy "C:\Program Files (x86)\Steam\steamapps\common\Mafia II Definitive Edition\pc\steam_api64.dll" assets\
```

После этого можно делать `cargo dist`.

---

## `pair-launcher.ahk` (опционально, для удобного запуска двух клиентов)

Файл `pair-launcher.ahk` — AutoHotkey v1 скрипт-обёртка над launcher.
Запускает 2 клиента подряд, ждёт появления окон, переименовывает (`m2mpde-cli-1`, `m2mpde-app-1` и `-2`) и расставляет на половины экрана.

### Как скомпилировать в exe

1. Скачать AutoHotkey v1.x: <https://www.autohotkey.com/download/>

2. Запустить `Compiler\Ahk2Exe.exe` (входит в установку AHK)

3. Параметры:
   - **Source**:    `assets\pair-launcher.ahk`
   - **Destination**: `assets\pair-launcher.exe`
   - **Base file**: `Unicode 64-bit.bin` (важно — игра x64)
   - **Custom Icon**: можно оставить пустым

4. После компиляции `cargo dist` скопирует exe в `binary/`.

### Использование

После `cargo dist` запускаешь `binary\client-pair.bat` —
он сам определит наличие `pair-launcher.exe` и использует его, либо
сделает простой запуск без раскладки окон.

Можно и напрямую: `binary\pair-launcher.exe`.