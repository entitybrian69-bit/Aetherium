#!/usr/bin/env python3
"""Era markers: the single mechanism that adapts the 1.21.1 reference source to the other 32 versions.

A Java (or JSON) file may contain blocks like

    // @era:options-begin instances
    <active code — the variant for the version this tree is currently set to>
    // @era:options-else fields
    //~ <inactive code, every line prefixed with "//~ ">
    // @era:options-end

``select(tree_text, version)`` swaps the active variant to the one ``ERAS`` picks for the version,
commenting the old one out. The operation is reversible and idempotent, and the reference tree
is simply the result for 1.21.1. A variant label may list several names separated by ``|``.

Every signature behind a variant was read from the javap dumps in tools/probe/ (see
tools/stubgen.py + tools/stubcheck.py, which compile each variant against those signatures).
"""
import re


def vkey(v):
    return [int(x) for x in v.split(".")]


def in_range(version, rng):
    if ".." not in rng:  # a bare version means exactly that version
        return vkey(version) == vkey(rng)
    lo, _, hi = rng.partition("..")
    k = vkey(version)
    if lo and k < vkey(lo):
        return False
    if hi and k > vkey(hi):
        return False
    return True


# name -> ordered list of (variant, version range); first match wins.
ERAS = {
    # --- options access (VanillaOptions, OptionInstanceMixin) -------------------------------
    "options": [("fields", "..1.18.2"), ("instances", "1.19..")],
    "simdist": [("none", "..1.17.1"), ("sim", "1.18..")],
    "graphics": [("status", "..1.21.10"), ("preset", "1.21.11..")],
    # --- GUI (McCanvas, AetheriumScreen, GuiMixin) ------------------------------------------
    "gui": [("stack", "..1.19.4"), ("graphics", "1.20..1.21.11"), ("extractor", "26.1..")],
    "gui-layer": [("none", "..1.21.5"), ("stratum", "1.21.6..")],
    "screen-pkg": [("flat", "..1.20.6"), ("options", "1.21..")],
    "screen-owner": [("minecraft", "..26.1"), ("gui", "26.2..")],
    "input": [("doubles", "..1.21.8"), ("events", "1.21.9..")],
    "scroll": [("three", "..1.20.1"), ("four", "1.20.2..")],
    "background": [("none", "..1.20.1"), ("render", "1.20.2..1.21.11"), ("extract", "26.1..")],
    "hud": [("stack", "..1.19.4"), ("graphics-float", "1.20..1.20.6"), ("graphics-delta", "1.21..1.21.11"),
            ("extractor", "26.1"), ("none", "26.2..")],
    # --- world / render hooks ---------------------------------------------------------------
    "light-hook": [("color", "..1.21.4"), ("brightness", "1.21.5..1.21.11"), ("coords", "26.1"), ("none", "26.2..")],
    "weather": [("level", "..1.21.1"), ("effect", "1.21.2..")],
    "vignette": [("render", "..1.21.11"), ("extract", "26.1"), ("none", "26.2..")],
    "dirty": [("blocks", "..26.1"), ("sections", "26.2..")],
    "reload": [("all-changed", "..26.1"), ("invalidate", "26.2..")],
    "entity-cull": [("five", "..26.2"), ("six", "26.3..")],
    "chunk-upload": [("crdbool", "..1.17.1"), ("crd", "1.18..1.20.1"), ("srd", "1.20.2..1.21.11"), ("none", "26.1..")],
    "section-vis": [("list16", "..1.16.5"), ("list17", "1.17..1.17.1"), ("list18", "1.18..1.20.1"),
                    ("sections", "1.20.2..1.21.4"),
                    ("render-origin", "1.21.5..")],
    "occlusion-box": [("entity", "..1.21.1"), ("renderer", "1.21.2..26.2"), ("renderer-tick", "26.3..")],
    "chunk-renderer": [("gl16", "..1.16.5"), ("none", "1.17..")],
    # 1.9.0, 1.16.5 only for testing: render scale (GameRenderer.renderLevel swaps Minecraft.mainRenderTarget)
    "render-scale": [("target", "..1.16.5"), ("r17", "1.17..1.21.1"), ("r212", "1.21.2..1.21.4"),
                     ("r215", "1.21.5..26.1"), ("g262", "26.2"), ("g263", "26.3..")],
    # ScaledRenderTarget constructor / resize / bindWrite inside the r17|r212|r215 class body
    "rs-api": [("none", "..1.16.5"), ("r17", "1.17..1.21.1"), ("r212", "1.21.2..1.21.4"),
               ("r215", "1.21.5..26.1"), ("g262", "26.2"), ("g263", "26.3..")],
    # 1.9.0, 1.16.5 only: the self-driving CI benchmark (Minecraft.createLevel(String, LevelSettings, ...))
    "autobench": [("create", "..1.16.5"), ("none", "1.17..")],
    # Zume-style zoom (ZoomKeys + Zoom*Mixin, KeyMappingAccessor, OptionsMixin key list): every version.
    "zoom": [("on", "..")],
    # KeyMapping(String, int, <category>): a String plus the CATEGORY_SORT_ORDER table until 1.21.8,
    # a registered KeyMapping$Category from 1.21.9 (probe dumps).
    # The category id is a ResourceLocation on 1.21.9-1.21.10 and an Identifier from 1.21.11.
    "keys": [("string", "..1.21.8"), ("category-rl", "1.21.9..1.21.10"), ("category-id", "1.21.11..")],
    # The zoomed FOV: GameRenderer.getFov returns double until 1.21.1 and float until 1.21.11;
    # 26.x moved it to Camera.calculateFov(F)F.
    "zoom-fov": [("double", "..1.21.1"), ("float", "1.21.2..1.21.11"), ("camera", "26.1..")],
    # Third-person distance: Camera.getMaxZoom(D)D until 1.20.6, (F)F after; called from Camera.setup
    # until 1.21.11 and from Camera.alignWithEntity on 26.x.
    "zoom-cam": [("double", "..1.20.6"), ("float", "1.21..1.21.11"), ("align", "26.1..")],
    # Threaded "build near" chunk rebuilds and the per-frame client light budget (NearBuildMixin).
    # 1.17 still rebuilds near sections on the render thread; 1.18+ vanilla builds them threaded by default.
    # Fast particles: SingleQuadParticle.render (VertexConsumer, Camera, float) with the camera
    # quaternion as com.mojang Quaternion (..1.19.2) or joml (1.19.3+); 1.20.3 adds facing modes;
    # 1.21 moves the vertices into renderRotatedQuad(VertexConsumer, Quaternionf, x, y, z, pt) with
    # the new vertex API and the corners in reverse order; 1.21.9 extracts render states instead.
    "particle-fast": [("v16", "..1.16.5"), ("quat", "1.17..1.19.2"), ("joml", "1.19.3..1.20.2"),
                      ("facing", "1.20.3..1.20.6"), ("inner", "1.21..1.21.8"), ("none", "1.21.9..")],
    # Per-frame view frustum: Frustum.calculateFrustum(view, projection), armed at the head of
    # LevelRenderer.prepareCullFrustum (void ..1.21.8, returns the Frustum 1.21.9..1.21.11). 1.16.5
    # captures in StreamingLevelMixin; 26.x builds the frustum outside LevelRenderer.
    "frustum-cap": [("none", "..1.16.5"), ("mojang", "1.17..1.19.2"), ("joml", "1.19.3..1.21.8"),
                    ("joml-ret", "1.21.9..1.21.11"), ("none", "26.1..")],
    # Fog cull of the visible-section list (1.16.5 culls inside its own search, StreamingLevelMixin):
    # 1.17 filters LevelRenderer.renderChunks after setupRender (search on the render thread, its
    # needsUpdate flag); 1.18-1.20.1 filter renderChunksInFrustum after applyFrustum
    # (needsFrustumUpdate); 1.20.2-26.1 filter visibleSections after applyFrustum
    # (SectionOcclusionGraph.needsFrustumUpdate); 26.2 has no applyFrustum.
    # 26.2+: the frustum pass (and the fog cull after it) moved to LevelExtractor.
    "fog-cull-x": [("none", "..26.1"), ("extract", "26.2..")],
    # 26.2+: section dirty flags live in SectionUpdateTracker$SectionDirtyState, read by LevelExtractor.
    "level-extract": [("none", "..26.1"), ("x262", "26.2..")],
    "fog-cull": [("none", "..1.16.5"), ("bfs", "1.17..1.17.1"), ("info", "1.18..1.20.1"),
                 ("section", "1.20.2..26.1"), ("none", "26.2..")],
    # Visible-section search throttle: the search runs on the render thread in setupRender up to
    # 1.17.1 (1.16.5 in StreamingLevelMixin; 1.17 needsUpdate PUTFIELD ordinals 0 camera moved,
    # 1 search start, 2 dirty section, as on 1.16.5); 1.18+ searches on a background thread.
    "search-throttle": [("v16", "..1.16.5"), ("bfs", "1.17..1.17.1"), ("none", "1.18..")],
    # Visible-only animated textures (1.7.0): the whole feature to 1.21.4 (1.21.5 changes the
    # tickers, quads and compiled sections).
    # Fast mob/block-entity models (1.7.0): BufferBuilder written directly while it is the old
    # ByteBuffer builder (..1.20.6); com.mojang.math to 1.19.2, joml from 1.19.3; 1.20.5 normalises
    # normals unless the pose's trustedNormals is set.
    # The Aetherium renderer on 1.17..1.21.4 (SectionCull): mesh upload through popNextBuffer
    # (..1.18.2), RenderedBuffer (1.19..1.20.6) or MeshData (1.21..1.21.4); 1.21.5 drops VertexBuffer.
    "section-cull": [("v16", "..1.16.5"), ("pop17", "1.17..1.18.2"), ("rendered19", "1.19..1.20.6"),
                     ("mesh21", "1.21..1.21.4"), ("none", "1.21.5..")],
    # renderChunkLayer / renderSectionLayer: (RenderType, PoseStack, ddd, Matrix4f) with com.mojang.math
    # (..1.19.2) or joml (1.19.3..1.20.4); 1.20.5 drops the PoseStack and takes two joml matrices.
    # The Aetherium renderer on 1.21.5+ (no VertexBuffer): meshes are sorted in SectionCompiler.compile
    # on the worker; draws are RenderPass$Draw records built in renderSectionLayer (1.21.5, layouts on
    # SectionBuffers) or prepareChunkRenders (1.21.6+, layouts on CompiledSectionMesh); 26.1 shares
    # vertex buffers (baseVertex), 26.2 moves IndexType, 26.3 builds IndexedDraw groups (renderpearl).
    "cull-new": [("old", "..1.21.4"), ("srd5", "1.21.5..1.21.5"), ("csm6", "1.21.6..1.21.11"),
                 ("uber26", "26.1..26.1"), ("uber262", "26.2..26.2"), ("pearl", "26.3..")],
    "layer-sig": [("v16", "..1.16.5"), ("mojang", "1.17..1.19.2"), ("joml", "1.19.3..1.20.4"),
                  ("nopose", "1.20.5..1.21.4"), ("none", "1.21.5..")],
    "model-fast": [("v16", "..1.16.5"), ("mojang", "1.17..1.19.2"), ("joml", "1.19.3..1.20.4"),
                   ("trusted", "1.20.5..1.20.6"), ("none", "1.21..")],
    "pose-normals": [("plain", "..1.20.4"), ("trusted", "1.20.5..")],
    "math": [("mojang", "..1.19.2"), ("joml", "1.19.3..")],
    # PoseStack.mulPose(Quaternionfc) became rotate(Quaternionfc) in 26.3.
    "pose-rot": [("mul", "..26.2"), ("rotate", "26.3..")],
    "anim": [("v16", "..1.16.5"), ("on", "1.17..1.21.4"), ("none", "1.21.5..")],
    # Atlas animation steps: 1.16.5 cycles sprites; 1.17 Tickable from getAnimationTicker (reload);
    # 1.19.3 TextureAtlasSprite$Ticker from createTicker (upload), tickAndUpload()V; 1.21.5 takes a
    # GpuTexture.
    "atlas-tick": [("v16", "..1.16.5"), ("tickable", "1.17..1.19.2"), ("ticker", "1.19.3..1.21.4"),
                   ("none", "1.21.5..")],
    "sprite-name": [("getname", "..1.19.2"), ("contents", "1.19.3..")],
    "sprite-u": [("d", "..1.20.1"), ("f", "1.20.2..")],
    "item-quads": [("instance", "..1.21.3"), ("static", "1.21.4..")],
    # Where a section's mesh is built: RebuildTask.compile with the CompiledChunk argument
    # (..1.18.2); compile returning CompileResults, the CompiledChunk/CompiledSection created right
    # after it in doTask on the same thread (1.19..1.20.6); SectionCompiler.compile (1.21..).
    "section-compile": [("v16", "..1.16.5"), ("param17", "1.17..1.18.2"), ("rebuild19", "1.19..1.20.1"),
                        ("rebuild202", "1.20.2..1.20.6"), ("compiler21", "1.21..1.21.4"), ("none", "1.21.5..")],
    "near-build": [("v16", "..1.17.1"), ("none", "1.18..")],
    # Entity yaw/pitch: public fields on 1.16.5, getYRot()/getXRot() from 1.17.
    "rot": [("fields", "..1.16.5"), ("getters", "1.17..")],
    # Freelook camera (FreelookCameraMixin): the view rotation is the first setRotation(FF)V call in
    # Camera.setup until 1.21.1; from 1.21.2 the first two (minecart lerp branch, normal branch), and
    # on 26.x they are in Camera.alignWithEntity (probe dumps).
    "freelook": [("v16", "..1.21.1"), ("minecart", "1.21.2..1.21.11"), ("align", "26.1..")],
    # 1.5.0 (1.16.5 first): streaming optimizations, fog/sky/particle controls.
    "qol16": [("v16", "..1.16.5"), ("none", "1.17..")],
    # 1.17.2 performance work (Minecraft 1.16.5 only): the Tuning* mixins, HudCache, ChestBaking.
    "boost16": [("v16", "..1.16.5"), ("none", "1.17..")],
    # --- Quality of life tab (1.16.x first; one era per hook file so each can be ported alone) --
    # Text components: new TextComponent(...) before 1.19, Component.literal(...) after.
    "chat": [("text", "..1.18.2"), ("literal", "1.19..")],
    # AbstractContainerScreen's slot lookup: findSlot before 1.21.2, getHoveredSlot after.
    "slot-find": [("find", "..1.21.1"), ("hovered", "1.21.2..")],
    # A chat line only the player sees: displayClientMessage before 26.1, sendSystemMessage after.
    "chat-send": [("display", "..1.21.11"), ("system", "26.1..")],
    # ResourceKey id: location() before 1.21.11, identifier() after.
    "res-key": [("location", "..1.21.10"), ("identifier", "1.21.11..")],
    # Debug screen shown: Options.renderDebug before 1.20.2, DebugScreenOverlay.showDebugScreen() after.
    "debug-screen": [("field", "..1.20.1"), ("overlay", "1.20.2..")],
    # Food values: Item.getFoodProperties() before 1.20.5, the FOOD data component after.
    "food-api": [("item", "..1.20.4"), ("component", "1.20.5..")],
    # Replaceable block test: Material.isReplaceable() before 1.19.3, BlockState.canBeReplaced() after.
    "q-place": [("material", "..1.19.2"), ("replace", "1.19.3..")],
    # Inventory helpers: carried stack on the inventory (1.16.5) or the menu; stacks compared by tag
    # (..1.20.4) or components (1.20.5..); clicks via ClickType or, from 26.1, ContainerInput.
    "q-inv": [("v16", "..1.16.5"), ("t17", "1.17..1.20.4"), ("c205", "1.20.5..1.21.11"), ("c261", "26.1..")],
    "q-food": [("on", "..")],
    # Crosshair sprite and hunger bar: GuiComponent.blit (instance ..1.19.3, static 1.19.4),
    # GuiGraphics.blit(icons) 1.20, blitSprite 1.20.2 (ResourceLocation.withDefaultNamespace 1.21),
    # + RenderType function 1.21.2, + RenderPipeline 1.21.6, Identifier 1.21.11, extractor 26.1,
    # the HUD in its own Hud class 26.2.
    "q-hud": [("v16", "..1.16.5"), ("s17", "1.17..1.19.3"), ("s194", "1.19.4"), ("g20", "1.20..1.20.1"),
              ("g202", "1.20.2..1.20.6"), ("g21", "1.21..1.21.1"), ("g212", "1.21.2..1.21.5"),
              ("g216", "1.21.6..1.21.10"), ("g2111", "1.21.11"), ("x261", "26.1"), ("x262", "26.2"),
              ("x263", "26.3..")],  # 26.3: RenderPipeline moved to com.mojang.renderpearl
    # Waypoints and damage numbers: drawn in the level renderer on 1.16.5, projected onto the HUD
    # (QolOverlay) from 1.17.
    "q-world": [("v16", "..1.16.5"), ("hud", "1.17..")],
    # ItemInHandRenderer.renderArmWithItem: TransformType -> ItemDisplayContext (1.19.4), its
    # left-hand flag dropped (1.21.5), buffers -> SubmitNodeCollector (1.21.9), renamed submit* (26.2).
    "q-hand": [("v16", "..1.16.5"), ("t17", "1.17..1.19.3"), ("d194", "1.19.4..1.21.4"),
               ("d215", "1.21.5..1.21.8"), ("s219", "1.21.9..26.1"), ("s262", "26.2"),
               ("none", "26.3..")],  # 26.3: FirstPersonHandsAndItemsRenderer (follow-up probe)
    # ScreenEffectRenderer.renderFire: (Minecraft, PoseStack) -> (PoseStack, MultiBufferSource) 1.21.4,
    # + TextureAtlasSprite 1.21.9, submitFire(PoseStack, SubmitNodeCollector, sprite) 26.2.
    "q-fire": [("v16", "..1.16.5"), ("f17", "1.17..1.21.3"), ("f214", "1.21.4..1.21.8"),
               ("f219", "1.21.9..26.1"), ("f262", "26.2..")],
    # 1.13.0: the 1.16.5 tuning features, one era per file (ported version by version).
    "t-ao-cache": [("v16", "..1.21.4"), ("a215", "1.21.5..1.21.11"), ("none", "26.1..")],
    "t-autosave": [("v16", "..1.17.1"), ("s18", "1.18..1.21.1"), ("s214", "1.21.2..")],
    "t-beam": [("v16", "..1.21.8"), ("b2111", "1.21.9..")],
    "t-block-entity": [("v16", "..1.16.5"), ("b17", "1.17..1.21.8"), ("b219", "1.21.9..")],
    "t-build-order": [("v16", "..1.16.5"), ("b17", "1.17..1.20.1"), ("b202", "1.20.2..1.21.1"), ("none", "1.21.2..")],
    "t-clouds": [("none", "..1.21.5"), ("k216", "1.21.6..1.21.11"), ("k261", "26.1..26.2"), ("k263", "26.3..")],
    "t-sort": [("none", "..1.21.1"), ("s212", "1.21.2..")],
    "t-weather": [("none", "..1.21.1"), ("w212", "1.21.2..1.21.8"), ("w219", "1.21.9..1.21.10"), ("w2111", "1.21.11..")],
    "t-compile-queue": [("none", "..1.21.1"), ("q212", "1.21.2..1.21.10"), ("q2111", "1.21.11..26.1"), ("none", "26.2..")],
    "t-chunk-save": [("v16", "..1.20.6"), ("c21", "1.21..1.21.3"), ("c214", "1.21.4..1.21.11"), ("c26", "26.1..")],
    "t-client": [("v16", "..1.21.8"), ("v19", "1.21.9..26.1"), ("v262", "26.2..")],
    "t-color-filter": [("v16", "..1.16.5"), ("r17", "1.17..26.1"), ("none", "26.2..")],
    "t-cull": [("v16", "..1.16.5"), ("c17", "1.17..1.20.1"), ("c202", "1.20.2..1.21.4"), ("c215", "1.21.5..")],
    "t-explosion": [("v16", "..1.21.1"), ("x212", "1.21.2..")],
    # Far terrain (1.15.0 on 1.17+): the GL 3 renderer (FarTerrainGl) plus the version bridges.
    "t-far-terrain": [("v16", "..1.16.5"), ("gl", "1.17..")],
    "t-far-color": [("material", "..1.19.4"), ("map", "1.20..")],
    "t-far-sections": [("none", "..1.16.5"), ("acc17", "1.17..1.18.1"), ("chunk", "1.18.2..1.20.1"), ("section", "1.20.2..1.21.10"),
                       ("visible", "1.21.11..26.2"), ("visible-t", "26.3..")],
    "t-far-acc": [("none", "..1.16.5"), ("v17", "1.17..1.18.1"), ("none", "1.18.2..")],
    "t-far-draw": [("v16", "..1.16.5"), ("m17", "1.17..1.19.2"), ("j193", "1.19.3..1.20.4"), ("s205", "1.20.5..1.21.5"),
                   ("g216", "1.21.6..1.21.11"), ("c26", "26.1..")],
    "t-far-group": [("none", "..1.21.5"), ("g216", "1.21.6..")],
    "t-far-proj": [("none", "..1.21.11"), ("b26", "26.1..26.2"), ("p263", "26.3..")],
    "t-far-gl": [("none", "..1.16.5"), ("id17", "1.17..1.21.4"), ("gl215", "1.21.5..26.1"), ("gl262", "26.2"),
                 ("pearl", "26.3..")],
    "t-flight-preload": [("v16", "..1.19.4"), ("f20", "1.20..1.21.4"), ("f215", "1.21.5"), ("f216", "1.21.6..")],
    "t-frame-item": [("v16", "..1.16.5"), ("f17", "1.17..1.21.1"), ("f212", "1.21.2..")],
    "t-gl-state": [("v16", "..1.16.5"), ("g17", "1.17..1.21.1"), ("g212", "1.21.2..1.21.4"), ("g215", "1.21.5..26.1"), ("none", "26.2..")],
    "t-glint": [("v16", "..1.20.6"), ("g21", "1.21..1.21.1"), ("g212", "1.21.2..1.21.8"), ("g219", "1.21.9..1.21.11"), ("none", "26.1..")],
    "t-grass": [("v16", "..1.18.2"), ("g19", "1.19..1.21.4"), ("g215", "1.21.5..1.21.11"), ("none", "26.1..")],
    "t-grass-quads": [("v16", "..1.21.11"), ("none", "26.1..")],
    "t-http-textureacc": [("v16", "..1.21.3"), ("none", "1.21.4..")],
    "t-layers": [("v16", "..1.21.1"), ("l212", "1.21.2..1.21.8"), ("l219", "1.21.9..")],
    "t-lazy-data-fixers": [("v16", "..1.18.2"), ("d19", "1.19..1.19.3"), ("none", "1.19.4..")],
    "t-leaf-cull": [("v16", "..1.16.5"), ("f17", "1.17..1.21.1"), ("f212", "1.21.2..1.21.4"), ("f215", "1.21.5..1.21.11"), ("none", "26.1..")],
    "t-leaves": [("v16", "..1.21.10"), ("l2111", "1.21.11"), ("none", "26.1..")],
    "t-level": [("v16", "..1.16.5"), ("l17", "1.17..1.19.4"), ("l20", "1.20..1.20.1"), ("l202", "1.20.2..1.21.1"), ("none", "1.21.2..")],
    "t-levelacc": [("v16", "..1.16.5"), ("none", "1.17..")],
    "t-light": [("v16", "..")],
    "t-lightmap": [("v16", "..1.21.10"), ("l2111", "1.21.11"), ("none", "26.1..")],
    "t-loading": [("v16", "..1.18.2"), ("d194", "1.19..1.21.8"), ("none", "1.21.9..")],
    "t-map": [("v16", "..1.21.1"), ("m212", "1.21.2..")],
    "t-minecart": [("v16", "..1.21.1"), ("m212", "1.21.2..1.21.8"), ("m219", "1.21.9..")],
    "t-name-tag": [("v16", "..1.16.5"), ("n17", "1.17..1.21.1"), ("n212", "1.21.2..")],
    "t-crowd": [("none", "..1.16.5"), ("c17", "1.17..")],
    # The overlay (damage tint) fast path is fixed-function GL; 1.17+ draws it in shaders.
    "t-damage-tint": [("v16", "..1.16.5"), ("none", "1.17..")],
    "t-ocean-floor": [("v16", "..1.16.5"), ("o17", "1.17..")],
    "t-portal": [("v16", "..1.20.6"), ("p21", "1.21..1.21.4"), ("p215", "1.21.5..")],
    "t-server-idle": [("v16", "..")],
    "t-server-light": [("v16", "..1.19.4"), ("none", "1.20..")],
    "t-shadow": [("v16", "..1.16.5"), ("s17", "1.17..1.21.1"), ("s212", "1.21.2..1.21.8"), ("s219", "1.21.9..")],
    "t-sign": [("v16", "..1.19.2"), ("s193", "1.19.3..1.19.4"), ("s20", "1.20..1.21.3"), ("s214", "1.21.4..1.21.8"), ("s219", "1.21.9..")],
    "t-skin": [("v16", "..1.21.3"), ("none", "1.21.4..")],
    "t-skull": [("v16", "..1.16.5"), ("none", "1.17..")],
    "t-skull-lookup": [("v16", "..1.16.5"), ("none", "1.17..")],
    "t-sky": [("v16", "..1.16.5"), ("e17", "1.17..1.20.6"), ("e21", "1.21..1.21.1"), ("none", "1.21.2..")],
    "t-view-areaacc": [("v16", "..1.16.5"), ("none", "1.17..")],
    "t-world": [("v16", "..1.18.1"), ("w182", "1.18.2..1.20.4"), ("w206", "1.20.5..1.20.6"), ("w21", "1.21..26.1"), ("none", "26.2..")],
    "t-raw-frame": [("none", "..1.16.5"), ("r17", "1.17..1.21.1"), ("r212", "1.21.2..1.21.4"), ("r215", "1.21.5..26.1"), ("none", "26.2..")],
    "t-world-cache": [("v16", "..1.16.5"), ("r17", "1.17..26.1"), ("none", "26.2..")],
    # 1.19.0 optimizations (every version)
    "t-sound": [("v16", "..1.21.5"), ("r216", "1.21.6..1.21.10"), ("r2111", "1.21.11..")],
    "t-drops": [("v16", "..1.20.4"), ("c205", "1.20.5..1.21.1"), ("c212", "1.21.2..1.21.3"), ("s214", "1.21.4..")],
    "t-unpack-make": [("v16", "..1.17.1"), ("r18", "1.18..1.19.2"), ("r193", "1.19.3..1.19.4"), ("r20", "1.20..1.21.1"),
                      ("l212", "1.21.2..1.21.8"), ("f219", "1.21.9..")],
    "t-unpack-sec": [("v16", "..1.17.1"), ("b18", "1.18..1.18.2"), ("b19", "1.19..1.21.11"), ("f261", "26.1..")],
    "t-unpack-net": [("v16", "..1.17.1"), ("d18", "1.18..26.2"), ("r263", "26.3..")],
    "t-tex-sprite": [("v16", "..1.16.5"), ("s17", "1.17..1.19.2"), ("c193", "1.19.3..1.21.10"), ("c2111", "1.21.11"),
                     ("f261", "26.1..")],
    "t-tex-atlas": [("v16", "..1.19.2"), ("c193", "1.19.3..")],
    # Scenery (fog / sky / clouds / particles), ported from 1.16.5 in 1.8.0.
    "fog": [("gl", "..1.16.5"), ("shader17", "1.17..1.18.2"), ("shader19", "1.19..1.21.1"),
            ("params", "1.21.2..1.21.5"), ("buffer", "1.21.6..1.21.11"), ("data", "26.1..")],
    "sky": [("v16", "..1.16.5"), ("end17", "1.17..1.18.2"), ("rb19", "1.19..1.19.2"), ("rb193", "1.19.3..1.20.6"),
            ("mesh", "1.21..1.21.1"), ("renderer", "1.21.2..")],
    "clouds": [("level", "..1.21.1"), ("renderer", "1.21.2..")],
    "block-particles": [("engine", "..1.21.8"), ("level", "1.21.9..")],
    "registry": [("static", "..1.19.2"), ("builtin", "1.19.3..")],
    "camera-entity": [("getter", "..1.21.10"), ("record", "1.21.11..")],
    "camera-get": [("getter", "..26.1"), ("record", "26.2..")],
    "mesh-exec": [("crd16", "..1.16.5"), ("crd", "1.17..1.20.1"), ("srd", "1.20.2..1.21.1"), ("tracing", "1.21.2..")],
    # vanilla allocates a BitSet per block and a MutableBlockPos per AO face up to 1.21.4 only.
    "mesh-alloc": [("v16", "..1.21.4"), ("none", "1.21.5..")],
    # Render sections: class, origin getter and "compiled" check (rebuild coalescing).
    "section-api": [("v16", "..1.16.5"), ("crd", "1.17..1.20.1"), ("srd", "1.20.2..1.21.4"), ("srd-ro", "1.21.5..1.21.5"),
                    ("mesh", "1.21.6..26.1"), ("none", "26.2..")],
    "cam-pos": [("getter", "..1.21.5"), ("position", "1.21.6..")],
    # 1.7.2 even frame pacing: static limitDisplayFPS(I)V is in RenderSystem on 1.16.5-1.21.11 and in
    # net.minecraft.client.FramerateLimiter on 26.x (probe dumps).
    "pacing": [("rs", "..1.21.11"), ("limiter", "26.1..")],
    "be-render": [("render", "..1.21.8"), ("extract", "1.21.9..26.1"), ("extract-flag", "26.2..")],
    "bg-threads": [("clamp", "..1.17.1"), ("property", "1.18..")],
    "sound-click": [("event", "..1.19.2"), ("holder", "1.19.3..")],
    # --- platform -------------------------------------------------------------------------
    "logger": [("log4j", "..1.17.1"), ("slf4j", "1.18..")],
}

