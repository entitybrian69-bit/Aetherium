package com.aetherium.config;

import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Objects;

/**
 * Every Aetherium option, in declaration order. Declaration order is load-bearing:
 * it drives {@code CONFIG_SCHEMA.json}, the GUI tab ordering and the file layout
 * of {@code aetherium.json}, so a delta patch that only adds an option still
 * produces a deterministic GUI.
 *
 * <p>Keys are {@code group.name}; {@link #byGroup(String)} powers the sidebar
 * tabs. Instances are created once in {@link #createDefaults()} and shared —
 * {@code Aetherium.config()} hands out the one live instance, matching how
 * Sodium's {@code GameOptions} singleton works. Never construct a second copy
 * at runtime: listeners bound by the GUI would be orphaned.</p>
 */
public final class AetheriumConfig {
    /** Schema version; bumped by a migration when the key set changes shape. */
    public static final int CURRENT_VERSION = 5;

    // --------------------------------------------------------------- general
    public final ConfigValue<Boolean> enabled = ConfigValue.bool(
            "general.enabled", true,
            "Master switch. true = OpenGL 3.0 backend (Aetherium hooks active), false = Vanilla. Live, no restart.");
    public final ConfigValue<Preset> preset = ConfigValue.enumerated(
            "general.preset", Preset.CUSTOM, Preset.class, false,
            "Last applied performance preset; any manual change turns it back into CUSTOM.");
    public final ConfigValue<Boolean> showFrameHud = ConfigValue.bool("general.hud.fps", false, "Corner FPS overlay.");
    public final ConfigValue<String> hudCorner = ConfigValue.string(
            "general.hud.corner", "top-left", "Overlay anchor: top-left, top-right, bottom-left, bottom-right.");
    public final ConfigValue<Boolean> notifyConflicts = ConfigValue.bool("general.notify_conflicts", true, "Log and show a notice when a conflicting mod is present.");
    public final ConfigValue<Boolean> darkMode = ConfigValue.bool("general.ui.dark_mode", false, "Dark theme for the Aetherium settings screen.");
    public final ConfigValue<Boolean> uiSounds = ConfigValue.bool("general.ui.sounds", true, "Interface sounds in the Aetherium settings screen.");

    // ------------------------------------------------------------ performance
    public final ConfigValue<Boolean> entityCulling = ConfigValue.bool("performance.entity_culling", true, "Skip rendering entities beyond the cull distance (bosses, vehicles and the camera entity are exempt).");
    public final ConfigValue<Boolean> occlusionCulling = ConfigValue.bool(
            "performance.occlusion_culling", true, "Skip entities hidden behind terrain, using the chunk visibility vanilla already computes for blocks. Players, named and glowing entities are never hidden.");
    public final ConfigValue<Integer> entityCullDistance = ConfigValue.intRange(
            "performance.entity_cull_distance", 64, 16, 256, false, "Entities farther than this many blocks are not rendered.");
    public final ConfigValue<Integer> particleDensity = ConfigValue.intRange(
            "performance.particle_density", 100, 0, 100, false, "Percentage of spawned particles that are kept.");
    public final ConfigValue<Boolean> adaptiveDistance = ConfigValue.bool(
            "performance.adaptive_distance", false, "Lower the render distance by one chunk after FPS stays under the target for 10 seconds, raise it after 30 seconds of headroom. Every change reloads all chunks, so it waits 30 seconds between changes.");
    public final ConfigValue<Integer> adaptiveTargetFps = ConfigValue.intRange(
            "performance.adaptive_target_fps", 60, 20, 240, false, "FPS the adaptive render distance tries to hold.");
    public final ConfigValue<Boolean> animatedTextures = ConfigValue.bool(
            "performance.animated_textures", true, "Animate water, lava, fire, portals and other animated block textures. Off skips every animation texture upload, a large saving on mobile GL layers.");
    public final ConfigValue<Integer> blockEntityDistance = ConfigValue.intRange(
            "performance.block_entity_distance", 64, 16, 64, false, "Chests, signs, banners, heads and other block entities farther than this many blocks are not drawn (64 = vanilla; beacon beams are exempt).");
    public final ConfigValue<Integer> renderScale = ConfigValue.intRange(
            "performance.render_scale", 100, 50, 100, false, "Draw the 3D world at this percentage of the window resolution and scale it up to the window; the HUD, menus and text stay sharp. 100 = full resolution. Lower values give a large FPS gain when the graphics chip is the limit (phones, integrated graphics, high resolutions). Not used with Fabulous graphics or while a glowing entity is on screen.");
    public final ConfigValue<Boolean> smoothChunkLoading = ConfigValue.bool(
            "performance.smooth_chunk_loading", true, "Spread finished chunk meshes over several frames instead of uploading them all in one frame, which removes the stutter while chunks load or generate.");
    public final ConfigValue<Boolean> threadedChunkBuilder = ConfigValue.bool(
            "performance.threaded_chunk_builder", true, "Rebuild changed chunks near you on the worker threads instead of freezing the frame (vanilla 1.16.5 and 1.17 rebuild everything within 28 blocks on the render thread). Blocks you place or break still update instantly. Also spreads client light updates over frames. The biggest fix for stutter while chunks load and while walking with dynamic lights.");
    public final ConfigValue<Boolean> smoothStreaming = ConfigValue.bool(
            "performance.smooth_streaming", true, "Spread the work of arriving chunks over several frames: incoming chunk, light and entity packets get a time budget per frame instead of all being processed in one frame, and chunk meshes are converted for the GPU on the worker threads. Removes the big frame drops when walking or flying into unloaded terrain.");
    public final ConfigValue<Boolean> coalesceRebuilds = ConfigValue.bool(
            "performance.coalesce_rebuilds", true, "While terrain loads, every chunk section is marked for rebuilding again each time a neighbouring chunk or its light arrives. Wait until a far or new section has been quiet for a moment, then build it once instead of up to ten times. Sections you edit, and built sections within 28 blocks, are never delayed.");
    public final ConfigValue<Boolean> smartChunkCulling = ConfigValue.bool(
            "performance.smart_chunk_culling", true, "Chunk sections that lie completely inside the distance fog are not drawn or rebuilt (every version). On 1.16.5 the visible-chunk search also stops at them, and while terrain streams in that search runs at most 20 times a second instead of every frame (turning the camera still updates it at once). Steps aside when the fog is removed.");
    public final ConfigValue<Boolean> fastParticles = ConfigValue.bool(
            "performance.fast_particles", true, "Minecraft 1.16.5 to 1.21.8: particles behind you or off screen are skipped, and the rest are written to the particle batch without the temporary vectors and quaternion copies vanilla creates for every particle every frame. Looks identical.");
    public final ConfigValue<Boolean> visibleAnimations = ConfigValue.bool(
            "performance.visible_animations", true, "Minecraft 1.16.5 to 1.21.4: only animated block textures you can see (in visible chunks, held or shown items, block entities, particles) keep animating; the rest pause, which saves their texture uploads every tick. Fire and portals always animate. If an animated texture from another mod stops moving, turn this off.");
    public final ConfigValue<Boolean> fastModels = ConfigValue.bool(
            "performance.fast_models", true, "Minecraft 1.16.5 to 1.20.6: mob, player and block-entity models are written straight into the vertex buffer, without the temporary vector vanilla creates for every vertex and the per-vertex buffer calls. Same bytes as vanilla; outlines, enchantment glint and unusual buffers keep the vanilla path.");
    public final ConfigValue<Boolean> framePacing = ConfigValue.bool(
            "performance.frame_pacing", true, "With a Max framerate set, frames are spaced evenly: each frame ends one frame-length after the previous one was due, so a late frame is made up instead of pushing every later frame back. Set Max framerate to your screen's refresh rate (60, 90 or 120) for the steadiest picture.");
    public final ConfigValue<Integer> workerThreads = ConfigValue.intRange(
            "performance.worker_threads", 0, 0, 16, false, "Threads for world generation. 0 = automatic (vanilla on desktop; on Android half the cores minus one, between 2 and 4, so the render and server threads are not starved). Chunk meshes are built on Aetherium's own threads. Applies after a restart.");

