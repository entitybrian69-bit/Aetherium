package com.aetherium.client;

import com.aetherium.Aetherium;
import com.aetherium.Capabilities;
import com.aetherium.android.AndroidEnvironment;
import com.aetherium.config.AetheriumConfig;
import com.aetherium.config.ConfigValue;
import com.aetherium.gui.AetheriumView;
import com.aetherium.gui.Page;
import com.aetherium.gui.PixelArt;
import com.aetherium.perf.TuningSupport;
import com.aetherium.perf.BlockEntityCull;
import com.aetherium.perf.RenderToggles;
import com.aetherium.perf.Tuning;
import com.aetherium.perf.WorkerThreads;
import com.aetherium.qol.ParticleFilter;
import com.aetherium.qol.Qol;
import com.aetherium.qol.QolSupport;
import com.aetherium.gui.Setting;
import com.aetherium.mixin.AetheriumMixinPlugin;

import java.util.ArrayList;
import java.util.List;
import java.util.function.IntConsumer;
import java.util.function.IntFunction;
import java.util.function.IntSupplier;

import net.minecraft.client.Minecraft;

/**
 * The tabs of the Aetherium video settings screen and every row on them.
 *
 * <p>Rows read live values when the screen opens and write them only on Apply
 * (see {@link Setting}). Vanilla options go through {@link VanillaOptions};
 * Aetherium options go straight to the config, and {@link ClientHooks} picks
 * them up after Apply, so every change takes effect immediately without a
 * restart.</p>
 */
public final class AetheriumPages {

    /** What buttons on the pages need from the screen. */
    public interface Actions {
        void resetDefaults();

        void openVanillaVideo();

        /** Opens the waypoint list (Quality of life). */
        void openWaypoints();
    }

    static final String NOT_ON_THIS_VERSION = "Not available on this version";
    static final String STEPPED_ASIDE = "Turned off: another installed mod already changes this part of the game, so Aetherium leaves it to that mod.";

    private static final String[] GRAPHICS = {"Fast", "Fancy", "Fabulous"};
    private static final String[] CLOUDS = {"Off", "Fast", "Fancy"};
    private static final String[] PARTICLES = {"All", "Decreased", "Minimal"};
    private static final String[] PRESETS = {"Max FPS", "Balanced", "Quality"};
    private static final String[] LIGHTS = {"Off", "Fast", "Fancy"};
    private static final String[] LEAVES = {"Graphics", "Fast", "Fancy"};
    private static final String[] SHADERS = {"Off", "Vibrant", "Warm", "Cinematic"};
    private static final String[] CORNERS = {"Top left", "Top right", "Bottom left", "Bottom right"};
    private static final String[] CORNER_KEYS = {"top-left", "top-right", "bottom-left", "bottom-right"};

    // Preset table. Columns follow PRESET_ROWS; rows are MAX_FPS, BALANCED, QUALITY.
    private static final String[] PRESET_ROWS = {
        "perf.render_distance", "perf.simulation_distance", "quality.graphics", "quality.clouds",
        "quality.particles", "quality.smooth_lighting", "quality.biome_blend", "quality.shadows",
        "perf.cull_distance", "perf.particle_density", "quality.weather", "quality.vignette",
        "effects.dynamic_lights", "perf.entity_distance", "perf.adaptive",
        "perf.animated_textures", "perf.block_entity_distance", "perf.smooth_chunks",
    };
    // Adaptive distance is off in every preset: each change rebuilds all chunks, so it is an
    // explicit opt-in. Dynamic lights cost chunk rebuilds too and are only part of Quality.
    private static final int[][] PRESET_VALUES = {
        {6, 5, 0, 0, 2, 0, 0, 0, 48, 30, 0, 0, 0, 75, 0, 0, 32, 1},
        {10, 8, 1, 1, 1, 1, 2, 1, 64, 70, 1, 1, 0, 100, 0, 1, 48, 1},
        {16, 12, 1, 2, 0, 1, 5, 1, 128, 100, 1, 1, 1, 150, 0, 1, 64, 1},
    };

    private final AetheriumConfig config;
    private final AndroidEnvironment android;
    private final Actions actions;
    private AetheriumView view;

    public AetheriumPages(final Aetherium.Subsystems sub, final Actions actions) {
        this.config = sub.config;
        this.android = sub.android;
        this.actions = actions;
    }

    /** The view is created from {@link #build()}, so it is attached afterwards. */
    public void attach(final AetheriumView view) {
        this.view = view;
        cacheHandRows();
    }

    public List<Page> build() {
        final List<Page> pages = new ArrayList<Page>();
        pages.add(general());
        pages.add(quality());
        pages.add(performance());
        pages.add(backend());
        pages.add(effects());
        if (QolSupport.ANY) {
            pages.add(qol());
        }
        pages.add(androidPage());
        return pages;
    }

    // ------------------------------------------------------------------ General

    private Page general() {
        final Page page = new Page("General", "General", "Master switch, presets and overlay", PixelArt.GEAR);
        page.add(bool("general.enabled", "Enable Aetherium", config.enabled));
        page.add(Setting.choice("general.preset", Setting.Kind.SEGMENTED, "Performance preset",
                        "Stages a full set of options. Editing any of them afterwards switches to Custom.", PRESETS,
                        () -> config.preset.get().ordinal() - 1,
                        v -> config.preset.set(AetheriumConfig.Preset.values()[Math.max(-1, v) + 1]))
                .withDefault(-1)
                .onPick(this::stagePreset));
        page.add(bool("general.hud", "Frame-time overlay", config.showFrameHud)
                .availableWhen(() -> Capabilities.HUD_OVERLAY, NOT_ON_THIS_VERSION));
        page.add(Setting.choice("general.hud_corner", Setting.Kind.DROPDOWN, "Overlay position",
                        "Screen corner the overlay is anchored to.", CORNERS,
                        () -> cornerIndex(config.hudCorner.get()),
                        v -> config.hudCorner.set(CORNER_KEYS[Math.max(0, Math.min(3, v))]))
                .withDefault(cornerIndex(config.hudCorner.getDefault()))
                .availableWhen(() -> Capabilities.HUD_OVERLAY, NOT_ON_THIS_VERSION));
        page.add(bool("general.notify", "Conflict notices", config.notifyConflicts));
        page.add(bool("general.ui_sounds", "Interface sounds", config.uiSounds));
        page.add(Setting.info("general.status", "Status", "What Aetherium is doing right now.", this::statusText));
        page.add(Setting.button("general.reset", "Reset Aetherium options",
                "Stages every Aetherium option back to its default. Vanilla options are left alone.", "Reset",
                actions::resetDefaults));
        page.add(Setting.button("general.vanilla", "Vanilla video settings",
                "Opens Minecraft's own video settings screen.", "Open", actions::openVanillaVideo));
        return page;
    }

