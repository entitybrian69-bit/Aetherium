package com.aetherium.client;

import com.aetherium.Aetherium;
import com.aetherium.Capabilities;
import com.aetherium.android.AndroidEnvironment;
import com.aetherium.android.ThermalMonitor;
import com.aetherium.config.AetheriumConfig;
import com.aetherium.hud.BenchmarkRecorder;
import com.aetherium.hud.FrameStats;
import com.aetherium.lighting.DynamicLightTracker;
import com.aetherium.mixin.AetheriumMixinPlugin;
import com.aetherium.perf.AdaptiveDistance;
import com.aetherium.perf.BlockEntityCull;
import com.aetherium.perf.CrowdCull;
import com.aetherium.perf.FrameLimiter;
import com.aetherium.perf.NearBuild;
import com.aetherium.perf.RenderToggles;
import com.aetherium.perf.SectionVisibility;
import com.aetherium.perf.Tuning;
import com.aetherium.qol.ParticleFilter;
import com.aetherium.util.AetheriumLog;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.core.BlockPos;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.block.entity.BeaconBlockEntity;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity;
import net.minecraft.world.phys.AABB;

/**
 * Everything the mixins call, in one place. Every entry point is render-thread
 * only, allocation-free on the per-frame and per-entity paths, and a no-op
 * while Aetherium is not ACTIVE (master switch off, conflict, or init failure),
 * so turning the mod off really is vanilla.
 */
public final class ClientHooks {

    private static final AetheriumLog LOG = AetheriumLog.of(ClientHooks.class);
    private static final FrameLimiter LIMITER = new FrameLimiter();
    private static final AdaptiveDistance ADAPTIVE = new AdaptiveDistance();

    private static final int ADAPTIVE_PERIOD_TICKS = 40; // 2 s
    private static final int THERMAL_PERIOD_TICKS = 100; // 5 s
    private static final int THERMAL_HYSTERESIS_C = 3;
    private static final int THERMAL_CAP_FPS = 30;

    /** Entities this large stay visible at any distance (dragons, withers, ghasts). */
    private static final float HUGE_ENTITY = 3.0f;

    /** Vanilla's visible sections, re-read on the first entity test of each frame. */
    private static final SectionVisibility VISIBLE = new SectionVisibility();
    private static boolean visibilityFrameDirty = true;

    private static DynamicLightTracker tracker;
    private static boolean initialised;
    private static Object lastLevel;
    private static int ticks;
    private static long lastFrameNanos;

    private static boolean thermalCapped;
    private static ThermalMonitor thermal;
    private static boolean adaptiveWasOn;
    /** Render distance the user picked; adaptive distance never exceeds it and restores it when turned off. */
    private static int userRenderDistance = -1;

    /** Set by the "Vanilla video settings" button so the next video screen is not swapped. */
    private static boolean bypassVideoScreen;

    private ClientHooks() {
    }

    // ------------------------------------------------------------------ per frame

    /** GameRenderer.render HEAD: frame pacing (battery/thermal caps) and frame-time stats. */
    public static void onFrameStart() {
        visibilityFrameDirty = true;
        // Per-frame budgets of the tuning options (crowd culling, block entity limit). On 1.16.5
        // LevelRenderer.renderLevel resets them too; both run once per frame, before any entity.
        CrowdCull.newFrame();
        Tuning.newFrame();
        OceanFloor.frame(Minecraft.getInstance().level, mainCamera(Minecraft.getInstance()));
        // Zoom reads its keys here, before the camera, the FOV and the mouse use its state.
        ZoomKeys.frame(Minecraft.getInstance());
        // Stutter log and adaptive upload budget, then freelook (before the camera is set up).
        QolHooks.frame(Minecraft.getInstance());
        FreelookHooks.frame(Minecraft.getInstance());
        FogCullHooks.frame(Minecraft.getInstance());
        ChunkRenderer.onFrame();
        if (Aetherium.isActive() && AetheriumMixinPlugin.isMixinActive()) {
            LIMITER.beforeFrame();
        }
        if (BlockEntityCull.active()) {
            final Entity view = Minecraft.getInstance().getCameraEntity();
            if (view != null) {
                BlockEntityCull.setView(view.getX(), view.getEyeY(), view.getZ());
            }
        }
        final long now = System.nanoTime();
        final long last = lastFrameNanos;
        lastFrameNanos = now;
        if (last == 0L) {
            return;
        }
        final Aetherium.Subsystems sub = Aetherium.subsystemsOrNull();
        if (sub != null) {
            final FrameStats stats = sub.frameStats;
            stats.record(now - last);
            BenchmarkRecorder.onFrame(stats);
            AutoBench.onFrame(now - last);
        }
    }