    public final ConfigValue<Boolean> lowEndMode = ConfigValue.bool(
            "performance.low_end_mode", false, "Minecraft 1.16.5: one switch for weak phones and PCs. Tightens every limit below at once (entity details beyond 16 blocks, menus at 20 FPS, at most 2 explosion clouds per tick, 8 redstone particles per tick, 256 block entities per frame, fast leaves, lighter rain and snow, no far terrain and no colour filter) without changing your own values, so turning it off restores them.");
    public final ConfigValue<Boolean> dynamicFps = ConfigValue.bool(
            "performance.dynamic_fps", true, "Minecraft 1.16.5: while the game window is in the background the frame rate drops to 15 FPS, and to 1 FPS while it is minimized. Saves battery and keeps the phone cool; full speed returns the moment you switch back.");
    public final ConfigValue<Integer> menuFps = ConfigValue.intRange(
            "performance.menu_fps", 30, 10, 120, false, "Minecraft 1.16.5: frame rate while an inventory, chest or pause menu is open in a world (the chat is excluded, so typing stays smooth). The world behind the menu barely changes, so drawing it 30 times a second looks the same and costs far less. 120 = no limit.");
    public final ConfigValue<Boolean> hitchGuard = ConfigValue.bool(
            "performance.hitch_guard", true, "Minecraft 1.16.5: after a frame that took much longer than usual (a garbage collection, a save, a burst of chunks), chunk uploads and light updates slow down for the next 20 frames, so the work waiting behind it does not cause a second hitch right away.");
    public final ConfigValue<Boolean> loadGc = ConfigValue.bool(
            "performance.loading_screen_gc", true, "Minecraft 1.16.5: clean up memory while a loading screen is shown (joining a world or server, changing dimension), when a short pause is invisible, instead of in the middle of gameplay.");
    public final ConfigValue<Integer> entityDetailDistance = ConfigValue.intRange(
            "performance.entity_detail_distance", 32, 8, 64, false, "Minecraft 1.16.5: mobs, armor stands and item frames farther than this many blocks are drawn without their small details: shadows, armor, held items, elytra, heads worn on the head, stuck arrows, items in item frames and blocks in minecarts. Sign text is not drawn beyond this distance either. Players always keep everything. 64 = vanilla.");
    public final ConfigValue<Boolean> crowdCulling = ConfigValue.bool(
            "performance.crowd_culling", true, "Minecraft 1.16.5: in places packed with entities (item piles, mob farms) only the first 4 dropped items and 6 mobs in each block space are drawn; the rest are still there and still work, they just are not drawn on top of each other.");
    public final ConfigValue<Integer> explosionParticles = ConfigValue.intRange(
            "performance.explosion_particles", 4, 0, 32, false, "Minecraft 1.16.5: at most this many big explosion clouds per game tick; the rest of a TNT chain shows the small puff instead. Prevents the frame-rate collapse of large TNT blasts. 0 = small puffs only, 32 = unlimited (vanilla).");
    public final ConfigValue<Boolean> lightUpdates = ConfigValue.bool(
            "performance.light_updates", true, "Minecraft 1.16.5: light changes reported by the server are collected and each chunk section is rebuilt once every 2 ticks instead of once per change, and the client's own light updates get a per-frame quota (smaller right after a hitch) instead of all running in one frame.");
    public final ConfigValue<Boolean> mapUpdates = ConfigValue.bool(
            "performance.map_updates", true, "Minecraft 1.16.5: held maps and maps in item frames redraw their texture at most 4 times per second instead of whenever they change.");
    public final ConfigValue<Boolean> asyncSkulls = ConfigValue.bool(
            "performance.async_skulls", true, "Minecraft 1.16.5: player heads look up their owner's skin on a background thread. Vanilla asks Mojang's servers on the main thread, which freezes the game for every new head that comes into view.");
    public final ConfigValue<Boolean> dimensionCulling = ConfigValue.bool(
            "performance.dimension_culling", true, "Minecraft 1.16.5: in the Nether, the chunks above the bedrock roof are not drawn while you are below it (you cannot see them through the roof).");
    public final ConfigValue<Boolean> smoothAutosave = ConfigValue.bool(
            "performance.smooth_autosave", true, "Minecraft 1.16.5, singleplayer: the automatic save every 5 minutes writes 16 chunks per tick instead of all of them in one tick, which removes the regular freeze. Saving when you pause or quit is unchanged and saves everything at once.");
    public final ConfigValue<Boolean> flightPreload = ConfigValue.bool(
            "performance.flight_preload", true, "Minecraft 1.16.5, singleplayer: while flying with an elytra, the chunks about 2 seconds ahead of you are loaded early, so you fly into a finished world instead of into empty space.");
    public final ConfigValue<Integer> redstoneParticleLimit = ConfigValue.intRange(
            "performance.redstone_particle_limit", 16, 0, 64, false, "Minecraft 1.16.5: at most this many redstone dust particles (wire, torches, repeaters, redstone ore) appear per game tick. Big contraptions and torch walls then cost a fixed amount instead of thousands of particles. 64 = unlimited (vanilla). Turning Redstone particles off on the Quality tab removes them completely.");
    public final ConfigValue<Integer> blockEntityLimit = ConfigValue.intRange(
            "performance.block_entity_limit", 512, 64, 1024, false, "Minecraft 1.16.5: at most this many block entities (chests, signs, banners, heads, beds, shulker boxes) are drawn per frame, nearest chunks first. Storage rooms and builds with thousands of them stop dragging the frame rate down; the farthest ones are simply not drawn that frame. 1024 = unlimited (vanilla).");
    public final ConfigValue<Boolean> calmSorting = ConfigValue.bool(
            "performance.translucent_sort_throttle", true, "Minecraft 1.16.5: water, stained glass and ice are re-sorted after you have moved 2 blocks instead of every block, and at most 8 chunk sections per move instead of 15. Sorting runs on the worker threads, so this frees them for building chunks. Translucent surfaces can look slightly wrong for a moment while you walk.");
    public final ConfigValue<Boolean> lazyFarRebuilds = ConfigValue.bool(
            "performance.lazy_far_rebuilds", true, "Minecraft 1.16.5, with Merge chunk rebuilds on: chunk sections more than 80 blocks away that are already built keep their old look for up to a second after they change, so light flicker, crops and arriving neighbours far away cost one rebuild per second at most. Your own block changes are always instant.");
    public final ConfigValue<Boolean> modelLod = ConfigValue.bool(
            "performance.model_lod", true, "Minecraft 1.16.5: beyond the Entity detail distance, the tiny parts of mob models (ears, noses, beaks, whiskers, horns: at most 2 pixels across) are not drawn, and beyond twice that distance parts up to 3 pixels across are skipped too. They are a few screen pixels at that range. Players and armor stands are never changed.");
    public final ConfigValue<Boolean> menuWorldCache = ConfigValue.bool(
            "performance.menu_world_cache", true, "Minecraft 1.16.5: while an inventory, chest, crafting or pause menu is open, the world behind it is drawn 4 times a second and the last picture is reused in between, so only the menu itself is drawn every frame. Menus open and scroll much more smoothly on weak devices. The chat is excluded, so the world stays live while you type.");
    public final ConfigValue<Boolean> serverLighting = ConfigValue.bool(
            "performance.server_lighting", true, "Minecraft 1.16.5, singleplayer: the world's lighting engine works through 64 queued light jobs per pass instead of 5, the batch size vanilla itself uses while preparing the spawn area. New chunks get their light, and become visible, sooner while you explore.");
    public final ConfigValue<Boolean> directionalCulling = ConfigValue.bool(
            "performance.directional_culling", true, "Minecraft 1.16.5: the search for visible chunk sections only ever moves away from you, never back towards you. Caves, ravines and rooms you cannot see into are skipped much more often, which means fewer chunks drawn and built underground and in mountains.");
    public final ConfigValue<Boolean> viewFirstBuilding = ConfigValue.bool(
            "performance.view_first_building", true, "Minecraft 1.16.5: chunks in front of you are built before chunks behind you at the same distance, so the world fills in where you are looking first. The nearest chunks still come first.");
    public final ConfigValue<Boolean> portalPreload = ConfigValue.bool(
            "performance.portal_preload", true, "Minecraft 1.16.5, singleplayer: while you stand in a Nether portal, the chunks around the other side start loading in the background, so the trip itself is much shorter.");
    public final ConfigValue<Boolean> fastLoading = ConfigValue.bool(
            "performance.fast_loading", true, "Minecraft 1.16.5: loading screens get the computer to themselves. The world is not drawn behind the Loading terrain screen of a join or dimension change (it is fully hidden there anyway), and while a singleplayer world is being prepared the progress screen is redrawn about 20 times a second instead of about 60, so the built-in server has more CPU to prepare the world. Faster world joins, dimension changes and portal trips on phones.");
    public final ConfigValue<Boolean> serverIdle = ConfigValue.bool(
            "performance.server_idle", true, "Minecraft 1.16.5, singleplayer: between ticks the built-in server checks for work 1000 times a second instead of 10000. New work still wakes it at once, so nothing gets slower, but it stops using up a CPU core your phone needs for drawing.");
    public final ConfigValue<Boolean> oceanFloorCulling = ConfigValue.bool(
            "performance.ocean_floor_culling", false, "Minecraft 1.16.5, experimental: while you are above the water, chunk sections more than 8 blocks below the floor of an ocean are not drawn. Caves under the sea are not visible from up there anyway; a rare opening in the sea floor can show a gap, which is why this is off by default.");
    public final ConfigValue<Boolean> endOptimizations = ConfigValue.bool(
            "performance.end_optimizations", true, "Minecraft 1.16.5: in the End, faces where two chorus plant stems join are not built (they are hidden inside each other), and the End sky is drawn in one draw call instead of six. Chorus forests near End cities get noticeably lighter.");
    public final ConfigValue<Boolean> glStateCache = ConfigValue.bool(
            "performance.gl_state_cache", true, "Minecraft 1.16.5: framebuffer, viewport and vertex-array switches that would not change anything are not sent to the graphics driver. Helps most on phones, where every call goes through a GL translator. Turned off automatically next to OptiFine, Iris, Canvas or Sodium.");
    public final ConfigValue<Boolean> cheapDamageTint = ConfigValue.bool(
            "performance.cheap_damage_tint", true, "Minecraft 1.16.5: the red hurt flash and white creeper flash on mobs set up their overlay once instead of with 16 GL calls for every group of mobs drawn. Looks exactly the same. Turned off automatically next to OptiFine, Iris, Canvas or Sodium.");
    public final ConfigValue<Boolean> lightmapRate = ConfigValue.bool(
            "performance.lightmap_rate", true, "Minecraft 1.16.5: the light colour table (sky brightness at sunrise and sunset, torch flicker) is recalculated 10 times a second instead of 20. Lightning flashes still update every tick.");
    public final ConfigValue<Boolean> aoCache = ConfigValue.bool(
            "performance.ao_cache", true, "Minecraft 1.16.5, with Smooth lighting: the chunk builder remembers the light of the last 320 blocks instead of 100 while shading, so the row next to the one it just shaded is still in memory. Chunks build faster; the result is identical.");
    public final ConfigValue<Boolean> asyncSkins = ConfigValue.bool(
            "performance.async_skins", true, "Minecraft 1.16.5: player skins and capes that are already saved on your device are read and decoded on a background thread instead of the main thread, so players joining a server or coming into view no longer cause small freezes.");