    private void stagePreset(final int index) {
        if (this.view == null || index < 0 || index >= PRESET_VALUES.length) {
            return;
        }
        final int[] values = PRESET_VALUES[index];
        for (int i = 0; i < PRESET_ROWS.length; i++) {
            final Setting row = this.view.find(PRESET_ROWS[i]);
            if (row != null && row.isEditable()) {
                row.stage(values[i]);
            }
        }
    }

    private String statusText() {
        switch (Aetherium.getState()) {
            case ACTIVE:
                return "Active";
            case VANILLA:
                return "Paused (master switch off)";
            case INCOMPATIBLE:
                return "Standing down: " + Aetherium.conflicts().summarize();
            default:
                return "Not running";
        }
    }

    private static int cornerIndex(final String key) {
        for (int i = 0; i < CORNER_KEYS.length; i++) {
            if (CORNER_KEYS[i].equals(key)) {
                return i;
            }
        }
        return 0;
    }

    // ------------------------------------------------------------------ Quality

    private Page quality() {
        final Page page = new Page("Quality", "Quality", "How the world looks", PixelArt.SPARKLES);
        page.add(Setting.choice("quality.graphics", Setting.Kind.SEGMENTED, "Graphics",
                "Fast skips transparent leaves and some effects. Fabulous uses extra passes for translucency.",
                GRAPHICS, VanillaOptions::getGraphics, VanillaOptions::setGraphics).presetMember());
        page.add(Setting.choice("quality.clouds", Setting.Kind.SEGMENTED, "Clouds", "Cloud rendering.",
                CLOUDS, VanillaOptions::getClouds, VanillaOptions::setClouds).presetMember());
        page.add(Setting.choice("quality.particles", Setting.Kind.SEGMENTED, "Particles",
                "Vanilla particle level. Combine with Particle density on the Performance tab.",
                PARTICLES, VanillaOptions::getParticles, VanillaOptions::setParticles).presetMember());
        page.add(flag("quality.smooth_lighting", "Smooth lighting", "Soft ambient-occlusion shading on blocks.",
                VanillaOptions::getSmoothLighting, VanillaOptions::setSmoothLighting).presetMember());
        page.add(Setting.slider("quality.biome_blend", "Biome blend",
                "Smooths grass, foliage and water colour between biomes. Higher costs chunk build time.",
                0, 7, 1, v -> v == 0 ? "Off" : (2 * v + 1) + "x" + (2 * v + 1),
                VanillaOptions::getBiomeBlend, VanillaOptions::setBiomeBlend).presetMember());
        page.add(flag("quality.shadows", "Entity shadows", "Round shadows under mobs and items.",
                VanillaOptions::getEntityShadows, VanillaOptions::setEntityShadows).presetMember());
        page.add(bool("quality.weather", "Weather", config.weather).presetMember());
        page.add(tuningRow(bool("quality.light_weather", "Lighter rain and snow", config.lightWeather)));
        page.add(tuningRow(intSlider("quality.far_terrain", "Far terrain", config.farTerrain, 4,
                v -> v == 0 ? "Off" : v + " chunks")));
        page.add(tuningRow(Setting.choice("quality.shader_preset", Setting.Kind.DROPDOWN, "Colour filter",
                        config.shaderPreset.getComment(), SHADERS,
                        () -> config.shaderPreset.get().ordinal(),
                        v -> config.shaderPreset.set(AetheriumConfig.ShaderPreset.values()[Math.max(0, Math.min(3, v))]))
                .withDefault(config.shaderPreset.getDefault().ordinal())));
        page.add(tuningRow(intSlider("quality.cloud_distance", "Cloud distance", config.cloudDistance, 25,
                v -> v >= 100 ? "Vanilla" : v + "%")));
        page.add(tuningRow(bool("quality.fast_grass", "Fast grass", config.fastGrass)));
        page.add(tuningRow(Setting.choice("quality.leaves", Setting.Kind.SEGMENTED, "Leaves",
                        config.leaves.getComment(), LEAVES,
                        () -> config.leaves.get().ordinal(),
                        v -> config.leaves.set(AetheriumConfig.LeavesMode.values()[Math.max(0, Math.min(2, v))]))
                .withDefault(config.leaves.getDefault().ordinal())));
        page.add(tuningRow(bool("quality.leaf_culling", "Leaf culling", config.leafCulling)));
        page.add(tuningRow(bool("quality.enchant_glint", "Enchantment glint", config.enchantGlint)));
        page.add(tuningRow(bool("quality.end_crystal_beams", "End crystal beams", config.endCrystalBeams)));
        page.add(bool("quality.vignette", "Vignette", config.vignette).presetMember()
                .availableWhen(() -> Capabilities.VIGNETTE_TOGGLE, NOT_ON_THIS_VERSION));
        addScenery(page);
        page.add(flag("quality.bob_view", "View bobbing", "Camera sway while walking.",
                VanillaOptions::getBobView, VanillaOptions::setBobView));
        page.add(Setting.slider("quality.brightness", "Brightness", "Vanilla brightness (gamma).",
                0, 100, 1, v -> v == 0 ? "Moody" : v == 100 ? "Bright" : v + "%",
                VanillaOptions::getBrightnessPercent, VanillaOptions::setBrightnessPercent));
        return page;
    }