    /**
     * EntityRenderDispatcher.shouldRender HEAD.
     *
     * @return true to skip rendering this entity this frame
     */
    public static boolean shouldCullEntity(final Entity entity, final double camX, final double camY, final double camZ) {
        if (!RenderToggles.entityCulling) {
            return false;
        }
        final double distanceSq = entity.distanceToSqr(camX, camY, camZ);
        if (!RenderToggles.beyondCullDistance(distanceSq)) {
            return false;
        }
        if (entity.getBbWidth() >= HUGE_ENTITY || entity.getBbHeight() >= HUGE_ENTITY) {
            return false;
        }
        final Minecraft mc = Minecraft.getInstance();
        if (entity == mc.getCameraEntity() || entity == mc.player) {
            return false;
        }
        if (mc.player != null && (entity == mc.player.getVehicle() || entity.hasPassenger(mc.player))) {
            return false;
        }
        return true;
    }

    /**
     * EntityRenderer.shouldRender RETURN, only when vanilla said "visible": true when every section
     * the entity's culling box touches is absent from vanilla's visible-section list, i.e. the
     * entity is behind terrain. Players (name tags show through walls), named and glowing entities
     * are never hidden; neither is anything the grid is unsure about (see SectionVisibility).
     */
    public static boolean hiddenBehindTerrain(final Entity entity, final AABB box, final double camX,
                                              final double camY, final double camZ) {
        if (!RenderToggles.occlusionCulling || box == null) {
            return false;
        }
        final Minecraft mc = Minecraft.getInstance();
        if (visibilityFrameDirty) {
            visibilityFrameDirty = false;
            collectVisibleSections(mc, camX, camY, camZ);
        }
        // Half a block of slack matches vanilla's own frustum test and covers render offsets.
        if (VISIBLE.isBoxVisible(box.minX - 0.5, box.minY - 0.5, box.minZ - 0.5,
                box.maxX + 0.5, box.maxY + 0.5, box.maxZ + 0.5)) {
            return false;
        }
        // Exemptions are only evaluated for entities that would be hidden (the cheap test runs first).
        return !(entity instanceof Player) && !entity.isCustomNameVisible()
                && entity != mc.getCameraEntity() && !mc.shouldEntityAppearGlowing(entity);
    }

    private static void collectVisibleSections(final Minecraft mc, final double camX, final double camY,
                                               final double camZ) {
        final Object renderer = mc.levelRenderer;
        // Without the frame-start hook the grid would never be refreshed; stale data must not cull.
        if (!(renderer instanceof VisibleSectionSource) || !AetheriumMixinPlugin.isApplied("FrameMixin")) {
            VISIBLE.invalidate();
            return;
        }
        try {
            VISIBLE.begin(SectionVisibility.floorSection(camX), SectionVisibility.floorSection(camY),
                    SectionVisibility.floorSection(camZ));
            ((VisibleSectionSource) renderer).aetheriumCollectVisible(VISIBLE);
        } catch (final RuntimeException | LinkageError error) {
            // A changed vanilla list must cost a feature, never a frame: stand down for the session.
            VISIBLE.invalidate();
            RenderToggles.breakOcclusion();
            LOG.warn("Entity occlusion culling disabled: {}", error.toString());
        }
    }

    /** ParticleEngine.add HEAD: true to drop the particle. */
    public static boolean shouldDropParticle() {
        return RenderToggles.active && RenderToggles.dropParticle();
    }

    public static boolean hideWeather() {
        return RenderToggles.hideWeather;
    }

    /** BlockEntityRenderDispatcher.render HEAD: true to skip a block entity past the distance limit. */
    public static boolean shouldCullBlockEntity(final BlockEntity blockEntity) {
        if (!BlockEntityCull.active() || blockEntity instanceof BeaconBlockEntity || blockEntity instanceof TheEndGatewayBlockEntity) {
            return false;
        }
        final BlockPos pos = blockEntity.getBlockPos();
        return BlockEntityCull.beyond(pos.getX(), pos.getY(), pos.getZ());
    }

