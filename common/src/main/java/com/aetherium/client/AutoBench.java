package com.aetherium.client;

import net.minecraft.client.Minecraft;
// @era:autobench-begin none
// @era:autobench-else create
//~ import java.io.File;
//~ import java.io.IOException;
//~ import java.lang.management.GarbageCollectorMXBean;
//~ import java.lang.management.ManagementFactory;
//~ import java.nio.charset.StandardCharsets;
//~ import java.nio.file.Files;
//~ import java.util.ArrayList;
//~ import java.util.List;
//~ import java.util.Locale;

//~ import com.aetherium.Aetherium;
//~ import com.aetherium.config.AetheriumConfig;
//~ import com.aetherium.hud.BenchStats;
//~ import com.aetherium.util.AetheriumLog;

//~ import net.minecraft.client.Screenshot;
//~ import net.minecraft.client.gui.screens.TitleScreen;
//~ import net.minecraft.client.gui.screens.worldselection.WorldPreset;
//~ import net.minecraft.client.player.LocalPlayer;
//~ import net.minecraft.core.RegistryAccess;
//~ import net.minecraft.world.Difficulty;
//~ import net.minecraft.world.level.DataPackConfig;
//~ import net.minecraft.world.level.GameRules;
//~ import net.minecraft.world.level.GameType;
//~ import net.minecraft.world.level.LevelSettings;
//~ import net.minecraft.world.level.levelgen.WorldGenSettings;
// @era:autobench-end

/**
 * Self-driving benchmark for CI (1.9.0; 1.16.5 for now). Enabled only by
 * {@code -Daetherium.autobench=<directory>}: from the title screen it creates a fixed-seed
 * world, then measures several scenarios (vanilla renderer, Aetherium, Aetherium with render
 * scale, flying into new terrain with and without Aetherium), takes a screenshot of each,
 * writes {@code report.md} and {@code results.json} into the directory and closes the game.
 * A normal player never has the property, so {@link #ACTIVE} is false and both hooks return
 * on their first comparison.
 */
public final class AutoBench {
    public static final String PROPERTY = "aetherium.autobench";
    public static final boolean ACTIVE = System.getProperty(PROPERTY) != null;

    // @era:autobench-begin none
    // @era:autobench-else create
    //~ private static final AetheriumLog LOGGER = AetheriumLog.of(AutoBench.class);
    //~ private static final long SEED = 7_294_105_550_031L;
    //~ private static final String WORLD = "aetherium-bench";
    //~ private static final int RENDER_DISTANCE = 8;
    //~ /** Whole run, in client ticks (25 minutes): after that the partial report is written. */
    //~ private static final int MAX_TICKS = 25 * 60 * 20;
    //~ /** Degrees per second the camera turns while a static scenario is measured. */
    //~ private static final float SPIN_DEG_PER_SECOND = 36.0F;
    //~ private static final float PITCH = 12.0F;
    //~ /** Blocks per tick while flying into new terrain (20 blocks/s, about elytra speed). */
    //~ private static final double FLY_SPEED = 1.0;

    //~ private static final class Scenario {
        //~ final String id;
        //~ final String label;
        //~ final boolean aetherium;
        //~ final int scale;
        //~ /** 0 = turn in place, +1/-1 = fly along +X/-X, +2/-2 = fly along +Z/-Z. */
        //~ final int fly;
        //~ final int seconds;
        //~ final boolean reported;
        //~ /** Options switched off ("+key": on) for this scenario (all others at their defaults). */
        //~ final String[] off;

        //~ Scenario(final String id, final String label, final boolean aetherium, final int scale, final int fly,
                 //~ final int seconds, final boolean reported, final String... off) {
            //~ this.off = off;
            //~ this.id = id;
            //~ this.label = label;
            //~ this.aetherium = aetherium;
            //~ this.scale = scale;
            //~ this.fly = fly;
            //~ this.seconds = seconds;
            //~ this.reported = reported;
        //~ }
    //~ }

    //~ /** The 1.12.0 performance options that are on by default. */
    //~ private static final String[] NEW_OPTIONS = {
        //~ "performance.server_lighting", "performance.directional_culling", "performance.view_first_building",
        //~ "performance.portal_preload", "performance.fast_loading", "performance.server_idle",
        //~ "performance.end_optimizations", "performance.gl_state_cache", "performance.cheap_damage_tint",
        //~ "performance.lightmap_rate", "performance.ao_cache", "performance.async_skins",
    //~ };

