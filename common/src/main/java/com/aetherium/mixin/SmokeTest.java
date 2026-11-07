package com.aetherium.mixin;

import com.aetherium.perf.TuningSupport;
import java.io.File;
import java.io.FileOutputStream;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.ConcurrentHashMap;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.AnnotationNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;

/**
 * CI smoke test for every Minecraft version (the smoke workflow; off unless the JVM is started
 * with {@code -Daetherium.smoke=<dir>}). Once the title screen has been up for a few seconds it
 * loads every class an Aetherium mixin targets, so every mixin is applied, then writes
 * {@code <dir>/report.txt} and ends the process. The report lists:
 * <ul>
 *   <li>targets that failed to load (a broken mixin: wrong handler signature, missing shadow);</li>
 *   <li>injectors that found nothing to hook ({@code require = 0} turns these into a feature
 *       that silently does nothing, which this makes visible);</li>
 *   <li>the mixins that were applied and the tuning features this build has.</li>
 * </ul>
 * Never active in a normal game: {@link #ENABLED} is false and every hook returns at once.
 */
public final class SmokeTest {
    static final String DIR = System.getProperty("aetherium.smoke");
    public static final boolean ENABLED = DIR != null && !DIR.isEmpty();

    private static final String MERGED = "Lorg/spongepowered/asm/mixin/transformer/meta/MixinMerged;";
    private static final String[] HANDLER_PREFIXES = {"handler$", "redirect$", "modify$", "args$", "localvar$",
            "constant$", "wrapOperation$", "modifyExpressionValue$", "modifyReturnValue$", "wrapWithCondition$",
            "wrapMethod$", "modifyReceiver$"};

    private static final Set<String> TARGETS = ConcurrentHashMap.newKeySet();
    private static final Set<String> UNUSED = ConcurrentHashMap.newKeySet();
    private static final List<Object[]> APPLIED_TO = new ArrayList<Object[]>();
    private static int ticks;
    private static boolean done;

    private SmokeTest() {
    }

    static void targets(final Set<String> mine) {
        if (ENABLED) {
            TARGETS.addAll(mine);
        }
    }

    /**
     * After a mixin was applied. The class is only scanned when the report is written: MixinExtras
     * applies some injectors (WrapOperation, ModifyExpressionValue) after this callback, into the
     * same ClassNode, so scanning now would report their handlers as unused.
     */
    static void applied(final ClassNode target, final String mixinClassName) {
        if (ENABLED) {
            synchronized (APPLIED_TO) {
                APPLIED_TO.add(new Object[] {target, mixinClassName});
            }
        }
    }

    private static void scan(final ClassNode target, final String mixinClassName) {
        if (target.methods == null) {
            return;
        }
        final List<String> handlers = new ArrayList<String>();
        for (final MethodNode method : target.methods) {
            if (handler(method.name) && mergedFrom(method, mixinClassName)) {
                handlers.add(method.name);
            }
        }
        if (handlers.isEmpty()) {
            return;
        }
        final Set<String> called = new TreeSet<String>();
        for (final MethodNode method : target.methods) {
            if (method.instructions == null) {
                continue;
            }
            for (final AbstractInsnNode insn : method.instructions) {
                if (insn instanceof MethodInsnNode) {
                    called.add(((MethodInsnNode) insn).name);
                }
            }
        }
        for (final String name : handlers) {
            if (!called.contains(name)) {
                UNUSED.add(shortName(mixinClassName) + " -> " + target.name.replace('/', '.') + " :: " + name);
            }
        }
    }

    private static boolean handler(final String name) {
        for (final String prefix : HANDLER_PREFIXES) {
            if (name.startsWith(prefix)) {
                return true;
            }
        }
        return false;
    }

    private static boolean mergedFrom(final MethodNode method, final String mixinClassName) {
        return mergedFrom(method.invisibleAnnotations, mixinClassName) || mergedFrom(method.visibleAnnotations, mixinClassName);
    }