    /** TextureAtlas.cycleAnimationFrames HEAD: true to keep animated textures on their current frame. */
    public static boolean freezeTextureAnimations() {
        return RenderToggles.freezeTextureAnimations;
    }

    public static boolean hideVignette() {
        return RenderToggles.hideVignette;
    }

    // ------------------------------------------------------------------ per tick

    /** Minecraft.tick TAIL. */
    public static void onClientTick(final Minecraft mc) {
        // The client thread owns the text measurement caches (other threads measure normally).
        com.aetherium.perf.TextWidths.setOwner(Thread.currentThread());
        com.aetherium.perf.TextWidths.tick();
        AutoBench.tick(mc);
        com.aetherium.mixin.SmokeTest.tick();
        final Aetherium.Subsystems sub = Aetherium.subsystemsOrNull();
        if (sub == null) {
            return;
        }
        if (!initialised) {
            initialised = true;
            tracker = new DynamicLightTracker(sub.config);
            userRenderDistance = VanillaOptions.getRenderDistance();
            adaptiveWasOn = sub.config.adaptiveDistance.get().booleanValue();
            onSettingsApplied();
        }
        sub.store.tick();
        ParticleFilter.newTick();
        TuningClient.tick(mc);

        if (mc.level != lastLevel) {
            lastLevel = mc.level;
            tracker.reset();
            ZoomKeys.reset();
            QolHooks.onLevelChange();
            QolClient.onLevelChange();
            TuningClient.levelChanged();
            ADAPTIVE.reset();
        }
        QolClient.onClientTick(mc);

        final boolean active = Aetherium.isActive() && AetheriumMixinPlugin.isMixinActive();
        if (mc.level == null || mc.player == null) {
            // Menus without a world: only the background / minimized caps can apply here, and a cap
            // picked up in a world (an open menu) must not stay behind.
            if (active) {
                updateCap(sub.config);
            } else {
                LIMITER.setCap(0);
            }
            return;
        }
        ticks++;
        final AetheriumConfig config = sub.config;

        // Without the light hook (it failed to apply) the tracker would only cause pointless chunk
        // rebuilds, so it stays idle.
        final boolean lightsAllowed = active && Capabilities.DYNAMIC_LIGHTS && AetheriumMixinPlugin.lightHooksInstalled();
        tracker.tick(mc, lightsAllowed);

        if (!active) {
            LIMITER.setCap(0);
            return;
        }

        // Not while a menu is open: its frame-rate cap and the reused world picture behind it say
        // nothing about how fast the world itself draws.
        if (ticks % ADAPTIVE_PERIOD_TICKS == 0 && config.adaptiveDistance.get().booleanValue() && currentScreen(mc) == null) {
            final int current = VanillaOptions.getRenderDistance();
            final int next = ADAPTIVE.evaluate(sub.frameStats.getFps(), config.adaptiveTargetFps.get().intValue(),
                    current, userRenderDistance > 0 ? userRenderDistance : current);
            if (next != current) {
                VanillaOptions.setRenderDistance(next);
            }
        }

        if (ticks % THERMAL_PERIOD_TICKS == 0) {
            updateThermal(sub.android, config);
        }
        updateCap(config);
    }

    private static void updateThermal(final AndroidEnvironment android, final AetheriumConfig config) {
        // Phones only: desktop CPUs sit above any sensible phone ceiling under normal load.
        final boolean wanted = config.thermalGuard.get().booleanValue() && android.isAndroid()
                && android.isThermalSensorUsable();
        if (thermal == null) {
            if (!wanted) {
                thermalCapped = false;
                return;
            }
            thermal = new ThermalMonitor(android);
        }
        thermal.setWanted(wanted);
        if (!wanted) {
            thermalCapped = false;
            return;
        }
        // Read on a background thread; the render thread only picks up the latest value.
        final int milli = thermal.lastMilliCelsius();
        if (milli <= 0) {
            return;
        }
        final int ceiling = config.thermalCeilingC.get().intValue();
        final int celsius = milli / 1000;
        if (celsius >= ceiling) {
            thermalCapped = true;
        } else if (celsius < ceiling - THERMAL_HYSTERESIS_C) {
            thermalCapped = false;
        }
    }