BEGIN = re.compile(r"^(\s*)// @era:([\w-]+)-begin (\S+)\s*$")
ELSE = re.compile(r"^(\s*)// @era:([\w-]+)-else (\S+)\s*$")
END = re.compile(r"^(\s*)// @era:([\w-]+)-end\s*$")
INACTIVE = re.compile(r"^(\s*)//~ ?(.*)$")


def variant_for(name, version):
    if name not in ERAS:
        raise KeyError("unknown era %r" % name)
    for variant, rng in ERAS[name]:
        if in_range(version, rng):
            return variant
    raise KeyError("era %r has no variant for %s" % (name, version))


def _activate(lines):
    out = []
    for line in lines:
        m = INACTIVE.match(line)
        if m:
            out.append(m.group(1) + m.group(2))
        else:
            out.append(line)
    return out


def _deactivate(lines):
    out = []
    for line in lines:
        if not line.strip():
            out.append("")  # never leave trailing whitespace in a ported tree
            continue
        indent = len(line) - len(line.lstrip())
        out.append(line[:indent] + "//~ " + line[indent:])
    return out


def select(text, version, path="<text>", used=None):
    """Return text with every era block switched to the variant for ``version``."""
    lines = text.split("\n")
    out = []
    i = 0
    while i < len(lines):
        m = BEGIN.match(lines[i])
        if not m:
            if ELSE.match(lines[i]) or END.match(lines[i]):
                raise SystemExit("%s:%d: era marker outside a block" % (path, i + 1))
            out.append(lines[i])
            i += 1
            continue
        indent, name = m.group(1), m.group(2)
        blocks = [(m.group(3), [])]
        i += 1
        while True:
            if i >= len(lines):
                raise SystemExit("%s: unterminated era block %s" % (path, name))
            e = ELSE.match(lines[i])
            n = END.match(lines[i])
            if e and e.group(2) == name:
                blocks.append((e.group(3), []))
            elif n and n.group(2) == name:
                break
            elif BEGIN.match(lines[i]) and BEGIN.match(lines[i]).group(2) == name:
                raise SystemExit("%s:%d: nested era block %s" % (path, i + 1, name))
            else:
                blocks[-1][1].append(lines[i])
            i += 1
        i += 1  # skip end
        want = variant_for(name, version)
        if used is not None:
            used.add(name)
        chosen = None
        for idx, (label, _) in enumerate(blocks):
            if want in label.split("|"):
                chosen = idx
                break
        if chosen is None:
            raise SystemExit("%s: era block %s has no variant %r (for %s); variants: %s"
                             % (path, name, want, version, [b[0] for b in blocks]))
        # normalise: active block = chosen, all others inactive
        normalised = []
        for idx, (label, body) in enumerate(blocks):
            plain = _activate(body) if idx != 0 else body
            normalised.append((label, plain))
        ordered = [normalised[chosen]] + [b for k, b in enumerate(normalised) if k != chosen]
        active_label, active_body = ordered[0]
        ordered[0] = (active_label, select("\n".join(active_body), version, path, used).split("\n") if active_body else active_body)
        out.append("%s// @era:%s-begin %s" % (indent, name, ordered[0][0]))
        out.extend(ordered[0][1])
        for label, body in ordered[1:]:
            out.append("%s// @era:%s-else %s" % (indent, name, label))
            out.extend(_deactivate(body))
        out.append("%s// @era:%s-end" % (indent, name))
    return "\n".join(out)


def has_markers(text):
    return "// @era:" in text


def select_all(text, version, path="<text>", used=None):
    """Apply ``select`` until stable, so blocks nested inside a variant are resolved too."""
    for _ in range(8):
        new = select(text, version, path, used)
        if new == text:
            return new
        text = new
    return text
