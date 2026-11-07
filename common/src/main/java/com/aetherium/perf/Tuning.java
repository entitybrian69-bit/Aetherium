package com.aetherium.perf;

import com.aetherium.config.AetheriumConfig;
import com.aetherium.qol.ParticleFilter;

/**
 * Live snapshot of the 1.10.0 tuning options, read by the hooks with one volatile load each.
 *
 * <p>Minecraft-free on purpose: the hooks live in mixins (Minecraft 1.16.5 only, see
 * {@code Capabilities.TUNING}) and only read these fields; {@link #configure} is called from
 * {@code ClientHooks.onSettingsApplied}. When Aetherium is off (master switch) every flag falls
 * back to vanilla behaviour.</p>
 *
 * <p>Low-end mode is one switch that tightens every limit at once without touching the user's own
 * values, so turning it off restores exactly what was set before.</p>
 */
public final class Tuning {
    /** Distance beyond which entity details are dropped, as "vanilla": no limit. */
    public static final int VANILLA_DETAIL = 64;
    /** Explosion slider value that means "unlimited" (vanilla). */
    public static final int EXPLOSIONS_UNLIMITED = 32;
    /** Menu frame-rate slider value that means "no cap". */
    public static final int MENU_FPS_OFF = 120;
    /** Block entity limit value meaning "no limit" (1.11.0). */
    public static final int BLOCK_ENTITIES_UNLIMITED = 1024;

    public static volatile boolean on;
    public static volatile boolean lowEnd;

    // frame rate
    public static volatile boolean dynamicFps;
    public static volatile int menuFps;
    public static volatile boolean hitchGuard;
    public static volatile boolean loadGc;

    // entities
    /** Squared distance beyond which shadows, armour, held items, frame items and minecart blocks are skipped; MAX = off. */
    public static volatile double detailDistanceSq = Double.MAX_VALUE;
    public static volatile boolean crowdCull;
    public static volatile boolean glint = true;
    public static volatile boolean crystalBeams = true;

    // world
    public static volatile int explosionEmitters = EXPLOSIONS_UNLIMITED;
    public static volatile boolean lightWeather;
    /** 0 = follow Graphics, 1 = fast, 2 = fancy. */
    public static volatile int leaves;
    public static volatile boolean leafCulling;
    /** Whether leaves are currently meshed see-through (set where vanilla decides it, setFancy). */
    public static volatile boolean leavesFancy = true;
    public static volatile boolean dimensionCulling;
    public static volatile boolean lightUpdates;
    public static volatile boolean mapThrottle;
    public static volatile boolean asyncSkulls;

    // integrated server
    public static volatile boolean smoothAutosave;
    public static volatile boolean flightPreload;
    /** Block entities drawn per frame (Integer.MAX_VALUE = no limit). */
    public static volatile int blockEntityLimit = Integer.MAX_VALUE;
    /** Render thread only: block entities drawn so far this frame. */
    private static int blockEntitiesThisFrame;
    public static volatile boolean calmSorting;
    public static volatile boolean modelLod;
    public static volatile boolean menuWorldCache;
    /** Far terrain distance in chunks, 0 = off. */
    public static volatile int farTerrain;
    /** Colour filter: ordinal of {@code AetheriumConfig.ShaderPreset}, 0 = off. */
    public static volatile int shaderPreset;

    // 1.12.0
    public static volatile boolean serverLighting;
    public static volatile boolean directionalCulling;
    public static volatile boolean viewFirstBuilding;
    public static volatile boolean portalPreload;
    public static volatile boolean fastLoading;
    public static volatile boolean serverIdle;
    public static volatile boolean oceanFloorCulling;
    public static volatile boolean endOptimizations;
    public static volatile boolean glStateCache;
    public static volatile boolean cheapDamageTint;
    public static volatile boolean lightmapRate;
    public static volatile boolean aoCache;
    public static volatile boolean asyncSkins;

    // 1.17.2 (the HUD rate lives in HudRate)
    public static volatile boolean lazySearch;
    public static volatile boolean entityTickCulling;
    public static volatile boolean fastSectionSnapshots;
    public static volatile boolean fastNetwork;
    public static volatile boolean multiDraw;
    public static volatile boolean batchedChests;
    public static volatile boolean packedTerrain;
    public static volatile boolean occlusionQueries;
    /** Cloud area in percent of vanilla's (100 = vanilla). */
    public static volatile int cloudDistance = 100;
    public static volatile boolean fastGrass;

    /** Set by the server thread while vanilla's periodic autosave runs (MinecraftServer.tickServer). */
    public static volatile boolean autosaving;

    private Tuning() {
    }

    /** False when another installed mod already changes what the option changes (see Overlap). */
    private static boolean free(final String option) {
        return !com.aetherium.mixin.Overlap.yielded(option);
    }