    /** Fog controls, sky toggles and per-type particle toggles (1.16.5 first). */
    private void addScenery(final Page page) {
        page.add(sceneryRow(bool("quality.distance_fog", "Distance fog", config.distanceFog)));
        page.add(sceneryRow(intSlider("quality.fog_start", "Fog start", config.fogStart, 5,
                v -> v == 75 ? "75% (vanilla)" : v + "%")));
        page.add(sceneryRow(intSlider("quality.water_fog", "Water fog", config.waterFog, 5,
                v -> v == 0 ? "Clear" : v == 100 ? "Vanilla" : v + "%")));
        page.add(sceneryRow(bool("quality.clear_lava", "Clear lava", config.clearLava)));
        page.add(sceneryRow(bool("quality.sky", "Sky", config.sky)));
        page.add(sceneryRow(bool("quality.sun_moon", "Sun and moon", config.sunMoon)));
        page.add(sceneryRow(bool("quality.stars", "Stars", config.stars)));
        page.add(sceneryRow(intSlider("quality.cloud_height", "Cloud height", config.cloudHeight, 8,
                v -> v == 0 ? "Default" : v + " blocks")));
        page.add(sceneryRow(bool("quality.particles_explosions", "Explosion particles", config.particlesExplosions)));
        page.add(sceneryRow(bool("quality.particles_weather", "Rain splashes", config.particlesWeather)));
        page.add(sceneryRow(bool("quality.particles_smoke", "Smoke particles", config.particlesSmoke)));
        page.add(sceneryRow(bool("quality.particles_potions", "Potion particles", config.particlesPotions)));
        page.add(sceneryRow(bool("quality.particles_combat", "Hit particles", config.particlesCombat)));
        page.add(sceneryRow(bool("quality.particles_fire", "Flame particles", config.particlesFire)));
        page.add(sceneryRow(bool("quality.particles_drips", "Drips and bubbles", config.particlesDrips)));
        page.add(sceneryRow(bool("quality.particles_portals", "Portal particles", config.particlesPortals)));
        page.add(sceneryRow(bool("quality.particles_ambient", "Ambient particles", config.particlesAmbient)));
        page.add(sceneryRow(bool("quality.particles_totem", "Totem particles", config.particlesTotem)));
        page.add(sceneryRow(bool("quality.particles_redstone", "Redstone particles", config.particlesRedstone)));
        page.add(sceneryRow(bool("quality.particles_block_break", "Block breaking particles", config.particlesBlockBreak)));
    }

    private static Setting sceneryRow(final Setting setting) {
        return setting.availableWhen(() -> Capabilities.SCENERY_CONTROLS, NOT_ON_THIS_VERSION);
    }

    // -------------------------------------------------------------- Performance

    private Page performance() {
        final Page page = new Page("Performance", "Performance", "Frame rate and draw distance", PixelArt.BARS);
        page.add(Setting.slider("perf.render_distance", "Render distance",
                        "Chunks drawn around you. The single biggest FPS cost.", 2, 32, 1, v -> v + " chunks",
                        VanillaOptions::getRenderDistance,
                        v -> {
                            VanillaOptions.setRenderDistance(v);
                            ClientHooks.noteUserRenderDistance(v);
                        })
                .presetMember());
        page.add(Setting.slider("perf.simulation_distance", "Simulation distance",
                        "Chunks that tick (mobs, crops, redstone) in singleplayer.", 5, 32, 1, v -> v + " chunks",
                        VanillaOptions::getSimulationDistance, VanillaOptions::setSimulationDistance)
                .presetMember()
                .availableWhen(() -> Capabilities.SIMULATION_DISTANCE, "Added in Minecraft 1.18"));
        page.add(Setting.slider("perf.max_fps", "Max framerate", "Vanilla frame-rate limit.", 10,
                VanillaOptions.UNLIMITED_FPS, 10, v -> v >= VanillaOptions.UNLIMITED_FPS ? "Unlimited" : v + " FPS",
                VanillaOptions::getFramerateLimit, VanillaOptions::setFramerateLimit));
        page.add(bool("perf.frame_pacing", "Even frame pacing", config.framePacing)
                .availableWhen(() -> Capabilities.FRAME_PACING, NOT_ON_THIS_VERSION));
        page.add(flag("perf.vsync", "VSync", "Sync to the display refresh rate. Removes tearing, adds latency.",
                VanillaOptions::getVsync, VanillaOptions::setVsync));
        page.add(bool("perf.entity_culling", "Entity culling", config.entityCulling));
        page.add(bool("perf.occlusion_culling", "Hide entities behind walls", config.occlusionCulling)
                .availableWhen(() -> Capabilities.ENTITY_OCCLUSION && !RenderToggles.isOcclusionBroken(),
                        "Turned off after an error (see log)"));
        page.add(intSlider("perf.cull_distance", "Entity cull distance", config.entityCullDistance, 8,
                v -> v + " blocks").presetMember());
        page.add(Setting.slider("perf.entity_distance", "Entity distance",
                "Vanilla entity render distance scaling.", 50, 500, 25, v -> v + "%",
                VanillaOptions::getEntityDistancePercent, VanillaOptions::setEntityDistancePercent).presetMember());
        page.add(intSlider("perf.particle_density", "Particle density", config.particleDensity, 5,
                v -> v + "%").presetMember());
        page.add(bool("perf.adaptive", "Adaptive render distance", config.adaptiveDistance).presetMember());
        page.add(intSlider("perf.adaptive_target", "Adaptive target", config.adaptiveTargetFps, 5, v -> v + " FPS"));
        page.add(bool("perf.smooth_chunks", "Smooth chunk loading", config.smoothChunkLoading)
                .presetMember()
                .availableWhen(() -> Capabilities.SMOOTH_CHUNK_UPLOADS, NOT_ON_THIS_VERSION));
        page.add(bool("perf.threaded_builder", "Threaded chunk builder", config.threadedChunkBuilder)
                .availableWhen(() -> Capabilities.THREADED_NEAR_BUILDS, "Vanilla already does this on this version"));
        page.add(bool("perf.animated_textures", "Animated textures", config.animatedTextures)
                .presetMember()
                .availableWhen(() -> Capabilities.TEXTURE_ANIMATION_TOGGLE, NOT_ON_THIS_VERSION));
        page.add(intSlider("perf.block_entity_distance", "Block entity distance", config.blockEntityDistance, 8,
                        v -> v >= BlockEntityCull.VANILLA ? "Vanilla (64)" : v + " blocks")
                .presetMember()
                .availableWhen(() -> Capabilities.BLOCK_ENTITY_CULL, NOT_ON_THIS_VERSION));
        page.add(intSlider("perf.render_scale", "Render scale", config.renderScale, 5,
                        v -> v >= 100 ? "100% (full)" : v + "%")
                .availableWhen(() -> Capabilities.RENDER_SCALE, NOT_ON_THIS_VERSION));
        page.add(bool("perf.smooth_streaming", "Smooth terrain streaming", config.smoothStreaming)
                .availableWhen(() -> Capabilities.PACKET_BUDGET || Capabilities.MESH_THREADS, NOT_ON_THIS_VERSION));
        page.add(bool("perf.coalesce_rebuilds", "Merge chunk rebuilds", config.coalesceRebuilds)
                .availableWhen(() -> Capabilities.COALESCE, NOT_ON_THIS_VERSION));
        page.add(bool("perf.smart_chunk_culling", "Smarter chunk culling", config.smartChunkCulling)
                .availableWhen(() -> Capabilities.STREAMING || Capabilities.FOG_CULL, NOT_ON_THIS_VERSION));
        page.add(bool("perf.fast_particles", "Fast particles", config.fastParticles)
                .availableWhen(() -> Capabilities.FAST_PARTICLES, NOT_ON_THIS_VERSION));
        page.add(bool("perf.visible_animations", "Animate visible textures only", config.visibleAnimations)
                .availableWhen(() -> Capabilities.VISIBLE_ANIMATIONS, NOT_ON_THIS_VERSION));
        page.add(bool("perf.fast_models", "Fast mob models", config.fastModels)
                .availableWhen(() -> Capabilities.FAST_MODELS, NOT_ON_THIS_VERSION));
        page.add(intSlider("perf.worker_threads", "Worker threads", config.workerThreads, 1,
                v -> v == 0 ? "Auto" + autoThreadsSuffix() : v + (v == 1 ? " thread" : " threads")));
        addTuning(page);
        return page;
    }