    //~ /** The 1.17.2 performance options that are on by default (HUD refresh rate goes to 0 = vanilla). */
    //~ private static final String[] BOOST_OPTIONS = {
        //~ "performance.lazy_search", "performance.hud_fps", "performance.entity_tick_culling",
        //~ "performance.fast_section_snapshots", "performance.fast_network",
        //~ "performance.batched_chests", "performance.packed_terrain", "performance.multi_draw",
    //~ };

    //~ private static final Scenario[] SCENARIOS = {
        //~ new Scenario("warmup", "Warm-up", true, 100, 0, 15, false),
        //~ new Scenario("vanilla", "Vanilla (Aetherium off)", false, 100, 0, 20, true),
        //~ new Scenario("aetherium", "Aetherium (defaults)", true, 100, 0, 20, true),
        //~ new Scenario("new-off", "Aetherium, all 1.12.0 options off", true, 100, 0, 20, true, NEW_OPTIONS),
        //~ new Scenario("no-cull", "Aetherium, directional culling + view-first building off", true, 100, 0, 20, true,
                //~ "performance.directional_culling", "performance.view_first_building"),
        //~ new Scenario("no-gl", "Aetherium, GL call trimming + cheap damage tint off", true, 100, 0, 20, true,
                //~ "performance.gl_state_cache", "performance.cheap_damage_tint"),
        //~ new Scenario("boost-off", "Aetherium, all 1.17.2 options off", true, 100, 0, 20, true, BOOST_OPTIONS),
        //~ new Scenario("occlusion", "Aetherium + GPU occlusion culling", true, 100, 0, 20, true,
                //~ "+performance.occlusion_queries"),
        //~ new Scenario("aetherium-again", "Aetherium (defaults), measured again", true, 100, 0, 20, true),
        //~ new Scenario("scale70", "Aetherium + render scale 70%", true, 70, 0, 20, true),
        //~ new Scenario("fly-vanilla", "Flying into new terrain (+X), Aetherium off", false, 100, 1, 20, true),
        //~ new Scenario("fly-aetherium", "Flying into new terrain (-X), Aetherium defaults", true, 100, -1, 20, true),
        //~ new Scenario("fly-boost-off", "Flying into new terrain (+Z), all 1.17.2 options off", true, 100, 2, 20, true,
                //~ BOOST_OPTIONS),
        //~ new Scenario("fly-aetherium-z", "Flying into new terrain (-Z), Aetherium defaults", true, 100, -2, 20, true),
    //~ };

    //~ private enum Phase { TITLE, LOADING, SETUP, SETTLE, MEASURE, SHOT, DONE, STOPPED }

    //~ private static Phase phase = Phase.TITLE;
    //~ private static int ticks;
    //~ private static int phaseTicks;
    //~ private static int scenario;
    //~ private static int quietTicks;
    //~ private static double baseX;
    //~ private static double baseY;
    //~ private static double baseZ;
    //~ private static long measureStart;
    //~ private static long[] frames = new long[1 << 16];
    //~ private static int frameCount;
    //~ private static long gcCountStart;
    //~ private static long gcMillisStart;
    //~ private static final List<String> ROWS = new ArrayList<>();
    //~ private static final List<String> JSON = new ArrayList<>();
    //~ private static final List<String> NOTES = new ArrayList<>();
    // @era:autobench-end

    private AutoBench() {
    }

    /** Frame start (ClientHooks.onFrameStart): the previous frame's duration. */
    public static void onFrame(final long frameNanos) {
        if (!ACTIVE) {
            return;
        }
        // @era:autobench-begin none
        // @era:autobench-else create
        //~ if (phase != Phase.MEASURE) {
            //~ return;
        //~ }
        //~ if (frameCount < frames.length) {
            //~ frames[frameCount++] = frameNanos;
        //~ }
        //~ final Scenario s = SCENARIOS[scenario];
        //~ final LocalPlayer player = Minecraft.getInstance().player;
        //~ if (player != null && s.fly == 0) {
            //~ final float seconds = (System.nanoTime() - measureStart) / 1e9F;
            //~ final float yaw = (seconds * SPIN_DEG_PER_SECOND) % 360.0F;
            //~ player.yRot = yaw;
            //~ player.yRotO = yaw;
            //~ player.xRot = PITCH;
            //~ player.xRotO = PITCH;
        //~ }
        // @era:autobench-end
    }

