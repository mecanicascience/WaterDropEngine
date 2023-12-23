# WaterDropEngine

### Structure

```
core/
├────────────────── Game ───────────────────
|   └── game/
|
├────────────────── Engine ───────────────────
|   └── engine/
|       ├── scene/
|       └── renderer/
|           ├── scene_graph/
|           └── low_level/
|
├──────────────── Resources ────────────────
|   └── resources/
|
├─────────────── Core Systems ──────────────
|   └── core/
|
├─────── Plateform Independent Layers ──────
|   └── wrappers/
|      ├── editor/
|      ├── logger/
|      └── graphics_wgpu/
|
├───────────── 3rd Party SDKs ──────────────
|   └── wgpu
|
└──────────────── Hardware ─────────────────
```