    /** 1.10.0 tuning options (Minecraft 1.16.5 first). */
    private void addTuning(final Page page) {
        page.add(tuningRow(bool("perf.low_end_mode", "Low-end mode", config.lowEndMode)));
        page.add(tuningRow(bool("perf.dynamic_fps", "Dynamic FPS", config.dynamicFps)));
        page.add(tuningRow(intSlider("perf.menu_fps", "Menu frame rate", config.menuFps, 5,
                v -> v >= Tuning.MENU_FPS_OFF ? "Off (full speed)" : v + " FPS")));
        page.add(tuningRow(bool("perf.hitch_guard", "Hitch guard", config.hitchGuard)));
        page.add(tuningRow(bool("perf.loading_screen_gc", "Memory cleanup on loading screens", config.loadGc)));
        page.add(tuningRow(intSlider("perf.entity_detail_distance", "Entity detail distance",
                config.entityDetailDistance, 4,
                v -> v >= Tuning.VANILLA_DETAIL ? "Vanilla (all details)" : v + " blocks")));
        page.add(tuningRow(bool("perf.crowd_culling", "Crowd culling", config.crowdCulling)));
        page.add(tuningRow(intSlider("perf.explosion_particles", "Explosion clouds per tick",
                config.explosionParticles, 1,
                v -> v >= Tuning.EXPLOSIONS_UNLIMITED ? "Unlimited" : v == 0 ? "Small puffs only" : String.valueOf(v))));
        page.add(tuningRow(bool("perf.light_updates", "Merge light updates", config.lightUpdates)));
        page.add(tuningRow(bool("perf.map_updates", "Throttle map updates", config.mapUpdates)));
        page.add(tuningRow(bool("perf.async_skulls", "Background player-head skins", config.asyncSkulls)));
        page.add(tuningRow(bool("perf.dimension_culling", "Nether roof culling", config.dimensionCulling)));
        page.add(tuningRow(bool("perf.smooth_autosave", "Smooth autosave", config.smoothAutosave)));
        page.add(tuningRow(bool("perf.flight_preload", "Elytra chunk preloading", config.flightPreload)));
        page.add(tuningRow(intSlider("perf.redstone_particle_limit", "Redstone particles per tick",
                config.redstoneParticleLimit, 4,
                v -> v >= ParticleFilter.REDSTONE_UNLIMITED ? "Unlimited" : v == 0 ? "None" : String.valueOf(v))));
        page.add(tuningRow(intSlider("perf.block_entity_limit", "Block entities per frame",
                config.blockEntityLimit, 64,
                v -> v >= Tuning.BLOCK_ENTITIES_UNLIMITED ? "Unlimited" : String.valueOf(v))));
        page.add(tuningRow(bool("perf.translucent_sort_throttle", "Calmer translucent sorting", config.calmSorting)));
        page.add(tuningRow(bool("perf.lazy_far_rebuilds", "Lazy far chunk rebuilds", config.lazyFarRebuilds)));
        page.add(tuningRow(bool("perf.model_lod", "Mob model detail levels", config.modelLod)));
        page.add(tuningRow(bool("perf.menu_world_cache", "Reuse the world behind menus", config.menuWorldCache)));
        page.add(tuningRow(bool("perf.server_lighting", "Faster singleplayer lighting", config.serverLighting)));
        page.add(tuningRow(bool("perf.directional_culling", "Directional cave culling", config.directionalCulling)));
        page.add(tuningRow(bool("perf.view_first_building", "Build chunks in view first", config.viewFirstBuilding)));
        page.add(tuningRow(bool("perf.portal_preload", "Portal preloading", config.portalPreload)));
        page.add(tuningRow(bool("perf.fast_loading", "Faster loading screens", config.fastLoading)));
        page.add(tuningRow(bool("perf.server_idle", "Calmer built-in server", config.serverIdle)));
        page.add(tuningRow(bool("perf.ocean_floor_culling", "Ocean floor culling", config.oceanFloorCulling)));
        page.add(tuningRow(bool("perf.end_optimizations", "End optimizations", config.endOptimizations)));
        page.add(tuningRow(bool("perf.gl_state_cache", "Skip redundant GL calls", config.glStateCache)));
        page.add(tuningRow(bool("perf.cheap_damage_tint", "Cheaper damage tint", config.cheapDamageTint)));
        page.add(tuningRow(bool("perf.lightmap_rate", "Calmer light table updates", config.lightmapRate)));
        page.add(tuningRow(bool("perf.ao_cache", "Bigger smooth-lighting cache", config.aoCache)));
        page.add(tuningRow(bool("perf.async_skins", "Background skin loading", config.asyncSkins)));
        page.add(tuningRow(bool("perf.lazy_search", "Search index on demand", config.lazySearch)));
        page.add(tuningRow(intSlider("perf.hud_fps", "HUD refresh rate", config.hudFps, 5,
                v -> v <= 0 ? "Every frame" : v + " FPS")));
        page.add(tuningRow(bool("perf.entity_tick_culling", "Skip unseen entity updates", config.entityTickCulling)));
        page.add(tuningRow(bool("perf.fast_section_snapshots", "Faster chunk snapshots", config.fastSectionSnapshots)));
        page.add(tuningRow(bool("perf.fast_network", "Faster packet decoding", config.fastNetwork)));
        page.add(tuningRow(bool("perf.batched_chests", "Batched chests", config.batchedChests)));
        page.add(tuningRow(bool("perf.packed_terrain", "Packed terrain", config.packedTerrain)));
        page.add(tuningRow(bool("perf.multi_draw", "Multi-draw terrain", config.multiDraw)));
        page.add(tuningRow(bool("perf.occlusion_queries", "GPU occlusion culling", config.occlusionQueries)));
    }