    // 1.17.2 (Minecraft 1.16.5)
    public final ConfigValue<Boolean> lazySearch = ConfigValue.bool(
            "performance.lazy_search", true, "Minecraft 1.16.5: the search index of the creative inventory and the recipe book is built the first time you actually search, instead of while the game starts and every time you join a world. Faster start-up and world joins, less memory; the first search takes a moment longer.");
    public final ConfigValue<Integer> hudFps = ConfigValue.intRange(
            "performance.hud_fps", 30, 0, 60, false, "Minecraft 1.16.5: the HUD (hotbar, hearts, chat, F3 screen, boss bars) is redrawn this many times a second and reused in between, which saves hundreds of small draws per frame. The crosshair and the screen-edge shadow stay live, and switching slots or resizing redraws at once. Only used while the game runs clearly faster than this rate. 0 = redraw every frame (vanilla).");
    public final ConfigValue<Boolean> entityTickCulling = ConfigValue.bool(
            "performance.entity_tick_culling", true, "Minecraft 1.16.5: mobs, items and other entities that are out of view and more than 8 blocks away skip their client-side animation update; they still follow the server's position updates and catch up the moment they come into view. Players, vehicles, bosses, projectiles and anything you ride are always updated.");
    public final ConfigValue<Boolean> fastSectionSnapshots = ConfigValue.bool(
            "performance.fast_section_snapshots", true, "Minecraft 1.16.5: when a chunk section is handed to the chunk builders, its block and fluid copy reads every block once instead of twice. That copy runs on the main thread, so chunk updates cost less frame time.");
    public final ConfigValue<Boolean> fastNetwork = ConfigValue.bool(
            "performance.fast_network", true, "Minecraft 1.16.5: incoming packets are cut out of the network stream without being copied, and compressed packets are unpacked through reused buffers instead of new ones for each packet. Less garbage and less work while chunks stream in on servers.");
    public final ConfigValue<Boolean> multiDraw = ConfigValue.bool(
            "performance.multi_draw", true, "Minecraft 1.16.5, Aetherium pipeline with Batched terrain, computers only: all visible pieces of one terrain buffer are drawn with a single call instead of one call each. Not used through GL translators on phones, where that call is not available; those keep the separate draws.");
    public final ConfigValue<Boolean> batchedChests = ConfigValue.bool(
            "performance.batched_chests", true, "Minecraft 1.16.5: closed chests, trapped chests and ender chests become part of the chunk they stand in, so a storage room costs almost nothing to draw. A chest switches back to its animated form the moment it starts to open or is being mined, and back again once it has closed.");
    public final ConfigValue<Boolean> packedTerrain = ConfigValue.bool(
            "performance.packed_terrain", true, "Minecraft 1.16.5, Aetherium pipeline with Batched terrain: solid and cutout terrain is stored with 20 bytes per vertex instead of 32. A third less graphics memory and memory bandwidth for the world; positions stay accurate to 1/256 of a block. Turned off automatically after a crash while drawing it.");
    public final ConfigValue<Boolean> occlusionQueries = ConfigValue.bool(
            "performance.occlusion_queries", false, "Minecraft 1.16.5, Aetherium pipeline, computers only, experimental: the graphics card reports which chunk sections were completely hidden behind other terrain last frame, and those are not drawn until they could be visible again. Big gains in mountains and cities on strong graphics cards; a hidden section can appear one frame late when you turn fast, which is why this is off by default.");

