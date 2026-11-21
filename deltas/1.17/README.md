# Aetherium port: Minecraft 1.17

- Java toolchain: **16** (mixin compatibilityLevel `JAVA_16`)
- Fabric loader floor: **0.14.24**, Loom **1.16.1**
- Fabric API (compile-only, optional at runtime): **0.46.1+1.17**
- NeoForge: **does not exist for this version - the neoforge module is disabled**
- Mappings: officialMojangMappings()
- Status: **unverified**

## What the delta changes

### Build pins

```properties
minecraft_version=1.17
java_version=16
fabric_loader_version=0.14.24
fabric_api_version=0.46.1+1.17
neoforge_version=unavailable (module disabled)
```

### Era transforms applied by the generated patch

- `anim=on`
- `atlas-tick=tickable`
- `background=none`
- `bg-threads=clamp`
- `boost16=none`
- `cam-pos=getter`
- `camera-entity=getter`
- `camera-get=getter`
- `chat-send=display`
- `chat=text`
- `chunk-renderer=none`
- `chunk-upload=crdbool`
- `cull-new=old`
- `debug-screen=field`
- `fog-cull-x=none`
- `fog-cull=bfs`
- `fog=shader17`
- `food-api=item`
- `frustum-cap=mojang`
- `graphics=status`
- `gui=stack`
- `hud=stack`
- `input=doubles`
- `layer-sig=mojang`
- `level-extract=none`
- `light-hook=color`
- `logger=log4j`
- `math=mojang`
- `mesh-alloc=v16`
- `mesh-exec=crd`
- `model-fast=mojang`
- `near-build=v16`
- `options=fields`
- `particle-fast=quat`
- `pose-normals=plain`
- `q-hand=t17`
- `q-hud=s17`
- `q-inv=t17`
- `q-place=material`
- `qol16=none`
- `registry=static`
- `reload=all-changed`
- `render-scale=r17`
- `res-key=location`
- `screen-owner=minecraft`
- `screen-pkg=flat`
- `scroll=three`
- `search-throttle=bfs`
- `section-api=crd`
- `section-compile=param17`
- `section-cull=pop17`
- `section-vis=list17`
- `simdist=none`
- `sky=end17`
- `slot-find=find`
- `sound-click=event`
- `sprite-name=getname`
- `sprite-u=d`
- `t-ao-cache=v16`
- `t-autosave=v16`
- `t-beam=v16`
- `t-block-entity=b17`
- `t-build-order=b17`
- `t-chunk-save=v16`
- `t-client=v16`
- `t-clouds=none`
- `t-color-filter=r17`
- `t-compile-queue=none`
- `t-crowd=c17`
- `t-cull=c17`
- `t-damage-tint=none`
- `t-drops=v16`
- `t-explosion=v16`
- `t-far-acc=v17`
- `t-far-color=material`
- `t-far-draw=m17`
- `t-far-gl=id17`
- `t-far-group=none`
- `t-far-proj=none`
- `t-far-sections=acc17`
- `t-far-terrain=gl`
- `t-flight-preload=v16`
- `t-frame-item=f17`
- `t-gl-state=g17`
- `t-glint=v16`
- `t-grass-quads=v16`
- `t-grass=v16`
- `t-http-textureacc=v16`
- `t-layers=v16`
- `t-lazy-data-fixers=v16`
- `t-leaf-cull=f17`
- `t-leaves=v16`
- `t-level=l17`
- `t-levelacc=none`
- `t-light=v16`
- `t-lightmap=v16`
- `t-loading=v16`
- `t-map=v16`
- `t-minecart=v16`
- `t-name-tag=n17`
- `t-ocean-floor=o17`
- `t-portal=v16`
- `t-server-idle=v16`
- `t-server-light=v16`
- `t-shadow=s17`
- `t-sign=v16`
- `t-skin=v16`
- `t-skull-lookup=none`
- `t-skull=none`
- `t-sky=e17`
- `t-sort=none`
- `t-tex-atlas=v16`
- `t-tex-sprite=s17`
- `t-unpack-make=v16`
- `t-unpack-net=v16`
- `t-unpack-sec=v16`
- `t-view-areaacc=none`
- `t-weather=none`
- `t-world-cache=r17`
- `t-world=v16`
- `vignette=render`
- `zoom-cam=double`
- `zoom=on`