    private static Setting tuningRow(final Setting setting) {
        final String key = setting.id;
        final boolean stepped = key.startsWith("perf.") && com.aetherium.mixin.Overlap.yielded(key.substring(5));
        return setting.availableWhen(() -> TuningSupport.option(key), stepped ? STEPPED_ASIDE : NOT_ON_THIS_VERSION);
    }

    // ------------------------------------------------------------------ Backend

    private Page backend() {
        final Page page = new Page("Backend", "Backend", "Renderer and runtime", PixelArt.CUBE);
        page.add(Setting.info("backend.api", "Rendering API",
                "Solid and cutout terrain is drawn by the Aetherium renderer when it is on; everything else is vanilla's pipeline with work removed.",
                () -> Aetherium.RENDERING_API + (ChunkRenderer.isDrawing() ? " (Aetherium renderer)" : " (vanilla pipeline)")));
        page.add(Setting.info("backend.gpu", "GPU", "Reported by the driver.", GlInfo::renderer));
        page.add(Setting.info("backend.vendor", "Vendor", "Reported by the driver.", GlInfo::vendor));
        page.add(Setting.info("backend.driver", "Driver", "GL version string reported by the driver.", GlInfo::version));
        page.add(Setting.info("backend.java", "Java", "Runtime running the game.",
                () -> System.getProperty("java.version") + " (" + System.getProperty("java.vendor", "?") + ")"));
        page.add(Setting.info("backend.memory", "Memory", "Heap in use / maximum.", AetheriumPages::memoryText));
        page.add(Setting.info("backend.loader", "Loader", "Mod loader and game version.",
                () -> Aetherium.platform().platformName() + " - Minecraft " + Aetherium.platform().minecraftVersion()));
        page.add(Setting.info("backend.hooks", "Hooks", "Mixins applied on this version.",
                () -> AetheriumMixinPlugin.appliedCount() + " active"));
        page.add(Setting.info("backend.cap", "Frame cap", "Live cap from battery saver / thermal guard.",
                () -> {
                    final int cap = ClientHooks.currentCap();
                    if (cap <= 0) {
                        return "None";
                    }
                    return cap + " FPS" + (ClientHooks.isThermalCapped() ? " (thermal)" : "");
                }));
        page.add(bool("backend.chunk_renderer", "Aetherium renderer", config.aetheriumRenderer)
                .availableWhen(() -> Capabilities.ANY_RENDERER, NOT_ON_THIS_VERSION));
        page.add(bool("backend.region_batching", "Batched terrain", config.regionBatching)
                .availableWhen(() -> Capabilities.AETHERIUM_RENDERER, NOT_ON_THIS_VERSION));
        page.add(Setting.info("backend.chunk_renderer_state", "Chunk renderer",
                "Which path draws solid and cutout terrain right now.",
                () -> Capabilities.SECTION_CULL || Capabilities.SECTION_CULL_MODERN ? SectionCull.statusText() : ChunkRenderer.statusText()));
        page.add(bool("backend.debug", "Debug logging", config.debugLogging));
        page.add(bool("backend.stutter_log", "Stutter log", config.stutterLog)
                .availableWhen(() -> Capabilities.STUTTER_LOG, NOT_ON_THIS_VERSION));
        page.add(bool("backend.delegate", "Hand off to conflicting mods", config.conflictAutoDelegate));
        return page;
    }

    private static String memoryText() {
        final Runtime rt = Runtime.getRuntime();
        final long used = (rt.totalMemory() - rt.freeMemory()) >> 20;
        final long max = rt.maxMemory() >> 20;
        return used + " / " + max + " MB";
    }

    // ------------------------------------------------------------------ Effects

    private Page effects() {
        final Page page = new Page("Effects", "Effects",
                QolSupport.ANY ? "Dynamic lights and brightness" : "Dynamic lights, brightness, zoom and freelook",
                PixelArt.SUN);
        page.add(Setting.choice("effects.dynamic_lights", Setting.Kind.SEGMENTED, "Dynamic lights",
                        config.dynamicLights.getComment(), LIGHTS,
                        () -> config.dynamicLights.get().ordinal(),
                        v -> config.dynamicLights.set(AetheriumConfig.LightMode.values()[clamp(v, 0, 2)]))
                .withDefault(config.dynamicLights.getDefault().ordinal())
                .presetMember()
                .availableWhen(() -> Capabilities.DYNAMIC_LIGHTS, NOT_ON_THIS_VERSION));
        page.add(bool("effects.held", "Held items glow", config.dynamicLightsHeld)
                .availableWhen(() -> Capabilities.DYNAMIC_LIGHTS, NOT_ON_THIS_VERSION));
        page.add(bool("effects.entities", "Glowing entities", config.dynamicLightsEntities)
                .availableWhen(() -> Capabilities.DYNAMIC_LIGHTS, NOT_ON_THIS_VERSION));
        page.add(Setting.info("effects.sources", "Active light sources",
                "Updated live: the number of light sources currently lighting the world around you.",
                () -> {
                    if (Capabilities.DYNAMIC_LIGHTS && !AetheriumMixinPlugin.lightHooksInstalled()) {
                        return "Unavailable (hooks failed)";
                    }
                    if (config.dynamicLights.get() == AetheriumConfig.LightMode.OFF) {
                        return "Off";
                    }
                    return Integer.toString(ClientHooks.activeLightSources());
                }));
        page.add(bool("effects.fullbright", "Fullbright", config.fullbright));
        page.add(intSlider("effects.fullbright_strength", "Fullbright strength", config.fullbrightStrength, 5,
                v -> v + "%"));
        if (!QolSupport.ANY) {
            addZoomAndFreelook(page);
        }
        return page;
    }

    /** Zoom and freelook rows: on the Quality of life tab where it exists, otherwise on Effects. */
    private void addZoomAndFreelook(final Page page) {
        addZoom(page);
        page.add(Setting.info("effects.freelook_key", "Freelook key",
                "Hold Freelook (Left Alt by default) to look around without turning your player; release to snap back. Rebind it in Options > Controls, category Aetherium.",
                () -> FreelookHooks.describeKey()));
        page.add(bool("effects.freelook_toggle", "Freelook toggles", config.freelookToggle)
                .availableWhen(() -> Capabilities.FREELOOK, NOT_ON_THIS_VERSION));
        page.add(bool("effects.freelook_third_person", "Freelook in third person", config.freelookThirdPerson)
                .availableWhen(() -> Capabilities.FREELOOK, NOT_ON_THIS_VERSION));
    }