    public static void configure(final AetheriumConfig c, final boolean active) {
        final boolean low = active && c.lowEndMode.get().booleanValue();
        lowEnd = low;
        dynamicFps = active && c.dynamicFps.get().booleanValue();
        int menu = c.menuFps.get().intValue();
        if (low) {
            menu = Math.min(menu, 20);
        }
        menuFps = active && menu < MENU_FPS_OFF ? menu : 0;
        hitchGuard = active && c.hitchGuard.get().booleanValue();
        loadGc = active && c.loadGc.get().booleanValue();

        int detail = c.entityDetailDistance.get().intValue();
        if (low) {
            detail = Math.min(detail, 16);
        }
        detailDistanceSq = active && detail < VANILLA_DETAIL ? (double) detail * detail : Double.MAX_VALUE;
        crowdCull = active && c.crowdCulling.get().booleanValue();
        glint = !active || c.enchantGlint.get().booleanValue();
        crystalBeams = !active || c.endCrystalBeams.get().booleanValue();

        int explosions = c.explosionParticles.get().intValue();
        if (low) {
            explosions = Math.min(explosions, 2);
        }
        explosionEmitters = active ? explosions : EXPLOSIONS_UNLIMITED;
        lightWeather = active && (low || c.lightWeather.get().booleanValue());
        final AetheriumConfig.LeavesMode mode = c.leaves.get();
        leaves = !active ? 0 : low ? 1 : mode == AetheriumConfig.LeavesMode.FAST ? 1
                : mode == AetheriumConfig.LeavesMode.FANCY ? 2 : 0;
        leafCulling = active && c.leafCulling.get().booleanValue();
        dimensionCulling = active && c.dimensionCulling.get().booleanValue();
        lightUpdates = active && c.lightUpdates.get().booleanValue();
        mapThrottle = active && c.mapUpdates.get().booleanValue();
        asyncSkulls = active && c.asyncSkulls.get().booleanValue();
        smoothAutosave = active && c.smoothAutosave.get().booleanValue();
        flightPreload = active && c.flightPreload.get().booleanValue();
        int redstone = c.redstoneParticleLimit.get().intValue();
        int blockEntities = c.blockEntityLimit.get().intValue();
        if (low) {
            redstone = Math.min(redstone, 8);
            blockEntities = Math.min(blockEntities, 256);
        }
        redstoneParticles = active ? redstone : ParticleFilter.REDSTONE_UNLIMITED;
        ParticleFilter.setRedstoneLimit(redstoneParticles);
        blockEntityLimit = active && blockEntities < BLOCK_ENTITIES_UNLIMITED ? blockEntities : Integer.MAX_VALUE;
        calmSorting = active && c.calmSorting.get().booleanValue();
        RebuildCoalescer.setLazyFar(active && c.lazyFarRebuilds.get().booleanValue());
        modelLod = active && c.modelLod.get().booleanValue();
        menuWorldCache = active && c.menuWorldCache.get().booleanValue();
        farTerrain = active && !low ? c.farTerrain.get().intValue() : 0;
        shaderPreset = active && !low ? c.shaderPreset.get().ordinal() : 0;
        serverLighting = active && c.serverLighting.get().booleanValue();
        directionalCulling = active && c.directionalCulling.get().booleanValue();
        viewFirstBuilding = active && c.viewFirstBuilding.get().booleanValue();
        portalPreload = active && c.portalPreload.get().booleanValue();
        fastLoading = active && c.fastLoading.get().booleanValue();
        serverIdle = active && c.serverIdle.get().booleanValue();
        oceanFloorCulling = active && c.oceanFloorCulling.get().booleanValue();
        endOptimizations = active && c.endOptimizations.get().booleanValue();
        glStateCache = active && c.glStateCache.get().booleanValue();
        cheapDamageTint = active && c.cheapDamageTint.get().booleanValue();
        lightmapRate = active && c.lightmapRate.get().booleanValue();
        aoCache = active && c.aoCache.get().booleanValue();
        asyncSkins = active && c.asyncSkins.get().booleanValue();
        lazySearch = active && c.lazySearch.get().booleanValue() && free("lazy_search");
        HudRate.setFps(active && free("hud_fps") ? c.hudFps.get().intValue() : HudRate.OFF);
        entityTickCulling = active && c.entityTickCulling.get().booleanValue() && free("entity_tick_culling");
        fastSectionSnapshots = active && c.fastSectionSnapshots.get().booleanValue() && free("fast_section_snapshots");
        fastNetwork = active && c.fastNetwork.get().booleanValue() && free("fast_network");
        multiDraw = active && c.multiDraw.get().booleanValue();
        batchedChests = active && c.batchedChests.get().booleanValue() && free("batched_chests");
        packedTerrain = active && c.packedTerrain.get().booleanValue();
        occlusionQueries = active && c.occlusionQueries.get().booleanValue();
        int clouds = c.cloudDistance.get().intValue();
        if (low) {
            clouds = Math.min(clouds, 50);
        }
        cloudDistance = active ? clouds : 100;
        fastGrass = active && (low || c.fastGrass.get().booleanValue());
        on = active;
    }

    /** Redstone particles per tick wanted by these options ({@code ParticleFilter.REDSTONE_UNLIMITED} = none). */
    public static volatile int redstoneParticles = ParticleFilter.REDSTONE_UNLIMITED;

    /** Start of a frame (LevelRenderer.renderLevel HEAD). */
    public static void newFrame() {
        blockEntitiesThisFrame = 0;
    }

    /** One more block entity this frame; false once the per-frame limit is reached. */
    public static boolean allowBlockEntity() {
        if (blockEntitiesThisFrame >= blockEntityLimit) {
            return false;
        }
        blockEntitiesThisFrame++;
        return true;
    }

    /** True when an entity at this squared distance from the camera should lose its details. */
    public static boolean beyondDetail(final double distanceSq) {
        return distanceSq > detailDistanceSq;
    }
}