    // ---------------------------------------------------------------- quality
    public final ConfigValue<Boolean> weather = ConfigValue.bool("quality.weather", true, "Render rain and snow. Off saves a lot of fill rate on mobile GPUs.");
    public final ConfigValue<Boolean> vignette = ConfigValue.bool("quality.vignette", true, "Draw the darkened screen edges.");

    public final ConfigValue<Boolean> distanceFog = ConfigValue.bool(
            "quality.fog.distance", true, "Fog that hides the edge of the render distance. Off shows terrain all the way to the edge (also removes the thick Nether fog). Fog under water, in lava and from blindness is unaffected.");
    public final ConfigValue<Integer> fogStart = ConfigValue.intRange(
            "quality.fog.start", 75, 0, 100, false, "Where the distance fog begins, in percent of the render distance. 75 = vanilla; higher keeps more of the world clear.");
    public final ConfigValue<Integer> waterFog = ConfigValue.intRange(
            "quality.fog.water", 100, 0, 100, false, "Density of the fog under water, in percent. 100 = vanilla, 0 = clear water.");
    public final ConfigValue<Boolean> clearLava = ConfigValue.bool(
            "quality.fog.clear_lava", false, "See a few blocks further while inside lava.");
    public final ConfigValue<Boolean> sky = ConfigValue.bool(
            "quality.sky.sky", true, "Draw the sky dome, sunrise colours, sun, moon and stars. Off leaves a flat sky colour (cheapest).");
    public final ConfigValue<Boolean> sunMoon = ConfigValue.bool(
            "quality.sky.sun_moon", true, "Draw the sun, the moon and the sunrise and sunset glow.");
    public final ConfigValue<Boolean> stars = ConfigValue.bool("quality.sky.stars", true, "Draw the stars at night.");
    public final ConfigValue<Integer> cloudHeight = ConfigValue.intRange(
            "quality.sky.cloud_height", 0, 0, 320, false, "Height of the cloud layer in blocks. 0 = the dimension's own height (128 in the Overworld).");
    public final ConfigValue<Boolean> particlesExplosions = ConfigValue.bool(
            "quality.particles.explosions", true, "Explosion, firework and flash particles.");
    public final ConfigValue<Boolean> particlesWeather = ConfigValue.bool(
            "quality.particles.weather", true, "Raindrop splashes on the ground and in water.");
    public final ConfigValue<Boolean> particlesSmoke = ConfigValue.bool(
            "quality.particles.smoke", true, "Smoke, campfire smoke, clouds and poof particles.");
    public final ConfigValue<Boolean> particlesPotions = ConfigValue.bool(
            "quality.particles.potions", true, "Potion swirls and splash effects, beacon and conduit effects.");
    public final ConfigValue<Boolean> particlesCombat = ConfigValue.bool(
            "quality.particles.combat", true, "Critical and enchanted hit stars, sweep attacks and damage hearts.");
    public final ConfigValue<Boolean> particlesFire = ConfigValue.bool(
            "quality.particles.fire", true, "Torch and fire flames, soul flames and lava pops.");
    public final ConfigValue<Boolean> particlesDrips = ConfigValue.bool(
            "quality.particles.drips", true, "Dripping water, lava, honey and tears; bubbles and underwater particles.");
    public final ConfigValue<Boolean> particlesPortals = ConfigValue.bool(
            "quality.particles.portals", true, "Nether portal, ender pearl and ender chest particles.");
    public final ConfigValue<Boolean> particlesAmbient = ConfigValue.bool(
            "quality.particles.ambient", true, "Floating ambient particles: Nether ash and spores, mycelium and the underwater haze.");
    public final ConfigValue<Boolean> particlesTotem = ConfigValue.bool(
            "quality.particles.totem", true, "The burst from a Totem of Undying.");
    public final ConfigValue<Boolean> particlesRedstone = ConfigValue.bool(
            "quality.particles.redstone", true, "The red dust from powered redstone wire, torches, repeaters, comparators, levers and redstone ore. Off helps a lot near big redstone builds.");
    public final ConfigValue<Boolean> particlesBlockBreak = ConfigValue.bool(
            "quality.particles.block_break", true, "Fragments when blocks are broken, and the chips while mining one.");