    private void addZoom(final Page page) {
        page.add(Setting.info("effects.zoom_keys", "Zoom keys",
                "Zoom / Zoom In / Zoom Out. Hold Zoom to zoom (Z by default), scroll or use Zoom In (=) and Zoom Out (-) while zooming. Rebind them in Options > Controls, category Aetherium.",
                () -> Capabilities.ZOOM ? ZoomKeys.describe() : NOT_ON_THIS_VERSION));
        page.add(zoomRow(intSlider("effects.zoom_default", "Zoom strength", config.zoomDefault, 5, v -> v + "%")));
        page.add(zoomRow(bool("effects.zoom_toggle", "Zoom toggles", config.zoomToggle)));
        page.add(zoomRow(bool("effects.zoom_cinematic", "Cinematic zoom", config.zoomCinematic)));
        page.add(zoomRow(bool("effects.zoom_scrolling", "Scroll to zoom", config.zoomScrolling)));
        page.add(zoomRow(intSlider("effects.zoom_speed", "Zoom speed", config.zoomSpeed, 1, v -> Integer.toString(v))));
        page.add(zoomRow(intSlider("effects.zoom_smoothness", "Zoom animation", config.zoomSmoothnessMs, 10,
                v -> v == 0 ? "Instant" : v + " ms")));
        page.add(zoomRow(intSlider("effects.zoom_sensitivity", "Zoomed mouse floor", config.zoomSensitivityFloor, 5,
                v -> v + "%")));
        page.add(zoomRow(intSlider("effects.zoom_min_fov", "Maximum zoom FOV", config.zoomMinFov, 1,
                v -> v + "\u00b0")));
        page.add(zoomRow(intSlider("effects.zoom_third_person", "Third-person zoom", config.zoomThirdPersonDistance, 1,
                v -> v == 0 ? "Off (FOV zoom)" : "0.5-" + v + " blocks")));
        page.add(zoomRow(bool("effects.zoom_third_person_toggle", "Third-person toggles", config.zoomThirdPersonToggle)));
    }

    private static Setting zoomRow(final Setting setting) {
        return setting.availableWhen(() -> Capabilities.ZOOM, NOT_ON_THIS_VERSION);
    }

    // ------------------------------------------------------------------ Quality of life

    private static final String[] CROSSHAIR_STYLES = {"Vanilla", "Cross", "Cross + dot", "Dot", "Circle", "Circle + dot", "T"};
    private static final String[] CROSSHAIR_COLORS = {"White", "Red", "Green", "Cyan", "Yellow", "Magenta", "Black"};
    private static final String[] HAND_PRESETS = {"Custom", "Default", "Lowered", "Centered", "Small", "Big"};
    private static final String[] HAND_ROWS = {
        "qol.hand.x", "qol.hand.y", "qol.hand.z", "qol.hand.pitch", "qol.hand.yaw", "qol.hand.roll", "qol.hand.scale",
    };

