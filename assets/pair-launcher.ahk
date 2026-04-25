#NoEnv
#SingleInstance Force
#Warn
SendMode Input
SetTitleMatchMode, 2
SetWorkingDir, %A_ScriptDir%

LAUNCHER := A_ScriptDir "\launcher.exe"
LAUNCH_ARGS := "--client --allow-second"
WAIT_TIMEOUT_SEC := 60

HALF_W   := A_ScreenWidth // 2
GAME_H   := A_ScreenHeight * 2 // 3
CONS_Y   := GAME_H
CONS_H   := A_ScreenHeight - GAME_H

LaunchClient(1, 0, 0, HALF_W, GAME_H, 0, CONS_Y, HALF_W, CONS_H)

Sleep, 1500   ; небольшая задержка чтобы Steam handshake не конфликтовал
LaunchClient(2, HALF_W, 0, HALF_W, GAME_H, HALF_W, CONS_Y, HALF_W, CONS_H)

ExitApp

LaunchClient(idx, gx, gy, gw, gh, cx, cy, cw, ch) {
    global LAUNCHER, LAUNCH_ARGS, WAIT_TIMEOUT_SEC

    consoleNew := "m2demp-cli-" . idx
    appNew     := "m2demp-app-" . idx

    Run, %LAUNCHER% %LAUNCH_ARGS%, %A_ScriptDir%

    WinWait, m2demp-cli, , %WAIT_TIMEOUT_SEC%
    if (ErrorLevel) {
        MsgBox, 16, pair-launcher, Cant wait "m2demp-cli" %idx% (timeout).
        return
    }
    WinSetTitle, m2demp-cli, , %consoleNew%
    WinMove, %consoleNew%, , %cx%, %cy%, %cw%, %ch%

    WinWait, Mafia II, , %WAIT_TIMEOUT_SEC%
    if (ErrorLevel) {
        MsgBox, 16, pair-launcher, Cant wait game window "Mafia II" %idx% (timeout).
        return
    }
    WinSetTitle, Mafia II, , %appNew%
    WinMove, %appNew%, , %gx%, %gy%, %gw%, %gh%
    return
}