    private static boolean mergedFrom(final List<AnnotationNode> annotations, final String mixinClassName) {
        if (annotations == null) {
            return false;
        }
        for (final AnnotationNode annotation : annotations) {
            if (!MERGED.equals(annotation.desc) || annotation.values == null) {
                continue;
            }
            for (int i = 0; i + 1 < annotation.values.size(); i += 2) {
                if ("mixin".equals(annotation.values.get(i)) && mixinClassName.equals(annotation.values.get(i + 1))) {
                    return true;
                }
            }
        }
        return false;
    }

    /** Client tick (title screen included): run once, a few seconds after start-up. */
    public static void tick() {
        if (!ENABLED || done || ++ticks < 200) {
            return;
        }
        done = true;
        final StringBuilder out = new StringBuilder();
        out.append("minecraft ").append(AetheriumMixinPlugin.minecraftVersion()).append('\n');
        final List<String> failures = new ArrayList<String>();
        final ClassLoader loader = SmokeTest.class.getClassLoader();
        final List<String> targets = new ArrayList<String>(TARGETS);
        Collections.sort(targets);
        for (final String target : targets) {
            try {
                Class.forName(target.replace('/', '.'), false, loader);
            } catch (final Throwable error) {
                Throwable root = error;
                while (root.getCause() != null && root.getCause() != root) {
                    root = root.getCause();
                }
                failures.add(target + ": " + error + (root != error ? " | root: " + root : ""));
            }
        }
        out.append("targets ").append(targets.size()).append(", failed ").append(failures.size()).append('\n');
        for (final String failure : failures) {
            out.append("FAILED ").append(failure).append('\n');
        }
        synchronized (APPLIED_TO) {
            for (final Object[] entry : APPLIED_TO) {
                scan((ClassNode) entry[0], (String) entry[1]);
            }
        }
        final List<String> unused = new ArrayList<String>(UNUSED);
        Collections.sort(unused);
        out.append("unused injectors ").append(unused.size()).append('\n');
        for (final String line : unused) {
            out.append("UNUSED ").append(line).append('\n');
        }
        final List<String> applied = new ArrayList<String>(AetheriumMixinPlugin.applied());
        Collections.sort(applied);
        out.append("applied ").append(applied.size()).append(": ").append(String.join(" ", applied)).append('\n');
        out.append("tuning options available:");
        for (final String key : TuningSupport.optionKeys()) {
            if (TuningSupport.option(key)) {
                out.append(' ').append(key);
            }
        }
        out.append('\n');
        String renderScale;
        try {
            renderScale = com.aetherium.client.ScaledRenderTarget.selfTest();
        } catch (final Throwable error) {
            renderScale = "FAILED: " + error;
        }
        out.append("render scale ").append(renderScale).append('\n');
        String farTerrain;
        try {
            farTerrain = com.aetherium.client.FarTerrain.selfTest();
        } catch (final Throwable error) {
            farTerrain = "FAILED: " + error;
        }
        out.append("far terrain ").append(farTerrain).append('\n');
        String foodOutline;
        try {
            foodOutline = com.aetherium.client.QolFoodIcons.selfTest();
        } catch (final Throwable error) {
            foodOutline = "FAILED: " + error;
        }
        out.append("food outline ").append(foodOutline).append('\n');
        out.append("stepped aside ").append(Overlap.yieldedOptions().size()).append('\n');
        out.append("overlap check ").append(Overlap.summary()).append('\n');
        for (final String note : Overlap.notes()) {
            out.append("STEPPED ").append(note).append('\n');
        }
        try {
            final File dir = new File(DIR);
            dir.mkdirs();
            try (Writer writer = new OutputStreamWriter(new FileOutputStream(new File(dir, "report.txt")),
                    StandardCharsets.UTF_8)) {
                writer.write(out.toString());
            }
        } catch (final Exception error) {
            System.out.println("[Aetherium smoke] could not write the report: " + error);
        }
        System.out.println("[Aetherium smoke] report\n" + out);
        Runtime.getRuntime().halt(0);
    }

    private static String shortName(final String className) {
        final int dot = className.lastIndexOf('.');
        return dot < 0 ? className : className.substring(dot + 1);
    }
}
