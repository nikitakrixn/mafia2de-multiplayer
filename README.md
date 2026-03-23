# Mafia II: Definitive Edition — Multiplayer

A multiplayer modification for Mafia II: Definitive Edition, written in Rust.

## Architecture

| Crate      | Type    | Description                                    |
|------------|---------|------------------------------------------------|
| `launcher` | binary  | Finds the game, injects client DLL             |
| `client`   | cdylib  | DLL injected into the game process             |
| `server`   | binary  | Dedicated multiplayer server                   |
| `sdk`      | lib     | Game structures, memory tools, pattern scanner |
| `protocol` | lib     | Network protocol shared by client and server   |
| `common`   | lib     | Logger and shared utilities                    |

## Video

[![Mafia II: Definitive Edition Multiplayer Mod](https://img.youtube.com/vi/ai1g9qWSW5I/0.jpg)](https://www.youtube.com/watch?v=ai1g9qWSW5I)

*Click the image to watch the demonstration on YouTube*