    public final ConfigValue<LeavesMode> leaves = ConfigValue.enumerated(
            "quality.leaves", LeavesMode.GRAPHICS, LeavesMode.class, false,
            "Minecraft 1.16.5: draw leaves fast (opaque) or fancy (see-through) independently of the Graphics setting. GRAPHICS follows the Graphics setting like vanilla.");
    public final ConfigValue<Boolean> leafCulling = ConfigValue.bool(
            "quality.leaf_culling", true, "Minecraft 1.16.5: leaf faces buried inside a tree are not drawn. With fancy leaves the outer two layers stay visible so trees keep their depth; with fast leaves every hidden face is skipped. Far fewer faces in forests.");
    public final ConfigValue<Integer> farTerrain = ConfigValue.intRange(
            "quality.far_terrain", 0, 0, 64, false, "Overworld, every version: shows a cheap low-detail outline of the terrain beyond your render distance, up to this many chunks away, from chunks you have already received this session (a height and colour map, not real blocks). Chunks inside your render distance that are not built yet show the outline too, so the world never looks empty while it loads. The distance fog moves out to the new edge. Costs memory and some GPU time; 0 = off. Limited to 4 times the render distance. Not available with shader packs or on the Vulkan backend (26.2+).");
    public final ConfigValue<ShaderPreset> shaderPreset = ConfigValue.enumerated(
            "quality.shader_preset", ShaderPreset.OFF, ShaderPreset.class, false,
            "Minecraft 1.16.5: a light colour filter over the world (not a full shader pack, so it costs one screen copy per frame and works on phones): Vibrant boosts colour and contrast, Warm adds a golden tone, Cinematic adds contrast and dark corners. Turns itself off if the graphics driver rejects it.");
    public final ConfigValue<Integer> cloudDistance = ConfigValue.intRange(
            "quality.cloud_distance", 100, 25, 100, false, "Minecraft 1.16.5: how far Fancy clouds reach, in percent of vanilla's cloud area (25 = only the clouds right around you). Smaller cloud areas cost less fill rate and fewer vertices, which matters on phones. 100 = vanilla; Fast clouds are not affected.");
    public final ConfigValue<Boolean> fastGrass = ConfigValue.bool(
            "quality.fast_grass", false, "Minecraft 1.16.5: the sides of grass blocks are drawn with one layer instead of two (the biome-coloured grass edge on top of the dirt is left out, the plain grass edge stays). Fewer faces in every chunk with grass; sides look slightly less green.");
    public final ConfigValue<Boolean> lightWeather = ConfigValue.bool(
            "quality.light_weather", false, "Minecraft 1.16.5: rain and snow are drawn in a smaller area around you (about half the radius), a large fill-rate saving on phones.");
    public final ConfigValue<Boolean> enchantGlint = ConfigValue.bool(
            "quality.enchant_glint", true, "Minecraft 1.16.5: the shimmer on enchanted items and armor. Off skips its extra rendering pass.");
    public final ConfigValue<Boolean> endCrystalBeams = ConfigValue.bool(
            "quality.end_crystal_beams", true, "Minecraft 1.16.5: the healing beams of End crystals (to the dragon and on the exit portal pillars).");

    // ---------------------------------------------------------------- effects
    public final ConfigValue<LightMode> dynamicLights = ConfigValue.enumerated(
            "effects.dynamic_lights", LightMode.OFF, LightMode.class, false,
            "Light emitted by held items and entities: hold a torch and the blocks around you light up. FAST updates 4x per second with a shorter radius; FANCY updates every tick. Takes effect immediately, no restart; Off costs nothing.");
    public final ConfigValue<Boolean> dynamicLightsHeld = ConfigValue.bool("effects.dynamic_lights.held", true, "Torches and other luminous items held by players and mobs emit light.");
    public final ConfigValue<Boolean> dynamicLightsEntities = ConfigValue.bool("effects.dynamic_lights.entities", true, "Dropped luminous items, burning entities and blazes emit light.");
    public final ConfigValue<Boolean> fullbright = ConfigValue.bool("effects.fullbright", false, "Override the brightness so caves and nights are fully lit.");
    public final ConfigValue<Integer> fullbrightStrength = ConfigValue.intRange(
            "effects.fullbright.strength", 100, 10, 100, false, "Fullbright strength in percent.");
    public final ConfigValue<Integer> zoomDefault = ConfigValue.intRange(
            "effects.zoom.default", 50, 0, 100, false, "How far the Zoom key zooms in when pressed, in percent. 50 is about four times, like OptiFine. Change it while zooming with the scroll wheel or the Zoom In / Zoom Out keys (rebind them in Options > Controls).");
    public final ConfigValue<Boolean> zoomToggle = ConfigValue.bool(
            "effects.zoom.toggle", false, "First person: press Zoom once to zoom and again to stop, instead of holding it.");
    public final ConfigValue<Boolean> zoomCinematic = ConfigValue.bool(
            "effects.zoom.cinematic", true, "Use the smooth cinematic camera while zoomed, for steady aiming at long range.");
    public final ConfigValue<Boolean> zoomScrolling = ConfigValue.bool(
            "effects.zoom.scrolling", true, "While zooming, the scroll wheel zooms in and out instead of changing the hotbar slot.");
    public final ConfigValue<Integer> zoomSpeed = ConfigValue.intRange(
            "effects.zoom.speed", 20, 1, 100, false, "How fast the scroll wheel and the Zoom In / Zoom Out keys change the zoom.");
    public final ConfigValue<Integer> zoomSmoothnessMs = ConfigValue.intRange(
            "effects.zoom.smoothness_ms", 150, 0, 500, false, "Zoom changes are animated over this many milliseconds. 0 = instant.");
    public final ConfigValue<Integer> zoomSensitivityFloor = ConfigValue.intRange(
            "effects.zoom.sensitivity_floor", 40, 0, 100, false, "Mouse sensitivity is lowered while zoomed, but never below this percentage of normal. 100 = unchanged.");
    public final ConfigValue<Integer> zoomMinFov = ConfigValue.intRange(
            "effects.zoom.min_fov", 1, 1, 30, false, "Field of view at full zoom, in degrees. Lower = stronger maximum zoom.");
    public final ConfigValue<Integer> zoomThirdPersonDistance = ConfigValue.intRange(
            "effects.zoom.third_person_distance", 15, 0, 32, false, "Third person: the zoom keys move the camera between 0.5 blocks and this distance instead of zooming. 0 = zoom the field of view in third person too.");
    public final ConfigValue<Boolean> zoomThirdPersonToggle = ConfigValue.bool(
            "effects.zoom.third_person_toggle", true, "Third person: the Zoom key toggles the camera distance instead of being held.");
    public final ConfigValue<Boolean> freelookToggle = ConfigValue.bool(
            "effects.freelook.toggle", false, "Press Freelook once to look around and again to stop, instead of holding it.");
    public final ConfigValue<Boolean> freelookThirdPerson = ConfigValue.bool(
            "effects.freelook.third_person", true, "Switch to the third-person camera while freelooking from first person, so you can see around your player. Off keeps the first-person view.");

