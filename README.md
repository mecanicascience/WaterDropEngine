<p align="center">
    <img src="imgs/logo.png" height="400" alt="logo"/>
</p>

# WaterDropEngine

![name](https://img.shields.io/badge/Made_by-MecanicaScience-9cf)
![language](https://img.shields.io/badge/Language-Rust-red)
![license](https://img.shields.io/github/license/mecanicascience/WaterDropEngine)
![stars](https://img.shields.io/github/stars/MecanicaScience)

<br />

## Presentation
WaterDropEngine (<i>WDE</i>) is a 3D rendering engine in Rust mainly designed for homemade computer graphics and physics simulations, based on the WebGPU api (<i>using the Wgpu Rust keybinds</i>).

<br/>

## Running the engine
To start running WaterDropEngine, you will need to have Rust installed on your computer. If you don't have it, you can install it by following the instructions on the [official website](https://www.rust-lang.org/tools/install).

Then, you'll need to fork the project into your own GitHub account, clone the repository using `git clone https://github.com/mecanicascience/WaterDropEngine.git`.

Once all of these is done, you are ready to start using WaterDropEngine!
If you use `Visual Studio Code`, you can open the current repository by running `code .` in the terminal in the root of the project. Then 4 different running configurations are available:
- `Trace editor` to run the editor in debug mode with tracing enabled.
- `Debug editor` to run the editor in debug mode without tracing.
- `Release editor` to run the editor in release mode.
- `Trace game` to run the game in debug mode with tracing enabled.
- `Debug game` to run the game in debug mode without tracing.
- `Release game` to run the game in release mode.


<br />

## Engine structure
The engine is divided into several crates in a top-to-bottom layer format. Each layer can only depend on the layers below it. This allow easier integration and avoid cross-dependencies. The main parts are the following:

```
core/
├───────────────── Editor ───────────────────
|   └── editor/
|
├────────────────── Game ───────────────────
|   └── game/
|
├────────────────── Core ───────────────────
|   └── core/
|      └── ecs/
|
├──────────────── Resources ────────────────
|   └── resources/
|
├─────────── Third Party Systems ───────────
|   └── third_party/
|      └── wgpu/
├──────────────── Wrappers ─────────────────
|   └── wrappers/
|      ├── logger/
|      └── math/
|
└──────────────── Hardware ─────────────────
```