# Aetherium port: Minecraft 26.2

- Java toolchain: **25** (mixin compatibilityLevel `JAVA_25`)
- Fabric loader floor: **0.19.5**, Loom **1.16.1**
- Fabric API (compile-only, optional at runtime): **0.161.0+26.2**
- NeoForge: **26.2.0.89**
- Mappings: officialMojangMappings()
- Status: **derived**

## What the delta changes

### Build pins

```properties
minecraft_version=26.2
java_version=25
fabric_loader_version=0.19.5
fabric_api_version=0.161.0+26.2
neoforge_version=26.2.0.89
```

### Era transforms applied by the generated patch

- `anim=none`
- `atlas-tick=none`
- `background=extract`
- `be-render=extract-flag`
- `bg-threads=property`
- `block-particles=level`
- `boost16=none`
- `cam-pos=position`
- `camera-entity=record`
- `camera-get=record`
- `chat-send=system`
- `chat=literal`
- `chunk-renderer=none`
- `chunk-upload=none`
- `clouds=renderer`
- `cull-new=uber262`
- `debug-screen=overlay`
- `dirty=sections`
- `fog-cull-x=extract`
- `fog-cull=none`
- `fog=data`
- `freelook=align`
- `frustum-cap=none`
- `graphics=preset`
- `gui=extractor`
- `hud=none`
- `input=events`
- `keys=category-id`
- `layer-sig=none`
- `level-extract=x262`
- `light-hook=none`
- `loom-plugin-id->no-remap`
- `mappings-line-removed`
- `mesh-alloc=none`
- `mesh-exec=tracing`
- `modCompileOnly->compileOnly`
- `modImplementation->implementation`
- `model-fast=none`
- `near-build=none`
- `occlusion-box=renderer`
- `options=instances`
- `pacing=limiter`
- `particle-fast=none`
- `q-fire=f262`
- `q-hand=s262`
- `q-hud=x262`
- `q-inv=c261`
- `q-world=hud`
- `qol16=none`
- `registry=builtin`
- `reload=invalidate`
- `remapJar-block-removed`
- `render-scale=g262`
- `res-key=identifier`
- `rs-api=g262`
- `screen-owner=gui`
- `screen-pkg=options`
- `scroll=four`
- `search-throttle=none`
- `section-api=none`
- `section-compile=none`
- `section-cull=none`
- `section-vis=render-origin`
- `simdist=sim`
- `sky=renderer`
- `slot-find=hovered`
- `t-ao-cache=none`
- `t-autosave=s214`
- `t-beam=b2111`
- `t-block-entity=b219`
- `t-build-order=none`
- `t-chunk-save=c26`
- `t-client=v262`
- `t-clouds=k261`
- `t-color-filter=none`
- `t-compile-queue=none`
- `t-crowd=c17`
- `t-cull=c215`
- `t-damage-tint=none`
- `t-drops=s214`
- `t-explosion=x212`
- `t-far-acc=none`
- `t-far-color=map`
- `t-far-draw=c26`
- `t-far-gl=gl262`
- `t-far-group=g216`
- `t-far-proj=b26`
- `t-far-sections=visible`
- `t-far-terrain=gl`
- `t-flight-preload=f216`
- `t-frame-item=f212`
- `t-gl-state=none`
- `t-glint=none`
- `t-grass-quads=none`
- `t-grass=none`
- `t-http-textureacc=none`
- `t-layers=l219`
- `t-lazy-data-fixers=none`
- `t-leaf-cull=none`
- `t-leaves=none`
- `t-level=none`
- `t-levelacc=none`
- `t-light=v16`
- `t-lightmap=none`
- `t-loading=none`
- `t-map=m212`
- `t-minecart=m219`
- `t-name-tag=n212`
- `t-ocean-floor=o17`
- `t-portal=p215`
- `t-raw-frame=none`
- `t-server-idle=v16`
- `t-server-light=none`
- `t-shadow=s219`
- `t-sign=s219`
- `t-skin=none`
- `t-skull-lookup=none`
- `t-skull=none`
- `t-sky=none`
- `t-sort=s212`
- `t-sound=r2111`
- `t-tex-sprite=f261`
- `t-unpack-make=f219`
- `t-unpack-sec=f261`
- `t-view-areaacc=none`
- `t-weather=w2111`
- `t-world-cache=none`
- `t-world=none`
- `vignette=none`
- `weather=effect`
- `zoom-cam=align`
- `zoom-fov=camera`
- `zoom=on`

### Where the per-version Java comes from