    /** Client tick (ClientHooks.onClientTick). */
    public static void tick(final Minecraft mc) {
        if (!ACTIVE) {
            return;
        }
        // @era:autobench-begin none
        // @era:autobench-else create
        //~ if (phase == Phase.STOPPED) {
            //~ return;
        //~ }
        //~ ticks++;
        //~ phaseTicks++;
        //~ try {
            //~ if (ticks > MAX_TICKS && phase != Phase.DONE) {
                //~ NOTES.add("Timed out in phase " + phase + " (scenario " + SCENARIOS[Math.min(scenario, SCENARIOS.length - 1)].id + ").");
                //~ enter(Phase.DONE);
            //~ }
            //~ step(mc);
        //~ } catch (final Throwable error) {
            //~ LOGGER.error("Auto benchmark failed", error);
            //~ NOTES.add("Failed in phase " + phase + ": " + error);
            //~ writeReport(mc);
            //~ phase = Phase.STOPPED;
            //~ mc.stop();
        //~ }
        // @era:autobench-end
    }

    // @era:autobench-begin none
    // @era:autobench-else create
    //~ private static void enter(final Phase next) {
        //~ phase = next;
        //~ phaseTicks = 0;
    //~ }

    //~ private static void step(final Minecraft mc) {
        //~ switch (phase) {
            //~ case TITLE:
                //~ if (mc.getOverlay() == null && mc.screen instanceof TitleScreen && phaseTicks > 40) {
                    //~ mc.options.pauseOnLostFocus = false;
                    //~ mc.options.renderDistance = RENDER_DISTANCE;
                    //~ mc.options.framerateLimit = 260;
                    //~ mc.options.enableVsync = false;
                    //~ mc.getWindow().updateVsync(false);
                    //~ final RegistryAccess.RegistryHolder registries = RegistryAccess.builtin();
                    //~ final LevelSettings settings = new LevelSettings(WORLD, GameType.SPECTATOR, false,
                            //~ Difficulty.PEACEFUL, true, new GameRules(), DataPackConfig.DEFAULT);
                    //~ final WorldGenSettings generator = WorldPreset.NORMAL.create(registries, SEED, true, false);
                    //~ LOGGER.info("Auto benchmark: creating world {}", WORLD);
                    //~ enter(Phase.LOADING);
                    //~ mc.createLevel(WORLD, settings, registries, generator);
                //~ }
                //~ break;
            //~ case LOADING:
                //~ if (mc.level != null && mc.player != null) {
                    //~ if (mc.screen != null) {
                        //~ mc.setScreen(null);
                    //~ }
                    //~ if (phaseTicks > 40) {
                        //~ final LocalPlayer player = mc.player;
                        //~ player.chat("/gamerule doDaylightCycle false");
                        //~ player.chat("/gamerule doWeatherCycle false");
                        //~ player.chat("/time set 6000");
                        //~ player.chat("/weather clear");
                        //~ player.chat("/tp @s ~ ~40 ~ 0 0");
                        //~ // Chests in front of the camera (single, double, ender) for the batched chests check.
                        //~ player.chat("/setblock ~ ~-1 ~5 minecraft:chest[facing=north]");
                        //~ player.chat("/setblock ~2 ~-1 ~5 minecraft:chest[facing=north,type=left]");
                        //~ player.chat("/setblock ~3 ~-1 ~5 minecraft:chest[facing=north,type=right]");
                        //~ player.chat("/setblock ~-2 ~-1 ~5 minecraft:ender_chest[facing=north]");
                        //~ enter(Phase.SETUP);
                    //~ }
                //~ }
                //~ break;
            //~ case SETUP:
                //~ if (mc.player == null) {
                    //~ break;
                //~ }
                //~ if (mc.screen != null) {
                    //~ mc.setScreen(null);
                //~ }
                //~ if (scenario == 0 && phaseTicks < 40) {
                    //~ break;
                //~ }
                //~ if (scenario == 0) {
                    //~ baseX = mc.player.getX();
                    //~ baseY = mc.player.getY();
                    //~ baseZ = mc.player.getZ();
                //~ }
                //~ apply(mc, SCENARIOS[scenario]);
                //~ enter(Phase.SETTLE);
                //~ quietTicks = 0;
                //~ break;
            //~ case SETTLE:
                //~ if (mc.screen != null) {
                    //~ mc.setScreen(null);
                //~ }
                //~ holdAtBase(mc);
                //~ quietTicks = mc.levelRenderer.hasRenderedAllChunks() ? quietTicks + 1 : 0;
                //~ // At least 5 s, then 2 s with nothing left to build; never more than 120 s.
                //~ if ((phaseTicks > 100 && quietTicks > 40) || phaseTicks > 2400) {
                    //~ if (phaseTicks > 2400) {
                        //~ NOTES.add(SCENARIOS[scenario].id + ": chunks still building after 120 s; measured anyway.");
                    //~ }
                    //~ startMeasure();
                //~ }
                //~ break;
            //~ case MEASURE: {
                //~ final Scenario s = SCENARIOS[scenario];
                //~ if (s.fly != 0 && mc.player != null) {
                    //~ final LocalPlayer player = mc.player;
                    //~ final boolean alongX = Math.abs(s.fly) == 1;
                    //~ final int sign = s.fly > 0 ? 1 : -1;
                    //~ player.yRot = alongX ? (sign > 0 ? -90.0F : 90.0F) : (sign > 0 ? 0.0F : 180.0F);
                    //~ player.yRotO = player.yRot;
                    //~ player.xRot = 0.0F;
                    //~ player.xRotO = 0.0F;
                    //~ if (alongX) {
                        //~ player.setPos(player.getX() + sign * FLY_SPEED, baseY, baseZ);
                    //~ } else {
                        //~ player.setPos(baseX, baseY, player.getZ() + sign * FLY_SPEED);
                    //~ }
                //~ }
                //~ if (System.nanoTime() - measureStart >= s.seconds * 1_000_000_000L) {
                    //~ finishMeasure(mc, s);
                    //~ enter(Phase.SHOT);
                //~ }
                //~ break;
            //~ }
            //~ case SHOT: {
                //~ final Scenario s = SCENARIOS[scenario];
                //~ if (s.fly == 0) {
                    //~ holdAtBase(mc);
                //~ }
                //~ // The debug screen is shown for the screenshot so the HUD (and the HUD refresh rate's
                //~ // reused picture) can be compared between scenarios.
                //~ if (phaseTicks == 24 && s.reported && s.fly == 0) {
                    //~ mc.options.renderDebug = true;
                //~ }
                //~ if (phaseTicks == 30 && s.reported && s.fly == 0) {
                    //~ Screenshot.grab(mc.gameDirectory, "bench-" + s.id + ".png", mc.getWindow().getWidth(),
                            //~ mc.getWindow().getHeight(), mc.getMainRenderTarget(), message -> { });
                //~ }
                //~ if (phaseTicks == 31) {
                    //~ mc.options.renderDebug = false;
                //~ }
                //~ if (phaseTicks > 40) {
                    //~ scenario++;
                    //~ enter(scenario < SCENARIOS.length ? Phase.SETUP : Phase.DONE);
                //~ }
                //~ break;
            //~ }
            //~ case DONE:
                //~ if (phaseTicks == 1) {
                    //~ writeReport(mc);
                //~ }
                //~ if (phaseTicks > 60) {
                    //~ phase = Phase.STOPPED;
                    //~ LOGGER.info("Auto benchmark finished, closing the game");
                    //~ mc.stop();
                //~ }
                //~ break;
            //~ default:
                //~ break;
        //~ }
    //~ }

