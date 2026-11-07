# Aetherium auto benchmark (1.16.5)

Window 854x480, render distance 8, seed 7294105550031, GPU: Unknown, Java 17.0.21, 4 CPUs.

| Scenario | Avg FPS | 1% low FPS | p50 ms | p99 ms | p99.9 ms | Max ms | Frames > 50 ms | Frames > 100 ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Vanilla (Aetherium off) | 35.4 | 19.9 | 27.79 | 41.87 | 59.68 | 59.7 | 3 | 0 |
| Aetherium (defaults) | 38.5 | 21.3 | 24.99 | 40.16 | 64.53 | 64.5 | 2 | 0 |
| Aetherium, all 1.12.0 options off | 39.2 | 23.1 | 24.83 | 36.54 | 64.84 | 64.8 | 1 | 0 |
| Aetherium, directional culling + view-first building off | 38.8 | 24.5 | 24.58 | 39.51 | 42.72 | 42.7 | 0 | 0 |
| Aetherium, GL call trimming + cheap damage tint off | 37.2 | 22.3 | 25.82 | 41.90 | 57.17 | 57.2 | 1 | 0 |
| Aetherium, all 1.17.2 options off | 37.4 | 24.9 | 26.14 | 38.31 | 42.43 | 42.4 | 0 | 0 |
| Aetherium + GPU occlusion culling | 39.3 | 23.0 | 24.88 | 35.46 | 59.72 | 59.7 | 1 | 0 |
| Aetherium (defaults), measured again | 35.3 | 22.3 | 27.31 | 43.20 | 47.97 | 48.0 | 0 | 0 |
| Aetherium + render scale 70% | 39.1 | 24.8 | 24.63 | 39.38 | 42.18 | 42.2 | 0 | 0 |
| Flying into new terrain (+X), Aetherium off | 56.4 | 22.8 | 15.99 | 38.00 | 47.26 | 66.1 | 1 | 0 |
| Flying into new terrain (-X), Aetherium defaults | 46.1 | 22.7 | 20.48 | 38.98 | 51.15 | 51.2 | 1 | 0 |
| Flying into new terrain (+Z), all 1.17.2 options off | 55.6 | 29.2 | 16.76 | 31.27 | 35.86 | 39.7 | 0 | 0 |
| Flying into new terrain (-Z), Aetherium defaults | 52.5 | 23.2 | 17.78 | 36.17 | 50.29 | 56.4 | 2 | 0 |

## Notes