    // ---------------------------------------------------------- quality of life
    public final ConfigValue<Boolean> accuratePlacement = ConfigValue.bool(
            "qol.accurate_placement", false, "While you hold Use and move, a block is placed as soon as the crosshair reaches a new spot instead of on vanilla's fixed timer, so held-down floors and walls come out without gaps or double blocks. Building onto the block you just placed keeps vanilla's speed, and blocks are never placed faster than 10 per second. Uses normal placement, so it works on any server.");
    public final ConfigValue<Boolean> scrollTransfer = ConfigValue.bool(
            "qol.inventory.scroll_transfer", false, "Mouse Tweaks wheel tweak: in inventories and chests, scroll over a stack. Scroll down pushes items to the other inventory (between the chest and your inventory), one per notch; scroll up pulls matching items into the stack. Over a crafting or furnace output, each notch moves one crafted batch. Done with normal inventory clicks, so it works on any server.");
    public final ConfigValue<Boolean> shiftDrag = ConfigValue.bool(
            "qol.inventory.shift_drag", false, "Mouse Tweaks drag tweaks. Right-drag with a stack: one item goes in every slot you pass over, again each time you come back. Left-drag with a stack over the same item: those stacks are picked up onto the mouse (with Shift: moved to the other inventory). Shift + left-drag with an empty mouse: every stack you pass over is shift-clicked. Done with normal inventory clicks, so it works on any server.");
    public final ConfigValue<Boolean> sortButton = ConfigValue.bool(
            "qol.inventory.sort_button", false, "A small sort button on chests, barrels, ender chests, shulker boxes, hoppers and dispensers, and one for your own inventory. Stacks of the same item are merged and everything is put in order, one move per tick with normal inventory clicks. The Sort key (unbound by default, Options > Controls) sorts the part of the inventory under the mouse. Strict anti-cheat servers may dislike fast inventory clicks.");
    public final ConfigValue<Boolean> foodTooltip = ConfigValue.bool(
            "qol.food.tooltip", false, "Food tooltips show how much hunger and saturation the food restores.");
    public final ConfigValue<Boolean> foodHud = ConfigValue.bool(
            "qol.food.hud", false, "Shows your hidden saturation as a gold tint over the hunger bar, and while you hold food, the hunger it would restore flashes on the bar.");
    public final ConfigValue<Boolean> waypoints = ConfigValue.bool(
            "qol.waypoints", false, "Show your waypoints in the world: a label with the name and distance, visible through blocks. Add one where you stand with the Add waypoint key (B), and manage them with the Waypoints key (N) or the button below. Kept for each world and server.");
    public final ConfigValue<Boolean> waypointBeams = ConfigValue.bool(
            "qol.waypoints.beams", true, "Draw a thin coloured beam of light at each waypoint, so it can be seen from far away.");
    public final ConfigValue<Boolean> waypointsCrossDimension = ConfigValue.bool(
            "qol.waypoints.cross_dimension", true, "In the Nether, show your Overworld waypoints at the matching Nether spot (divided by 8), and Nether waypoints in the Overworld (multiplied by 8), for building linked portals.");
    public final ConfigValue<Boolean> deathPoint = ConfigValue.bool(
            "qol.death_point", false, "When you die, the spot is saved as a red Death waypoint (replacing the previous one) and its coordinates are shown on the death screen.");
    public final ConfigValue<Boolean> dimensionCoords = ConfigValue.bool(
            "qol.dimension_coords", false, "Shows where you are in the other dimension: in the Overworld the matching Nether coordinates (x and z divided by 8), in the Nether the Overworld ones (multiplied by 8).");
    public final ConfigValue<String> dimensionCoordsCorner = ConfigValue.string(
            "qol.dimension_coords.corner", "top-left", "Screen corner of the dimension coordinates: top-left, top-right, bottom-left, bottom-right.");
    public final ConfigValue<Integer> fireOverlay = ConfigValue.intRange(
            "qol.fire_overlay", 100, 0, 100, false, "Height of the flames on your screen while you are burning, in percent. 100 = vanilla, 0 = hidden. Strict PvP servers may consider a hidden fire overlay unfair.");
    public final ConfigValue<Integer> shieldOffset = ConfigValue.intRange(
            "qol.shield_offset", 0, 0, 100, false, "How far a raised shield is lowered in first person, in percent, so it covers less of the screen while blocking. 0 = vanilla.");
    public final ConfigValue<Boolean> damageNumbers = ConfigValue.bool(
            "qol.damage_numbers", false, "Floating numbers show how much health a mob or player near you lost. Uses the health the server already sends to every client.");
    public final ConfigValue<Boolean> damageNumbersHeals = ConfigValue.bool(
            "qol.damage_numbers.heals", false, "Also show green numbers when a mob or player near you heals.");
    public final ConfigValue<CrosshairStyle> crosshairStyle = ConfigValue.enumerated(
            "qol.crosshair.style", CrosshairStyle.VANILLA, CrosshairStyle.class, false,
            "Shape of the crosshair. VANILLA keeps Minecraft's own; the others are drawn by Aetherium in the colour below.");
    public final ConfigValue<CrosshairColor> crosshairColor = ConfigValue.enumerated(
            "qol.crosshair.color", CrosshairColor.WHITE, CrosshairColor.class, false,
            "Colour of a custom crosshair.");
    public final ConfigValue<Integer> crosshairSize = ConfigValue.intRange(
            "qol.crosshair.size", 4, 1, 15, false, "Length of the crosshair arms, or the radius of the circle, in GUI pixels.");
    public final ConfigValue<Integer> crosshairThickness = ConfigValue.intRange(
            "qol.crosshair.thickness", 1, 1, 4, false, "Line width of a custom crosshair, in GUI pixels.");
    public final ConfigValue<Integer> crosshairGap = ConfigValue.intRange(
            "qol.crosshair.gap", 2, 0, 8, false, "Empty space between the centre and the arms of a custom crosshair, in GUI pixels.");
    public final ConfigValue<Integer> crosshairOpacity = ConfigValue.intRange(
            "qol.crosshair.opacity", 100, 10, 100, false, "Opacity of a custom crosshair, in percent.");
    public final ConfigValue<Boolean> crosshairOutline = ConfigValue.bool(
            "qol.crosshair.outline", true, "A thin black outline around a custom crosshair, so it stays visible on bright and dark backgrounds.");
    public final ConfigValue<Boolean> crosshairTargetColor = ConfigValue.bool(
            "qol.crosshair.target_color", false, "A custom crosshair turns red while you aim at a mob or player within reach.");
    public final ConfigValue<HandPreset> handPreset = ConfigValue.enumerated(
            "qol.hand.preset", HandPreset.CUSTOM, HandPreset.class, false,
            "Last hand position preset; editing any hand value turns it back into CUSTOM.");
    public final ConfigValue<Boolean> handEditor = ConfigValue.bool(
            "qol.hand.enabled", false, "Move, turn and resize your hand and the items you hold in first person with the values below. Only changes your own view.");
    public final ConfigValue<Integer> handX = ConfigValue.intRange(
            "qol.hand.x", 0, -200, 200, false, "Sideways position: positive moves the hand outwards, negative towards the centre of the screen. 10 = one tenth of a block.");
    public final ConfigValue<Integer> handY = ConfigValue.intRange(
            "qol.hand.y", 0, -200, 200, false, "Height: positive moves the hand up, negative down. 10 = one tenth of a block.");
    public final ConfigValue<Integer> handZ = ConfigValue.intRange(
            "qol.hand.z", 0, -200, 200, false, "Depth: positive moves the hand towards you, negative away from you. 10 = one tenth of a block.");
    public final ConfigValue<Integer> handPitch = ConfigValue.intRange(
            "qol.hand.pitch", 0, -180, 180, false, "Tilt forwards or backwards, in degrees.");
    public final ConfigValue<Integer> handYaw = ConfigValue.intRange(
            "qol.hand.yaw", 0, -180, 180, false, "Turn left or right, in degrees.");
    public final ConfigValue<Integer> handRoll = ConfigValue.intRange(
            "qol.hand.roll", 0, -180, 180, false, "Roll sideways, in degrees.");
    public final ConfigValue<Integer> handScale = ConfigValue.intRange(
            "qol.hand.scale", 100, 10, 300, false, "Size of the hand and held item, in percent. 100 = vanilla.");
    public final ConfigValue<Boolean> handVisible = ConfigValue.bool(
            "qol.hand.visible", false, "Draws your arm holding whatever you carry in first person, like Punchy and Hold My Items (vanilla only shows the arm when the hand is empty). The arm follows every swing, eat and block animation and moves with the hand editor values.");
    public final ConfigValue<Boolean> handMirror = ConfigValue.bool(
            "qol.hand.mirror", true, "Use the same values for the off hand, mirrored, so both hands match. Off leaves the off hand as in vanilla.");
    public final ConfigValue<Boolean> armBoth = ConfigValue.bool(
            "qol.arm.both", false, "Visible hands at all times, like Punchy: your off hand is drawn even when it is empty, so you always see both hands (vanilla hides an empty off hand).");
    public final ConfigValue<Integer> armX = ConfigValue.intRange(
            "qol.arm.x", 0, -200, 200, false, "Moves both arms sideways together, with or without an item, like Punchy's arm sculpt: positive spreads them apart, negative brings them towards the middle. 100 = one block.");
    public final ConfigValue<Integer> armY = ConfigValue.intRange(
            "qol.arm.y", 0, -200, 200, false, "Moves both arms up or down together, with or without an item. 100 = one block.");
    public final ConfigValue<Integer> armZ = ConfigValue.intRange(
            "qol.arm.z", 0, -200, 200, false, "Moves both arms farther away (positive) or closer to you (negative) together, with or without an item. 100 = one block.");
    public final ConfigValue<Integer> armScale = ConfigValue.intRange(
            "qol.arm.scale", 100, 10, 300, false, "Size of both arms and what they hold, in percent. 100 = vanilla.");