    //~ private static void apply(final Minecraft mc, final Scenario s) {
        //~ final AetheriumConfig config = Aetherium.config();
        //~ config.enabled.set(Boolean.valueOf(s.aetherium));
        //~ config.renderScale.set(Integer.valueOf(s.scale));
        //~ for (final String key : NEW_OPTIONS) {
            //~ config.byKey(key).reset();
        //~ }
        //~ for (final String key : BOOST_OPTIONS) {
            //~ config.byKey(key).reset();
        //~ }
        //~ config.byKey("performance.occlusion_queries").reset();
        //~ for (final String key : s.off) {
            //~ if (key.startsWith("+")) {
                //~ config.byKey(key.substring(1)).accept(Boolean.TRUE);
            //~ } else if (key.endsWith(".hud_fps")) {
                //~ config.byKey(key).accept(Integer.valueOf(0));
            //~ } else {
                //~ config.byKey(key).accept(Boolean.FALSE);
            //~ }
        //~ }
        //~ Aetherium.refreshState();
        //~ // Republish every flag (Tuning and friends follow "enabled" and the options above only here).
        //~ ClientHooks.onSettingsApplied();
        //~ if (mc.player != null) {
            //~ mc.player.setPos(baseX, baseY, baseZ);
        //~ }
        //~ // Every scenario starts from a full rebuild so they are measured alike.
        //~ mc.levelRenderer.allChanged();
        //~ LOGGER.info("Auto benchmark: scenario {}", s.id);
    //~ }