### Where the per-version Java comes from

Every Java difference is an era block in the source (`// @era:<name>-begin ...`), selected
by `tools/eras.py`; the variant names above say which shape this row compiles.
`python3 tools/stubcheck.py 1.17` compiles exactly this selection against the
Minecraft signatures CI extracted from this version's jar (`tools/probe/1.17.txt`).

### Version facts from earlier porting research (historical; tools/eras.py is authoritative)

- `options_access` = **fields_no_sim** - Options#renderDistance()/simulationDistance() OptionInstance getters are 1.19+; 1.17-1.18.2 read public int fields, and 1.16.5-1.17.x have no simulationDistance at all (the benchmark label degrades to `sim=n/a`) [VERIFIED: the 1.19/1.19.1/1.19.2 legs compiled the getters while 1.18.2 rejected them]
- `button_builder` = **False** - Button.builder arrives with the 1.19.3 screen rework; 1.16.5-1.19.2 construct Button directly [VERIFIED: the 1.19.3 leg rejected the constructor form]
- `cycle_button` = **CycleButton** - CycleButton under that exact mojmap name from 1.17 on [VERIFIED: official-mapping javadoc @ 1.17.1/1.18.2; the 1.17/1.18.2 legs rejected CycleButtonWidget]
- `logging` = **slf4j** - slf4j on the compile classpath
- `no_remap_loom` = **False** - 26.x ships unobfuscated jars: the no-remap Loom plugin id, no mappings() line, no remapJar task [VERIFIED: sodium @ 26.2/stable applies net.fabricmc.fabric-loom @ 1.16.1 with no mappings block; the 26.x legs failed with 'Failed to find official mojang mappings']
- `options_pkg` = **screens** - the screens.options package is 1.21+ [VERIFIED: 1.20.5/1.20.6 legs rejected it; sodium @ 1.21.1 imports it]
- `resource_location` = **ctor** - ResourceLocation.parse is 1.21+; earlier rows use the constructor
- `level_sections` = **minSection** - getMinSection/getMaxSection through 1.21.1, getMinSectionY/getMaxSectionY from 1.21.2 [VERIFIED: ship legs + sodium @ 1.21.4]
- `input_events` = **doubles** - 1.21.9 replaced the double-based mouse handlers with MouseButtonEvent forms [VERIFIED: sodium @ 26.2/stable]
- `identifier` = **False** - ResourceLocation becomes Identifier (with fromNamespaceAndPath) from 1.21.11 [VERIFIED: sodium @ 1.21.11/stable; the 1.21.9/1.21.10 legs compiled ResourceLocation.parse fine]
- `gui_extractor` = **False** - 26.x replaces GuiGraphics with GuiGraphicsExtractor, Screen#render with extractRenderState, and drawString/drawCenteredString with text/centeredText [VERIFIED: sodium @ 26.1.2/stable and 26.2/stable; this row's exact version derived from 26.2 if newer]
- `entities_iter` = **entitiesForRendering** - entitiesForRendering [VERIFIED on 1.21.1: Iris MixinLevelRenderer_SkipRendering targets ClientLevel#entitiesForRendering]
- `fabric_api` pin read from FabricMC/fabric tags on 2026-10-10 [VERIFIED tag]

## Files the generated patch touches

- `gradle.properties`
- `settings.gradle.kts`
- `build.gradle.kts`
- `common/src/main/resources/aetherium-common.mixins.json`
- `common/src/main/java/com/aetherium/Capabilities.java`
- `common/src/main/java/com/aetherium/client/AetheriumScreen.java`
- `common/src/main/java/com/aetherium/client/ChunkUnpacking.java`
- `common/src/main/java/com/aetherium/client/ClientHooks.java`
- `common/src/main/java/com/aetherium/client/FarTerrainBridge.java`
- `common/src/main/java/com/aetherium/client/FlightPreload.java`
- `common/src/main/java/com/aetherium/client/McCanvas.java`
- `common/src/main/java/com/aetherium/client/QolClient.java`
- `common/src/main/java/com/aetherium/client/QolFoodInfo.java`
- `common/src/main/java/com/aetherium/client/QolHandPose.java`
- `common/src/main/java/com/aetherium/client/QolHudDraw.java`
- `common/src/main/java/com/aetherium/client/QolInventory.java`
- `common/src/main/java/com/aetherium/client/SceneryHooks.java`
- `common/src/main/java/com/aetherium/client/UiSounds.java`
- `common/src/main/java/com/aetherium/client/VanillaOptions.java`
- `common/src/main/java/com/aetherium/client/WaypointScreen.java`
- `common/src/main/java/com/aetherium/mixin/core/BufferBuilderAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/ChunkUploadMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CoalesceMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/CompiledChunkSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/DefaultedVertexAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/FastModelMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FastParticleMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FogControlMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FogCullMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FrustumArmMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FrustumCaptureMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/GuiMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/LazyDataFixersMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/Matrix3fAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/MeshExecutorMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ModelCubeMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ModelPolygonMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ModelVertexMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/NearBuildMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/OptionInstanceMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolContainerScreenMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolDeathScreenMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolGuiMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolHandMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/QolPlacementMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/RenderChunkDirtyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/RenderChunkInfoAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/RenderChunkInfoMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SearchThrottleMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullBufferMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullLayerMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullOwnerMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionOcclusionGraphAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SkyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SpriteActivityMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TextureAtlasMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningAutosaveMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningBuildOrderMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningChunkSaveMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningCullMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningDropsMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarAreaAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarDrawMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarViewAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningGlintMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningGrassMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLevelMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningLoadingMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningPortalMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningServerLightMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSignMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSkyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningTextureAtlasMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningTextureSpriteMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningUnpackNetMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningUnpackSectionMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningWorldMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/VisibleSectionsMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/VisibleSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ZoomCameraMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ZoomMouseMixin.java`
- `common/src/main/java/com/aetherium/perf/TuningSupport.java`
- `common/src/main/java/com/aetherium/util/AetheriumLog.java`

## Verifying this row before shipping it

The port is applied with `sh tools/port.sh 1.17` and then checked against the
real mapped jar - the jar is the authority, not a wiki:

```sh
MC=1.17
JAR=$(find "$HOME/.gradle/caches/fabric-loom" -name "minecraft-*-"$MC"-*.jar" 2>/dev/null | head -n 1)
test -n "$JAR" || { echo "run ./gradlew build once so Loom downloads $MC"; exit 1; }
javap -p -classpath "$JAR" net.minecraft.client.renderer.LightTexture | grep -E 'updateLightTexture|tickLightTexture|pixels|NativeImage|DynamicTexture'
javap -p -classpath "$JAR" net.minecraft.client.gui.Gui | grep -E 'public void render'
javap -p -classpath "$JAR" net.minecraft.client.renderer.LevelRenderer | grep -E 'setSectionDirty|allChanged'
javap -p -classpath "$JAR" net.minecraft.client.gui.screens.OptionsScreen | grep -nE 'lambda|init'
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

- Java 16 toolchain. `LightTexture` still holds a `NativeImage`; `LevelRenderer#setSectionDirty` has the boolean overload on some mappings only - check with javap.
- No `GuiGraphics`; `MatrixStack`-based drawing.

Generated by `tools/gen_deltas.py`; re-run that instead of editing.