    private Page qol() {
        final Page page = new Page("QoL", "Quality of life", "Client-side helpers that work on any server", PixelArt.PIN);
        page.add(qolRow(bool("qol.accurate_placement", "Accurate placement", config.accuratePlacement), QolSupport.PLACEMENT));

        page.add(qolRow(bool("qol.inventory.scroll_transfer", "Mouse Tweaks: wheel", config.scrollTransfer),
                QolSupport.INVENTORY));
        page.add(qolRow(bool("qol.inventory.shift_drag", "Mouse Tweaks: drag", config.shiftDrag), QolSupport.INVENTORY));
        page.add(qolRow(bool("qol.inventory.sort_button", "Sort buttons", config.sortButton), QolSupport.INVENTORY));

        page.add(qolRow(bool("qol.food.tooltip", "Food values in tooltips", config.foodTooltip), QolSupport.FOOD));
        page.add(qolRow(bool("qol.food.hud", "Saturation on the hunger bar", config.foodHud), QolSupport.HUD));

        page.add(qolRow(bool("qol.waypoints", "Waypoints", config.waypoints), QolSupport.WORLD));
        page.add(qolRow(bool("qol.waypoints.beams", "Waypoint beams", config.waypointBeams), QolSupport.WORLD));
        page.add(qolRow(bool("qol.waypoints.cross_dimension", "Nether / Overworld waypoints", config.waypointsCrossDimension),
                QolSupport.WORLD));
        page.add(qolRow(bool("qol.death_point", "Death point", config.deathPoint), QolSupport.WORLD));
        page.add(Setting.button("qol.waypoints.manage", "Manage waypoints",
                        "Opens the waypoint list of the world you are in: rename, recolour, hide or delete waypoints, or add one where you stand.",
                        "Open", actions::openWaypoints)
                .availableWhen(() -> QolSupport.WORLD && Minecraft.getInstance().player != null,
                        QolSupport.WORLD ? "Join a world to manage its waypoints" : NOT_ON_THIS_VERSION));
        page.add(Setting.info("qol.keys", "Quality of life keys",
                "Add waypoint (B), Waypoints (N) and Sort inventory (not bound). Rebind them in Options > Controls, category Aetherium.",
                QolClient::describeKeys));

        page.add(qolRow(bool("qol.dimension_coords", "Nether / Overworld coordinates", config.dimensionCoords), QolSupport.HUD));
        page.add(qolRow(Setting.choice("qol.dimension_coords.corner", Setting.Kind.DROPDOWN, "Coordinates position",
                        config.dimensionCoordsCorner.getComment(), CORNERS,
                        () -> cornerIndex(config.dimensionCoordsCorner.get()),
                        v -> config.dimensionCoordsCorner.set(CORNER_KEYS[clamp(v, 0, 3)]))
                .withDefault(cornerIndex(config.dimensionCoordsCorner.getDefault())), QolSupport.HUD));

        page.add(qolRow(intSlider("qol.fire_overlay", "Fire overlay height", config.fireOverlay, 5,
                v -> v == 100 ? "Vanilla" : v == 0 ? "Hidden" : v + "%"), QolSupport.FIRE));
        page.add(qolRow(intSlider("qol.shield_offset", "Lower raised shield", config.shieldOffset, 5,
                v -> v == 0 ? "Vanilla" : v + "%"), QolSupport.HAND));

        page.add(qolRow(bool("qol.damage_numbers", "Damage numbers", config.damageNumbers), QolSupport.WORLD));
        page.add(qolRow(bool("qol.damage_numbers.heals", "Show healing too", config.damageNumbersHeals), QolSupport.WORLD));

        page.add(qolRow(Setting.choice("qol.crosshair.style", Setting.Kind.DROPDOWN, "Crosshair",
                        config.crosshairStyle.getComment(), CROSSHAIR_STYLES,
                        () -> config.crosshairStyle.get().ordinal(),
                        v -> config.crosshairStyle.set(AetheriumConfig.CrosshairStyle.values()[clamp(v, 0, CROSSHAIR_STYLES.length - 1)]))
                .withDefault(config.crosshairStyle.getDefault().ordinal()), QolSupport.HUD));
        page.add(qolRow(Setting.choice("qol.crosshair.color", Setting.Kind.DROPDOWN, "Crosshair colour",
                        config.crosshairColor.getComment(), CROSSHAIR_COLORS,
                        () -> config.crosshairColor.get().ordinal(),
                        v -> config.crosshairColor.set(AetheriumConfig.CrosshairColor.values()[clamp(v, 0, CROSSHAIR_COLORS.length - 1)]))
                .withDefault(config.crosshairColor.getDefault().ordinal()), QolSupport.HUD));
        page.add(qolRow(intSlider("qol.crosshair.size", "Crosshair size", config.crosshairSize, 1, v -> v + " px"), QolSupport.HUD));
        page.add(qolRow(intSlider("qol.crosshair.thickness", "Crosshair thickness", config.crosshairThickness, 1,
                v -> v + " px"), QolSupport.HUD));
        page.add(qolRow(intSlider("qol.crosshair.gap", "Crosshair gap", config.crosshairGap, 1, v -> v + " px"), QolSupport.HUD));
        page.add(qolRow(intSlider("qol.crosshair.opacity", "Crosshair opacity", config.crosshairOpacity, 5, v -> v + "%"),
                QolSupport.HUD));
        page.add(qolRow(bool("qol.crosshair.outline", "Crosshair outline", config.crosshairOutline), QolSupport.HUD));
        page.add(qolRow(bool("qol.crosshair.target_color", "Red on mobs", config.crosshairTargetColor), QolSupport.HUD));

        page.add(qolRow(bool("qol.hand.enabled", "Hand editor", config.handEditor), QolSupport.HAND).livePreview());
        page.add(qolRow(bool("qol.hand.visible", "Show hand with items", config.handVisible), QolSupport.HAND).livePreview());
        page.add(qolRow(Setting.choice("qol.hand.preset", Setting.Kind.DROPDOWN, "Hand preset",
                        "Moves your hand to a ready-made position (and turns the hand editor on). In a world the menu fades for a moment so you can see the result; adjust the values below afterwards as you like.",
                        HAND_PRESETS,
                        () -> config.handPreset.get().ordinal(),
                        v -> config.handPreset.set(AetheriumConfig.HandPreset.values()[clamp(v, 0, HAND_PRESETS.length - 1)]))
                .withDefault(config.handPreset.getDefault().ordinal())
                .onPick(this::stageHandPreset), QolSupport.HAND).livePreview());
        page.add(handRow(intSlider("qol.hand.x", "Hand sideways", config.handX, 1, AetheriumPages::hundredths)));
        page.add(handRow(intSlider("qol.hand.y", "Hand height", config.handY, 1, AetheriumPages::hundredths)));
        page.add(handRow(intSlider("qol.hand.z", "Hand depth", config.handZ, 1, AetheriumPages::hundredths)));
        page.add(handRow(intSlider("qol.hand.pitch", "Hand tilt", config.handPitch, 1, v -> v + "\u00b0")));
        page.add(handRow(intSlider("qol.hand.yaw", "Hand turn", config.handYaw, 1, v -> v + "\u00b0")));
        page.add(handRow(intSlider("qol.hand.roll", "Hand roll", config.handRoll, 1, v -> v + "\u00b0")));
        page.add(handRow(intSlider("qol.hand.scale", "Hand size", config.handScale, 5, v -> v + "%")));
        page.add(qolRow(bool("qol.hand.mirror", "Same for off hand", config.handMirror), QolSupport.HAND).livePreview());
        page.add(armRow(bool("qol.arm.both", "Show both hands", config.armBoth)));
        page.add(armRow(intSlider("qol.arm.x", "Arms sideways", config.armX, 1, AetheriumPages::hundredths)));
        page.add(armRow(intSlider("qol.arm.y", "Arms height", config.armY, 1, AetheriumPages::hundredths)));
        page.add(armRow(intSlider("qol.arm.z", "Arms distance", config.armZ, 1, AetheriumPages::hundredths)));
        page.add(armRow(intSlider("qol.arm.scale", "Arms size", config.armScale, 5, v -> v + "%")));

        addZoomAndFreelook(page);
        return page;
    }

    private static Setting qolRow(final Setting setting, final boolean supported) {
        return setting.availableWhen(() -> supported, NOT_ON_THIS_VERSION);
    }

    /**
     * A hand value: available with the hand editor, shown live while dragged (the rest of the
     * screen fades away), and editing it turns the preset back into Custom and the editor on.
     */
    private Setting handRow(final Setting setting) {
        return qolRow(setting, QolSupport.HAND).livePreview().onPick(v -> {
            if (this.view != null) {
                final Setting preset = this.view.find("qol.hand.preset");
                if (preset != null) {
                    preset.stage(0);
                }
                enableHandEditor();
            }
        });
    }

    /**
     * Punchy's arm sculpt: moves both arms (and what they hold) together, with or without an item.
     * Like Punchy, touching it shows both hands with whatever they hold, so the change is visible.
     */
    private Setting armRow(final Setting setting) {
        return qolRow(setting, QolSupport.HAND).livePreview().onPick(v -> {
            if (this.view != null) {
                stageOn("qol.hand.visible", "qol.arm.both");
            }
        });
    }

    private void stageOn(final String... ids) {
        for (final String id : ids) {
            final Setting row = this.view.find(id);
            if (row != null && row.isEditable()) {
                row.stage(1);
            }
        }
    }

    private void stageHandPreset(final int index) {
        if (this.view == null || index < 1 || index > Qol.HAND_PRESETS.length) {
            return;
        }
        final int[] values = Qol.HAND_PRESETS[index - 1];
        for (int i = 0; i < HAND_ROWS.length; i++) {
            final Setting row = this.view.find(HAND_ROWS[i]);
            if (row != null && row.isEditable()) {
                row.stage(values[i]);
            }
        }
        enableHandEditor();
    }

    /**
     * A preset or a hand value only shows with the editor on, and the hand itself only with
     * "Show hand with items" (vanilla hides the arm while you hold something), so picking one
     * turns both on, the way Punchy's hand editor always shows the arm.
     */
    private void enableHandEditor() {
        stageOn("qol.hand.enabled", "qol.hand.visible");
    }

    // ------------------------------------------------------------------ hand live preview

    /** The rows of {@link Qol#HAND_KEYS}, in that order; null when the page has none. */
    private Setting[] handRows;
    private boolean handPreviewing;
    /** Reused every previewed frame (no per-frame garbage). */
    private final int[] handValues = new int[Qol.HAND_KEYS.length];