    //~ private static void holdAtBase(final Minecraft mc) {
        //~ final LocalPlayer player = mc.player;
        //~ if (player == null) {
            //~ return;
        //~ }
        //~ player.setPos(baseX, baseY, baseZ);
        //~ player.yRot = 0.0F;
        //~ player.yRotO = 0.0F;
        //~ player.xRot = PITCH;
        //~ player.xRotO = PITCH;
    //~ }

    //~ private static void startMeasure() {
        //~ frameCount = 0;
        //~ gcCountStart = gcCount();
        //~ gcMillisStart = gcMillis();
        //~ measureStart = System.nanoTime();
        //~ enter(Phase.MEASURE);
    //~ }

    //~ private static void finishMeasure(final Minecraft mc, final Scenario s) {
        //~ final BenchStats stats = BenchStats.of(frames, frameCount);
        //~ LOGGER.info("Auto benchmark: {} -> {}", s.id, stats.markdownRow(s.label));
        //~ if (!s.reported) {
            //~ return;
        //~ }
        //~ ROWS.add(stats.markdownRow(s.label));
        //~ JSON.add(stats.json(s.id));
        //~ NOTES.add(String.format(Locale.ROOT, "%s: %d GC collections, %d ms in GC; renderer: %s; regions: %s; render scale: %d%%; chests baked so far: %d%s",
                //~ s.id, gcCount() - gcCountStart, gcMillis() - gcMillisStart, ChunkRenderer.statusText(),
                //~ TerrainRegions.status(), RenderScale.lastPercent(), Long.valueOf(ChestBaking.baked()),
                //~ ChestBaking.failed() ? " (failed, turned off)" : ""));
    //~ }

    //~ private static long gcCount() {
        //~ long total = 0L;
        //~ for (final GarbageCollectorMXBean bean : ManagementFactory.getGarbageCollectorMXBeans()) {
            //~ total += Math.max(0L, bean.getCollectionCount());
        //~ }
        //~ return total;
    //~ }

    //~ private static long gcMillis() {
        //~ long total = 0L;
        //~ for (final GarbageCollectorMXBean bean : ManagementFactory.getGarbageCollectorMXBeans()) {
            //~ total += Math.max(0L, bean.getCollectionTime());
        //~ }
        //~ return total;
    //~ }

    //~ private static void writeReport(final Minecraft mc) {
        //~ try {
            //~ final File dir = new File(System.getProperty(PROPERTY));
            //~ dir.mkdirs();
            //~ final StringBuilder md = new StringBuilder();
            //~ md.append("# Aetherium auto benchmark (1.16.5)\n\n");
            //~ md.append(String.format(Locale.ROOT, "Window %dx%d, render distance %d, seed %d, GPU: %s, Java %s, %d CPUs.\n\n",
                    //~ mc.getWindow().getWidth(), mc.getWindow().getHeight(), RENDER_DISTANCE, SEED, GlInfo.renderer(),
                    //~ System.getProperty("java.version"), Runtime.getRuntime().availableProcessors()));
            //~ md.append(BenchStats.markdownHeader()).append('\n');
            //~ for (final String row : ROWS) {
                //~ md.append(row).append('\n');
            //~ }
            //~ md.append("\n## Notes\n\n");
            //~ for (final String note : NOTES) {
                //~ md.append("- ").append(note).append('\n');
            //~ }
            //~ Files.write(new File(dir, "report.md").toPath(), md.toString().getBytes(StandardCharsets.UTF_8));
            //~ Files.write(new File(dir, "results.json").toPath(),
                    //~ ("[" + String.join(",\n", JSON) + "]\n").getBytes(StandardCharsets.UTF_8));
        //~ } catch (final IOException | RuntimeException error) {
            //~ LOGGER.error("Auto benchmark: could not write the report", error);
        //~ }
    //~ }
    // @era:autobench-end
}