    private static void updateCap(final AetheriumConfig config) {
        int cap = 0;
        if (config.batterySaver.get().booleanValue()) {
            cap = config.batteryFpsCap.get().intValue();
        }
        if (thermalCapped) {
            cap = cap == 0 ? THERMAL_CAP_FPS : Math.min(cap, THERMAL_CAP_FPS);
        }
        // 1.10.0: background / minimized / open-menu caps (Minecraft 1.16.5; 0 elsewhere).
        final int tuning = TuningClient.frameCap(Minecraft.getInstance());
        if (tuning > 0) {
            cap = cap == 0 ? tuning : Math.min(cap, tuning);
        }
        LIMITER.setCap(cap);
    }

    /** Current live frame cap (0 = none), for the GUI. */
    public static int currentCap() {
        return LIMITER.getCap();
    }

    public static boolean isThermalCapped() {
        return thermalCapped;
    }

    public static int activeLightSources() {
        return tracker == null ? 0 : tracker.activeSources();
    }

    // ------------------------------------------------------------------ settings

    /** The user picked a render distance in our GUI; adaptive distance treats it as the ceiling. */
    public static void noteUserRenderDistance(final int chunks) {
        userRenderDistance = chunks;
        ADAPTIVE.reset();
    }

    /** Republishes every live flag from the config. Called after Apply and once at startup. */
    public static void onSettingsApplied() {
        final Aetherium.Subsystems sub = Aetherium.subsystemsOrNull();
        if (sub == null) {
            return;
        }
        Aetherium.refreshState();
        final AetheriumConfig config = sub.config;
        final boolean active = Aetherium.isActive() && AetheriumMixinPlugin.isMixinActive();
        ZoomKeys.configure(config);
        NearBuild.configure(config);
        QolHooks.configure(config);
        QolClient.configure(config, active);
        Tuning.configure(config, active && Capabilities.TUNING);
        TuningClient.configured(Minecraft.getInstance());

        if (active && config.fullbright.get().booleanValue()) {
            final double strength = config.fullbrightStrength.get().intValue() / 100.0;
            VanillaOptions.setGammaOverride(1.0 + strength * 14.0);
        } else {
            VanillaOptions.setGammaOverride(Double.NaN);
        }

        final boolean adaptiveOn = active && config.adaptiveDistance.get().booleanValue();
        if (adaptiveWasOn && !adaptiveOn && userRenderDistance > 0
                && VanillaOptions.getRenderDistance() != userRenderDistance) {
            VanillaOptions.setRenderDistance(userRenderDistance);
        }
        adaptiveWasOn = adaptiveOn;
        ADAPTIVE.reset();

        if (tracker != null && (!active || config.dynamicLights.get() == AetheriumConfig.LightMode.OFF)) {
            tracker.clear(Minecraft.getInstance());
        }
        if (!config.thermalGuard.get().booleanValue()) {
            thermalCapped = false;
            if (thermal != null) {
                thermal.setWanted(false);
            }
        }
        if (!active) {
            LIMITER.setCap(0);
        } else {
            updateCap(config);
        }
    }

    // ------------------------------------------------------------------ screens

    /**
     * Minecraft.setScreen HEAD. Returns the screen to open instead of {@code next},
     * or null to let vanilla proceed unchanged.
     */
    /**
     * Aetherium's settings screen for Mod Menu's "Configure" button; the parent itself (nothing
     * happens) when Aetherium failed to start.
     */
    public static Screen settingsScreen(final Screen parent) {
        if (Aetherium.subsystemsOrNull() == null) {
            return parent;
        }
        return new AetheriumScreen(parent);
    }

    public static Screen replaceScreen(final Minecraft mc, final Screen next) {
        TuningClient.onScreen(mc, next);
        if (!isVideoSettings(next)) {
            return null;
        }
        if (bypassVideoScreen) {
            bypassVideoScreen = false;
            return null;
        }
        if (Aetherium.subsystemsOrNull() == null) {
            return null;
        }
        return new AetheriumScreen(currentScreen(mc));
    }

    /** Opens vanilla's own video screen on top of ours (escape hatch). */
    public static void openVanillaVideoSettings(final Minecraft mc, final Screen parent) {
        bypassVideoScreen = true;
        // @era:screen-pkg-begin options
        setScreen(mc, new net.minecraft.client.gui.screens.options.VideoSettingsScreen(parent, mc, mc.options));
        // @era:screen-pkg-else flat
        //~ setScreen(mc, new net.minecraft.client.gui.screens.VideoSettingsScreen(parent, mc.options));
        // @era:screen-pkg-end
        bypassVideoScreen = false;
    }

