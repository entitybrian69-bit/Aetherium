# Aetherium port: Minecraft 1.20.3

- Java toolchain: **17** (mixin compatibilityLevel `JAVA_17`)
- Fabric loader floor: **0.15.11**, Loom **1.16.1**
- Fabric API (compile-only, optional at runtime): **0.91.1+1.20.3**
- NeoForge: **does not exist for this version - the neoforge module is disabled**
- Mappings: officialMojangMappings()
- Status: **derived**

## What the delta changes

### Build pins

```properties
minecraft_version=1.20.3
java_version=17
fabric_loader_version=0.15.11
fabric_api_version=0.91.1+1.20.3
neoforge_version=unavailable (module disabled)
```

### Era transforms applied by the generated patch

- `anim=on`
- `bg-threads=property`
- `cam-pos=getter`
- `camera-entity=getter`
- `camera-get=getter`
- `chat=literal`
- `chunk-renderer=none`
- `chunk-upload=srd`
- `cull-new=old`
- `fog-cull-x=none`
- `fog-cull=section`
- `food-api=item`
- `frustum-cap=joml`
- `hud=graphics-float`
- `layer-sig=joml`
- `level-extract=none`
- `light-hook=color`
- `math=joml`
- `mesh-alloc=v16`
- `model-fast=joml`
- `near-build=none`
- `options=instances`
- `particle-fast=facing`
- `pose-normals=plain`
- `q-hud=g202`
- `q-inv=t17`
- `qol16=none`
- `render-scale=r17`
- `screen-owner=minecraft`
- `screen-pkg=flat`
- `search-throttle=none`
- `section-api=srd`
- `section-compile=rebuild202`
- `section-cull=rendered19`
- `section-vis=sections`
- `simdist=sim`
- `sky=rb193`
- `t-chunk-save=v16`
- `t-drops=v16`
- `t-far-draw=j193`
- `t-glint=v16`
- `t-portal=v16`
- `t-sky=e17`
- `t-world=w182`
- `vignette=render`
- `zoom-cam=double`
- `zoom=on`

### Where the per-version Java comes from

Every Java difference is an era block in the source (`// @era:<name>-begin ...`), selected
by `tools/eras.py`; the variant names above say which shape this row compiles.
`python3 tools/stubcheck.py 1.20.3` compiles exactly this selection against the
Minecraft signatures CI extracted from this version's jar (`tools/probe/1.20.3.txt`).

### Version facts from earlier porting research (historical; tools/eras.py is authoritative)

- `options_access` = **getters** - Options#renderDistance()/simulationDistance() OptionInstance getters are 1.19+; 1.17-1.18.2 read public int fields, and 1.16.5-1.17.x have no simulationDistance at all (the benchmark label degrades to `sim=n/a`) [VERIFIED: the 1.19/1.19.1/1.19.2 legs compiled the getters while 1.18.2 rejected them]
- `button_builder` = **True** - Button.builder arrives with the 1.19.3 screen rework; 1.16.5-1.19.2 construct Button directly [VERIFIED: the 1.19.3 leg rejected the constructor form]
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
- `common/src/main/java/com/aetherium/client/ClientHooks.java`
- `common/src/main/java/com/aetherium/client/QolFoodInfo.java`
- `common/src/main/java/com/aetherium/client/QolHudDraw.java`
- `common/src/main/java/com/aetherium/client/QolInventory.java`
- `common/src/main/java/com/aetherium/mixin/core/BufferBuilderAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/DefaultedVertexAccessor.java`
- `common/src/main/java/com/aetherium/mixin/core/FastModelMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/FastParticleMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/GuiMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ModelCubeMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ModelPolygonMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ModelVertexMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullBufferMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionCullLayerMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SectionSpritesMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/SkyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningChunkSaveMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningDropsMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningFarDrawMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningGlintMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningPortalMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningSkyMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/TuningWorldMixin.java`
- `common/src/main/java/com/aetherium/mixin/core/ZoomCameraMixin.java`

## Verifying this row before shipping it

The port is applied with `sh tools/port.sh 1.20.3` and then checked against the
real mapped jar - the jar is the authority, not a wiki:

```sh
MC=1.20.3
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

- `mouseClicked`/`mouseScrolled` return `boolean` from 1.20.2; Screen overrides in AetheriumVideoOptionsScreen need no change on this row.

Generated by `tools/gen_deltas.py`; re-run that instead of editing.