    private void cacheHandRows() {
        final Setting[] rows = new Setting[Qol.HAND_KEYS.length];
        for (int i = 0; i < rows.length; i++) {
            rows[i] = this.view.find(Qol.HAND_KEYS[i]);
        }
        for (final Setting row : rows) {
            if (row == null || !row.isEditable()) {
                this.handRows = null;
                return;
            }
        }
        this.handRows = rows;
    }

    /**
     * Every frame while the screen is open: staged (not yet applied) hand values are shown on the
     * real hand at once, so a preset or a dragged slider can be judged before Apply.
     */
    public void previewFrame() {
        final Setting[] rows = this.handRows;
        if (rows == null) {
            return;
        }
        boolean dirty = false;
        for (final Setting row : rows) {
            if (row.isDirty()) {
                dirty = true;
                break;
            }
        }
        if (dirty) {
            final int[] values = this.handValues;
            for (int i = 0; i < rows.length; i++) {
                values[i] = rows[i].pending();
            }
            Qol.previewHand(values);
            this.handPreviewing = true;
        } else if (this.handPreviewing) {
            endPreview();
        }
    }

    /** The screen closed: back to the applied hand values. */
    public void endPreview() {
        if (this.handPreviewing) {
            this.handPreviewing = false;
            Qol.endHandPreview(this.config);
        }
    }

    /** 12 -> "0.12 blocks". */
    private static String hundredths(final int v) {
        final String sign = v < 0 ? "-" : "";
        final int a = Math.abs(v);
        return sign + (a / 100) + "." + (a % 100 < 10 ? "0" : "") + (a % 100) + " blocks";
    }

    // ------------------------------------------------------------------ Android

    private Page androidPage() {
        final Page page = new Page("Android", "Android", "Mobile launchers and power", PixelArt.ROBOT);
        page.add(Setting.info("android.device", "Device", "Detected launcher and renderer.",
                () -> android.isAndroid()
                        ? android.getLauncher().getDisplayName() + " - " + android.getRenderer().getDisplayName()
                        : "Desktop (not Android)"));
        page.add(Setting.info("android.cpu", "CPU", "Architecture and SIMD features.",
                () -> System.getProperty("os.arch", "?") + (android.hasNeon() ? " - NEON" : "")));
        page.add(bool("android.support", "Android support", config.androidSupport));
        final AetheriumConfig.AndroidRendererChoice[] choices = AetheriumConfig.AndroidRendererChoice.values();
        final String[] names = new String[choices.length];
        for (int i = 0; i < choices.length; i++) {
            names[i] = rendererName(choices[i]);
        }
        page.add(Setting.choice("android.renderer", Setting.Kind.DROPDOWN, "Renderer override",
                        config.androidForceRenderer.getComment(), names,
                        () -> config.androidForceRenderer.get().ordinal(),
                        v -> config.androidForceRenderer.set(choices[clamp(v, 0, choices.length - 1)]))
                .withDefault(config.androidForceRenderer.getDefault().ordinal()));
        page.add(bool("android.custom_env", "Read custom_env.txt", config.readCustomEnv));
        page.add(bool("android.battery", "Battery saver", config.batterySaver));
        page.add(intSlider("android.battery_cap", "Battery FPS cap", config.batteryFpsCap, 5, v -> v + " FPS"));
        page.add(bool("android.thermal", "Thermal guard", config.thermalGuard)
                .availableWhen(android::isAndroid, "Android only"));
        page.add(intSlider("android.thermal_ceiling", "Thermal ceiling", config.thermalCeilingC, 1,
                v -> v + "\u00b0C")
                .availableWhen(android::isAndroid, "Android only"));
        page.add(Setting.info("android.temperature", "Temperature", "Last reading of the device sensor.",
                () -> {
                    if (!android.isAndroid() || !android.isThermalSensorUsable()) {
                        return "No sensor";
                    }
                    return (android.getLastThermalMilliCelsius() / 1000) + "\u00b0C";
                }));
        page.add(bool("android.touch", "Touch mode", config.touchMode));
        page.add(Setting.info("android.heap", "Heap usage", "Share of the maximum heap in use.",
                () -> Math.round(android.heapUsageFraction() * 100.0) + "%"));
        return page;
    }

    private static String rendererName(final AetheriumConfig.AndroidRendererChoice choice) {
        switch (choice) {
            case AUTO:
                return "Auto";
            case GL4ES:
                return "GL4ES";
            case ZINK:
                return "Zink";
            case LTW:
                return "LTW";
            case MOBILEGLUES:
                return "MobileGlues";
            case VIRGL:
                return "VirGL";
            case ANGLE:
                return "ANGLE";
            case NATIVE_VULKAN:
                return "Native Vulkan";
            default:
                return choice.name();
        }
    }

    // ------------------------------------------------------------------ builders

    private static Setting bool(final String id, final String label, final ConfigValue<Boolean> value) {
        return Setting.toggle(id, label, value.getComment(),
                        () -> value.get().booleanValue() ? 1 : 0,
                        v -> value.set(Boolean.valueOf(v != 0)))
                .withDefault(value.getDefault().booleanValue() ? 1 : 0);
    }

    private static Setting intSlider(final String id, final String label, final ConfigValue<Integer> value,
                                     final int step, final IntFunction<String> format) {
        final int min = ((Number) value.getMin()).intValue();
        final int max = ((Number) value.getMax()).intValue();
        return Setting.slider(id, label, value.getComment(), min, max, step, format,
                        () -> value.get().intValue(),
                        v -> value.set(Integer.valueOf(v)))
                .withDefault(value.getDefault().intValue());
    }

    /** A boolean row backed by vanilla getters/setters (no default: Reset leaves vanilla alone). */
    private static Setting flag(final String id, final String label, final String description,
                                final BoolGetter getter, final BoolSetter setter) {
        final IntSupplier reader = () -> getter.get() ? 1 : 0;
        final IntConsumer writer = v -> setter.set(v != 0);
        return Setting.toggle(id, label, description, reader, writer);
    }

    /** " (5)" when this launch resized the pool automatically, "" when vanilla decided. */
    private String autoThreadsSuffix() {
        final int applied = WorkerThreads.applied();
        return applied > 0 && this.config.workerThreads.get().intValue() == 0 ? " (" + applied + ")" : "";
    }

    private static int clamp(final int v, final int lo, final int hi) {
        return Math.max(lo, Math.min(hi, v));
    }

    interface BoolGetter {
        boolean get();
    }

    interface BoolSetter {
        void set(boolean value);
    }
}