    /** The main camera; its getter became record-style in 26.2. Null before the renderer exists. */
    public static net.minecraft.client.Camera mainCamera(final Minecraft mc) {
        if (mc == null || mc.gameRenderer == null) {
            return null;
        }
        // @era:camera-get-begin getter
        return mc.gameRenderer.getMainCamera();
        // @era:camera-get-else record
        //~ return mc.gameRenderer.mainCamera();
        // @era:camera-get-end
    }

    /** The camera's eye height (Camera.getPosition became position() in 1.21.6). */
    public static double cameraY(final net.minecraft.client.Camera camera) {
        // @era:cam-pos-begin getter
        return camera.getPosition().y;
        // @era:cam-pos-else position
        //~ return camera.position().y;
        // @era:cam-pos-end
    }

    /** The entity the camera follows (Camera.getEntity became entity() in 1.21.11). */
    public static net.minecraft.world.entity.Entity cameraEntity(final net.minecraft.client.Camera camera) {
        // @era:camera-entity-begin getter
        return camera.getEntity();
        // @era:camera-entity-else record
        //~ return camera.entity();
        // @era:camera-entity-end
    }

    /** The camera's yaw in degrees (Camera.getYRot became yRot() in 1.21.11). */
    public static float cameraYaw(final net.minecraft.client.Camera camera) {
        // @era:camera-entity-begin getter
        return camera.getYRot();
        // @era:camera-entity-else record
        //~ return camera.yRot();
        // @era:camera-entity-end
    }

    /** The camera's pitch in degrees, positive looking down (getXRot became xRot() in 1.21.11). */
    public static float cameraPitch(final net.minecraft.client.Camera camera) {
        // @era:camera-entity-begin getter
        return camera.getXRot();
        // @era:camera-entity-else record
        //~ return camera.xRot();
        // @era:camera-entity-end
    }

    /** The camera's position (Camera.getPosition became position() in 1.21.6). */
    public static net.minecraft.world.phys.Vec3 cameraPosition(final net.minecraft.client.Camera camera) {
        // @era:cam-pos-begin getter
        return camera.getPosition();
        // @era:cam-pos-else position
        //~ return camera.position();
        // @era:cam-pos-end
    }

    /** Squared distance from the camera to a block centre (render thread); 0 before the camera exists. */
    public static double cameraDistanceSq(final net.minecraft.core.BlockPos pos) {
        final net.minecraft.client.Camera camera = mainCamera(Minecraft.getInstance());
        if (camera == null) {
            return 0.0;
        }
        // @era:cam-pos-begin getter
        final net.minecraft.world.phys.Vec3 cam = camera.getPosition();
        // @era:cam-pos-else position
        //~ final net.minecraft.world.phys.Vec3 cam = camera.position();
        // @era:cam-pos-end
        final double dx = pos.getX() + 0.5 - cam.x;
        final double dy = pos.getY() + 0.5 - cam.y;
        final double dz = pos.getZ() + 0.5 - cam.z;
        return dx * dx + dy * dy + dz * dz;
    }

    /** The open screen; it moved from {@code Minecraft} to {@code Gui} in 26.2. */
    public static Screen currentScreen(final Minecraft mc) {
        // @era:screen-owner-begin minecraft
        return mc.screen;
        // @era:screen-owner-else gui
        //~ return mc.gui.screen();
        // @era:screen-owner-end
    }

    public static void setScreen(final Minecraft mc, final Screen screen) {
        // @era:screen-owner-begin minecraft
        mc.setScreen(screen);
        // @era:screen-owner-else gui
        //~ mc.gui.setScreen(screen);
        // @era:screen-owner-end
    }

    private static boolean isVideoSettings(final Screen screen) {
        // @era:screen-pkg-begin options
        return screen instanceof net.minecraft.client.gui.screens.options.VideoSettingsScreen;
        // @era:screen-pkg-else flat
        //~ return screen instanceof net.minecraft.client.gui.screens.VideoSettingsScreen;
        // @era:screen-pkg-end
    }

    /** A plain chat/tooltip text component (TextComponent before 1.19, Component.literal after). */
    public static net.minecraft.network.chat.MutableComponent literal(final String text) {
        // @era:chat-begin literal
        return net.minecraft.network.chat.Component.literal(text);
        // @era:chat-else text
        //~ return new net.minecraft.network.chat.TextComponent(text);
        // @era:chat-end
    }
}