- vanilla: 3 GC collections, 14 ms in GC; renderer: Vanilla; regions: ; render scale: 100%; chests baked so far: 12
- aetherium: 3 GC collections, 10 ms in GC; renderer: Aetherium pipeline, compatible mode (111 sections in the last layer, 28% of faces skipped; last frame 325 section layers in 206 draws, vanilla: 325; batched: 170 buffers (170 packed), 26 MB (21 MB used), last layer 34 buffers / 73 draws); regions: ; batched: 170 buffers (170 packed), 26 MB (21 MB used), last layer 34 buffers / 73 draws; render scale: 100%; chests baked so far: 16
- new-off: 3 GC collections, 9 ms in GC; renderer: Aetherium pipeline, compatible mode (111 sections in the last layer, 28% of faces skipped; last frame 325 section layers in 208 draws, vanilla: 325; batched: 169 buffers (169 packed), 26 MB (21 MB used), last layer 34 buffers / 74 draws); regions: ; batched: 169 buffers (169 packed), 26 MB (21 MB used), last layer 34 buffers / 74 draws; render scale: 100%; chests baked so far: 20
- no-cull: 3 GC collections, 17 ms in GC; renderer: Aetherium pipeline, compatible mode (110 sections in the last layer, 28% of faces skipped; last frame 325 section layers in 206 draws, vanilla: 325; batched: 168 buffers (168 packed), 26 MB (21 MB used), last layer 34 buffers / 74 draws); regions: ; batched: 168 buffers (168 packed), 26 MB (21 MB used), last layer 34 buffers / 74 draws; render scale: 100%; chests baked so far: 24
- no-gl: 3 GC collections, 9 ms in GC; renderer: Aetherium pipeline, compatible mode (111 sections in the last layer, 28% of faces skipped; last frame 330 section layers in 206 draws, vanilla: 330; batched: 169 buffers (169 packed), 26 MB (21 MB used), last layer 34 buffers / 73 draws); regions: ; batched: 169 buffers (169 packed), 26 MB (21 MB used), last layer 34 buffers / 73 draws; render scale: 100%; chests baked so far: 28
- boost-off: 3 GC collections, 14 ms in GC; renderer: Aetherium pipeline, compatible mode (108 sections in the last layer, 28% of faces skipped; last frame 324 section layers in 205 draws, vanilla: 324; batched: 168 buffers, 41 MB (33 MB used), last layer 34 buffers / 72 draws); regions: ; batched: 168 buffers, 41 MB (33 MB used), last layer 34 buffers / 72 draws; render scale: 100%; chests baked so far: 28
- occlusion: 3 GC collections, 12 ms in GC; renderer: Aetherium pipeline, compatible mode (94 sections in the last layer, 25% of faces skipped; last frame 259 section layers in 163 draws, vanilla: 259; batched: 170 buffers (170 packed), 26 MB (21 MB used), last layer 32 buffers / 65 draws); regions: ; batched: 170 buffers (170 packed), 26 MB (21 MB used), last layer 32 buffers / 65 draws; render scale: 100%; chests baked so far: 32
- aetherium-again: 3 GC collections, 8 ms in GC; renderer: Aetherium pipeline, compatible mode (109 sections in the last layer, 28% of faces skipped; last frame 325 section layers in 206 draws, vanilla: 325; batched: 169 buffers (169 packed), 26 MB (21 MB used), last layer 34 buffers / 72 draws); regions: ; batched: 169 buffers (169 packed), 26 MB (21 MB used), last layer 34 buffers / 72 draws; render scale: 100%; chests baked so far: 36
- scale70: 3 GC collections, 10 ms in GC; renderer: Aetherium pipeline, compatible mode (111 sections in the last layer, 28% of faces skipped; last frame 325 section layers in 208 draws, vanilla: 325; batched: 170 buffers (170 packed), 26 MB (21 MB used), last layer 34 buffers / 73 draws); regions: ; batched: 170 buffers (170 packed), 26 MB (21 MB used), last layer 34 buffers / 73 draws; render scale: 70%; chests baked so far: 40
- fly-vanilla: 12 GC collections, 75 ms in GC; renderer: Vanilla; regions: ; render scale: 100%; chests baked so far: 40
- fly-aetherium: 16 GC collections, 69 ms in GC; renderer: Aetherium pipeline, compatible mode (78 sections in the last layer, 25% of faces skipped; last frame 229 section layers in 119 draws, vanilla: 229; batched: 160 buffers (160 packed), 26 MB (17 MB used), last layer 13 buffers / 18 draws); regions: ; batched: 160 buffers (160 packed), 26 MB (17 MB used), last layer 13 buffers / 18 draws; render scale: 100%; chests baked so far: 47
- fly-boost-off: 15 GC collections, 75 ms in GC; renderer: Aetherium pipeline, compatible mode (39 sections in the last layer, 27% of faces skipped; last frame 64 section layers in 59 draws, vanilla: 64; batched: 124 buffers, 54 MB (42 MB used), last layer 14 buffers / 39 draws); regions: ; batched: 124 buffers, 54 MB (42 MB used), last layer 14 buffers / 39 draws; render scale: 100%; chests baked so far: 47
- fly-aetherium-z: 26 GC collections, 114 ms in GC; renderer: Aetherium pipeline, compatible mode (50 sections in the last layer, 39% of faces skipped; last frame 204 section layers in 201 draws, vanilla: 204; batched: 129 buffers (129 packed), 30 MB (23 MB used), last layer 7 buffers / 9 draws); regions: ; batched: 129 buffers (129 packed), 30 MB (23 MB used), last layer 7 buffers / 9 draws; render scale: 100%; chests baked so far: 60