Every Java difference is an era block in the source (`// @era:<name>-begin ...`), selected
by `tools/eras.py`; the variant names above say which shape this row compiles.
`python3 tools/stubcheck.py 26.2` compiles exactly this selection against the
Minecraft signatures CI extracted from this version's jar (`tools/probe/26.2.txt`).

### Version facts from earlier porting research (historical; tools/eras.py is authoritative)

- `options_access` = **getters** - Options#renderDistance()/simulationDistance() OptionInstance getters are 1.19+; 1.17-1.18.2 read public int fields, and 1.16.5-1.17.x have no simulationDistance at all (the benchmark label degrades to `sim=n/a`) [VERIFIED: the 1.19/1.19.1/1.19.2 legs compiled the getters while 1.18.2 rejected them]
- `button_builder` = **True** - Button.builder arrives with the 1.19.3 screen rework; 1.16.5-1.19.2 construct Button directly [VERIFIED: the 1.19.3 leg rejected the constructor form]
- `cycle_button` = **CycleButton** - CycleButton under that exact mojmap name from 1.17 on [VERIFIED: official-mapping javadoc @ 1.17.1/1.18.2; the 1.17/1.18.2 legs rejected CycleButtonWidget]
- `logging` = **slf4j** - slf4j on the compile classpath
- `no_remap_loom` = **True** - 26.x ships unobfuscated jars: the no-remap Loom plugin id, no mappings() line, no remapJar task [VERIFIED: sodium @ 26.2/stable applies net.fabricmc.fabric-loom @ 1.16.1 with no mappings block; the 26.x legs failed with 'Failed to find official mojang mappings']
- `options_pkg` = **screens.options** - the screens.options package is 1.21+ [VERIFIED: 1.20.5/1.20.6 legs rejected it; sodium @ 1.21.1 imports it]
- `resource_location` = **parse** - ResourceLocation.parse is 1.21+; earlier rows use the constructor
- `level_sections` = **minSectionY** - getMinSection/getMaxSection through 1.21.1, getMinSectionY/getMaxSectionY from 1.21.2 [VERIFIED: ship legs + sodium @ 1.21.4]
- `input_events` = **event** - 1.21.9 replaced the double-based mouse handlers with MouseButtonEvent forms [VERIFIED: sodium @ 26.2/stable]
- `identifier` = **True** - ResourceLocation becomes Identifier (with fromNamespaceAndPath) from 1.21.11 [VERIFIED: sodium @ 1.21.11/stable; the 1.21.9/1.21.10 legs compiled ResourceLocation.parse fine]
- `gui_extractor` = **True** - 26.x replaces GuiGraphics with GuiGraphicsExtractor, Screen#render with extractRenderState, and drawString/drawCenteredString with text/centeredText [VERIFIED: sodium @ 26.1.2/stable and 26.2/stable; this row's exact version derived from 26.2 if newer]
- `entities_iter` = **entitiesForRendering** - entitiesForRendering [VERIFIED on 1.21.1: Iris MixinLevelRenderer_SkipRendering targets ClientLevel#entitiesForRendering]
- `fabric_api` pin read from FabricMC/fabric tags on 2026-10-10 [VERIFIED tag]

## Files the generated patch touches

- `gradle.properties`
- `gradle/libs.versions.toml`
- `common/src/main/resources/aetherium-common.mixins.json`
- `common/build.gradle.kts`
- `fabric/build.gradle.kts`
- `common/src/main/java/com/aetherium/Capabilities.java`
- `common/src/main/java/com/aetherium/client/AetheriumScreen.java`
- `common/src/main/java/com/aetherium/client/ChunkUnpacking.java`
- `common/src/main/java/com/aetherium/client/ClientHooks.java`
- `common/src/main/java/com/aetherium/client/CoalesceHooks.java`
- `common/src/main/java/com/aetherium/client/ColorFilter.java`
- `common/src/main/java/com/aetherium/client/FarTerrainBridge.java`
- `common/src/main/java/com/aetherium/client/FlightPreload.java`
- `common/src/main/java/com/aetherium/client/FogCullHooks.java`
- `common/src/main/java/com/aetherium/client/GrassQuads.java`
- `common/src/main/java/com/aetherium/client/McCanvas.java`
- `common/src/main/java/com/aetherium/client/QolClient.java`
- `common/src/main/java/com/aetherium/client/QolHudDraw.java`
- `common/src/main/java/com/aetherium/client/QolInventory.java`
- `common/src/main/java/com/aetherium/client/RawFrame.java`
- `common/src/main/java/com/aetherium/client/ScaledRenderTarget.java`
- `common/src/main/java/com/aetherium/client/SceneryHooks.java`
- `common/src/main/java/com/aetherium/client/TuningClient.java`
- `common/src/main/java/com/aetherium/client/VanillaOptions.java`
- `common/src/main/java/com/aetherium/client/WaypointScreen.java`
- `common/src/main/java/com/aetherium/client/WorldCache.java`
- `common/src/main/java/com/aetherium/client/ZoomKeys.java`
- `common/src/main/java/com/aetherium/lighting/DynamicLightTracker.java`
- `common/src/main/java/com/aetherium/mixin/core/AoFaceAllocMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/BakedQuadAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/BlockEntityCullMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/BlockParticleMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/BlockSpriteUseMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ChunkUploadMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CloudHeightMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CoalesceDirtyStateMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CoalesceExtractMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CoalesceMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CompiledChunkSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/EntityOcclusionMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FastParticleMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FogControlMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FogCullExtractMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FogCullMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FramePacingMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FreelookCameraMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FrustumArmMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FrustumCaptureMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/GuiMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ItemSpriteUseMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/KeyMappingAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/LightLevelMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/MainTargetAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/MeshAllocMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/MeshExecutorMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolContainerScreenMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolDeathScreenMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolFireMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolGuiMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolHandMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolHudMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/RenderChunkDirtyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ScreenSwapMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullBufferMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullCompiledMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullDrawMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullLayerMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullOwnerMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullSortMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionOcclusionGraphAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SkyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SpriteActivityMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TextureAtlasMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningAoCacheMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningAutosaveMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningBeamMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningBlockEntityMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningBuildOrderMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningChunkSaveMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningCloudsMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningCullMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningDropsMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningExplosionMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarDrawMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarGroupMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarProjMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFrameItemMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningGlStateMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningGlintMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningGrassMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningHttpTextureAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLayersMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLeafCullMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLeavesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLevelMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLightmapMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLoadingMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningMapMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningMinecartMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningNameTagMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningPortalMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningShadowMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSignMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSkinMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSkyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSortMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSoundMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningTextureSpriteMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningUnpackSectionMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningWeatherMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningWorldMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/VisibleSectionsMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/VisibleSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/WeatherMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ZoomCameraMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ZoomFovMixin.java`
- `common/src/main/java/com/aetherium/perf/TuningSupport.java`
- `common/src/main/java/com/aetherium/qol/QolSupport.java`

## Verifying this row before shipping it

The port is applied with `sh tools/port.sh 26.2` and then checked against the
real mapped jar - the jar is the authority, not a wiki:

```sh
MC=26.2
JAR=$(find "$HOME/.gradle/caches/fabric-loom" -name "minecraft-*-"$MC"-*.jar" 2>/dev/null | head -n 1)
test -n "$JAR" || { echo "run ./gradlew build once so Loom downloads $MC"; exit 1; }
javap -p -classpath "$JAR" net.minecraft.client.renderer.LightTexture | grep -E 'updateLightTexture|tickLightTexture|pixels|NativeImage|DynamicTexture'
javap -p -classpath "$JAR" net.minecraft.client.gui.Gui | grep -E 'public void render'
javap -p -classpath "$JAR" net.minecraft.client.renderer.LevelRenderer | grep -E 'setSectionDirty|allChanged'
javap -p -classpath "$JAR" net.minecraft.client.gui.screens.options.OptionsScreen | grep -nE 'lambda|init'
javap -p -classpath "$JAR" net.minecraft.client.gui.components.AbstractWidget | grep -E 'renderWidget|renderButton|setX|updateNarration'
```

```sh
python3 tools/check_refs.py . && python3 tools/check.py --skip-deltas
./gradlew --no-daemon :common:compileJava :fabric:compileJava
```

A compile error on the GUI overrides is the *good* outcome: it is loud, local and
mechanical. A mixin that silently stops applying is the bad one, which is why the
HUD prints `Aetherium.describeRuntime()` and the log prints every refused mixin.

## Notes

- Real: NeoForge ships 26.x branches, and Iris/LambDynamicLights build for it. `minecraft_version` in this repo's pin table is a date-based id, so `nextMinor()` in neoforge/build.gradle.kts handles it (26.3 -> 26.4); verify the range on first build.

Generated by `tools/gen_deltas.py`; re-run that instead of editing.