    // ----------------------------------------------------------------- android
    public final ConfigValue<Boolean> androidSupport = ConfigValue.bool("android.enabled", true, "Enable launcher/renderer probing. No-op on desktop.");
    public final ConfigValue<AndroidRendererChoice> androidForceRenderer = ConfigValue.enumerated(
            "android.force_renderer", AndroidRendererChoice.AUTO, AndroidRendererChoice.class, true,
            "Override the detected renderer; use when the launcher env is unreadable.");
    public final ConfigValue<Boolean> readCustomEnv = ConfigValue.bool("android.read_custom_env", true, "Parse the launcher's custom_env.txt for renderer hints.");
    public final ConfigValue<Integer> memoryBudgetMb = ConfigValue.intRange("android.memory_budget_mb", 1024, 256, 8192, false, "Heap budget reported for the device; auto-detected from cgroup limits.");
    public final ConfigValue<Boolean> batterySaver = ConfigValue.bool("android.battery_saver", false, "Cap the frame rate to save battery.");
    public final ConfigValue<Integer> batteryFpsCap = ConfigValue.intRange("android.battery_fps_cap", 30, 15, 60, false, "Frame-rate cap used by battery saver.");
    public final ConfigValue<Boolean> thermalGuard = ConfigValue.bool("android.thermal_guard", false, "Android only, opt-in: cap the frame rate at 30 while the phone is above the thermal ceiling. The temperature is read on a background thread.");
    public final ConfigValue<Integer> thermalCeilingC = ConfigValue.intRange("android.thermal_ceiling_c", 75, 40, 95, false, "Hottest CPU/GPU temperature at which the thermal guard engages.");
    public final ConfigValue<Boolean> touchMode = ConfigValue.bool("android.touch_mode", false, "Taller rows and larger hit targets in the Aetherium screen.");

