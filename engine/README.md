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
|   └── third_party/
|      ├── editor/
|      ├── wgpu/
|      └── window/
|   |
|   └── wrappers/
|      ├── logger/
|      └── math/
|
└──────────────── Hardware ─────────────────
```