    // ---------------------------------------------------------------- advanced
    public final ConfigValue<Boolean> debugLogging = ConfigValue.bool("advanced.debug_logging", false, "Enable dev-level logging (hot-path cost: near zero when off).");
    public final ConfigValue<Boolean> stutterLog = ConfigValue.bool(
            "advanced.stutter_log", true, "Write every frame that takes far longer than usual to logs/aetherium-stutter.log, with what happened in that frame (packets, chunk uploads, builds, garbage collection). Written on a background thread; at most 1 MB.");
    public final ConfigValue<Boolean> aetheriumRenderer = ConfigValue.bool(
            "advanced.aetherium_renderer", true, "Aetherium's own terrain renderer (every version, 1.16.5 to 26.3). Solid and cutout chunks are grouped by face direction, and each chunk section is drawn as one range that leaves out the face groups pointing away from you, never with more draw calls than vanilla (on 26.3 a section facing fully away is still drawn). On 1.16.5 neighbouring sections also share GPU buffers when Batched terrain is on. Translucent blocks stay vanilla. Switching it on rebuilds the loaded chunks once (1.21.6 and newer: chunks pick it up as they rebuild); it turns itself off and falls back to vanilla if anything fails, and stays off after a crash while it was drawing.");
    public final ConfigValue<Boolean> regionBatching = ConfigValue.bool(
            "advanced.region_batching", true, "With the Aetherium pipeline on: store the terrain of every 8 x 4 x 8 chunk sections in a few shared GPU buffers and draw each buffer with one setup instead of setting up every chunk section on its own; neighbouring pieces are joined into one draw call. Same vertex format and GL calls as vanilla. Turns itself off at the next start if the game crashed while drawing with it.");
    public final ConfigValue<Boolean> conflictAutoDelegate = ConfigValue.bool("advanced.conflict_auto_delegate", true, "Hand features back to a conflicting mod instead of double-applying them.");

    // ------------------------------------------------------------------- lookup
    /** Declaration-order index, keyed by {@code group.name}. */
    private final Map<String, ConfigValue<?>> options = new LinkedHashMap<>();

    private int fileVersion = CURRENT_VERSION;

    public AetheriumConfig() {
        for (final java.lang.reflect.Field field : this.getClass().getFields()) {
            if (!ConfigValue.class.isAssignableFrom(field.getType())) {
                continue;
            }
            try {
                final ConfigValue<?> value = (ConfigValue<?>) field.get(this);
                this.options.put(value.getKey(), value);
            } catch (final IllegalAccessException error) {
                // Public fields cannot throw here; surfacing it beats hiding it.
                throw new IllegalStateException("Failed to index config field " + field.getName(), error);
            }
        }
    }

    public static AetheriumConfig createDefaults() {
        return new AetheriumConfig();
    }

    public Map<String, ConfigValue<?>> all() {
        return this.options;
    }

    public Collection<ConfigValue<?>> values() {
        return this.options.values();
    }

    public ConfigValue<?> byKey(final String key) {
        return this.options.get(Objects.requireNonNull(key, "key"));
    }

    public Map<String, ConfigValue<?>> byGroup(final String group) {
        final Map<String, ConfigValue<?>> out = new LinkedHashMap<>();
        for (final Map.Entry<String, ConfigValue<?>> entry : this.options.entrySet()) {
            final String key = entry.getKey();
            if (key.startsWith(group + ".")) {
                out.put(key, entry.getValue());
            }
        }
        return out;
    }

    public int getFileVersion() {
        return this.fileVersion;
    }

    public void setFileVersion(final int version) {
        this.fileVersion = version;
    }

    public boolean anyDirty() {
        for (final ConfigValue<?> value : this.options.values()) {
            if (value.isDirty()) {
                return true;
            }
        }
        return false;
    }

    public void clearDirty() {
        for (final ConfigValue<?> value : this.options.values()) {
            // Direct flag reset: set(get()) never clears the flag, so a write that
            // "succeeded" would leave every value dirty and the shutdown hook would
            // rewrite the file for no reason.
            value.clearDirtyFlag();
        }
    }

    /** Performance presets offered on the General tab. */
    public enum Preset {
        CUSTOM, MAX_FPS, BALANCED, QUALITY
    }

    /** Leaves drawing mode (1.10.0). */
    public enum LeavesMode {
        GRAPHICS, FAST, FANCY
    }

    /** Colour filter presets (1.11.0). */
    public enum ShaderPreset {
        OFF, VIBRANT, WARM, CINEMATIC
    }

    /** Custom crosshair shapes (Quality of life); same order as {@code qol.Crosshair.Style}. */
    public enum CrosshairStyle {
        VANILLA, CROSS, CROSS_DOT, DOT, CIRCLE, CIRCLE_DOT, T_SHAPE
    }

    /** Custom crosshair colours (Quality of life); same order as {@code qol.Crosshair.Color}. */
    public enum CrosshairColor {
        WHITE, RED, GREEN, CYAN, YELLOW, MAGENTA, BLACK
    }

    /** Hand position presets (Quality of life, hand editor). */
    public enum HandPreset {
        CUSTOM, DEFAULT, LOWERED, CENTERED, SMALL, BIG
    }

    /** Dynamic light update modes. */
    public enum LightMode {
        OFF, FAST, FANCY
    }

    /** Manual override for the Android renderer when env probing is not enough. */
    public enum AndroidRendererChoice {
        AUTO,
        GL4ES,
        ZINK,
        LTW,
        MOBILEGLUES,
        VIRGL,
        ANGLE,
        NATIVE_VULKAN
    }

    @Override
    public String toString() {
        return "AetheriumConfig{version=" + this.fileVersion + ", options=" + this.options.size() + '}';
    }
}
