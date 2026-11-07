package com.aetherium.mixin;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

import org.objectweb.asm.tree.AnnotationNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.MethodNode;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;

/**
 * Steps aside when another installed mod already changes the same game method as one of
 * Aetherium's performance options.
 *
 * <p>Two mods rewriting the same method do not mix: the second one either fails to apply or,
 * worse, skips the code the first one relies on (a crash while the game registers its blocks).
 * Right after every mod's mixins are known, and before any game class is transformed, this
 * looks at the other mixins that target each class one of the options below hooks. If one of
 * them replaces or hooks the same method, Aetherium's mixin is taken out of the target (so it is
 * never applied) and the option is switched off for this launch; the menu then explains why.
 * Nothing here knows about any particular mod: the check reads the other mixins' own
 * annotations, so it covers whatever is installed.</p>
 *
 * <p>Everything goes through reflection into the mixin library's internals and is wrapped in
 * a catch-all: if the library ever changes shape, the check simply finds nothing and Aetherium
 * behaves as before.</p>
 */
public final class Overlap {

    /** Any hook or replacement of the method by another mod counts. */
    private static final String ANY = "any";
    /** Only another mod replacing the whole method counts (hooks into it compose fine). */
    private static final String REPLACED = "replaced";

    /**
     * Mixin simple name, option it belongs to, what counts, then the names of the handler methods
     * whose targets the option owns. The targets are read from those handlers' own annotations,
     * so they come out in whatever names the running game uses: a release jar carries the
     * rewritten (intermediary) names in the annotations and has no refmap, a development run
     * carries the source names.
     */
    private static final String[][] TABLE = {
        {"TuningFrameDecoderMixin", "fast_network", ANY, "aetherium$decode"},
        {"TuningInflateMixin", "fast_network", ANY, "aetherium$decode"},
        {"TuningSearchMixin", "lazy_search", ANY, "aetherium$deferRefresh", "aetherium$deferIndex"},
        {"TuningEntityTickMixin", "entity_tick_culling", ANY, "aetherium$tickIfSeen"},
        // These hook one call inside a method other mods commonly hook too (the game's own
        // rendering API among them); that composes, only a mod replacing the whole method clashes.
        {"TuningSnapshotMixin", "fast_section_snapshots", REPLACED, "aetherium$fluidFromBlock"},
        {"TuningChestBakeMixin", "batched_chests", REPLACED, "aetherium$bakeChest", "aetherium$compileStart"},
        {"TuningHudBlendMixin", "hud_fps", ANY, "aetherium$captureBlend", "aetherium$captureBlendSeparate"},
        {"TuningHudFrameMixin", "hud_fps", REPLACED, "aetherium$hudRate"},
        {"TuningServerIdleMixin", "server_idle", ANY, "aetherium$park"},
    };

    private static final String OWN_PACKAGE = "com.aetherium.";
    private static final String OVERWRITE = "Lorg/spongepowered/asm/mixin/Overwrite;";
    private static final String INJECTORS = "Lorg/spongepowered/asm/mixin/injection/";
    private static final String EXTRA_INJECTORS = "/mixinextras/injector/";

    /** Options switched off this launch because another mod already changes their methods. */
    private static final Set<String> YIELDED = ConcurrentHashMap.newKeySet();
    /** One line per mixin that stepped aside (for the log and the smoke report). */
    private static final List<String> NOTES = Collections.synchronizedList(new ArrayList<String>());

    private Overlap() {
    }

    /** Whether the option (e.g. {@code "fast_network"}) stepped aside this launch. */
    public static boolean yielded(final String option) {
        return option != null && YIELDED.contains(option);
    }

    public static Set<String> yieldedOptions() {
        return Collections.unmodifiableSet(new HashSet<String>(YIELDED));
    }

    public static List<String> notes() {
        synchronized (NOTES) {
            return new ArrayList<String>(NOTES);
        }
    }

    /** One summary line of the last check (log and smoke report). */
    private static volatile String summary = "not run";
    private static int scanned;
    private static int shared;
    private static String firstError;

    public static String summary() {
        return summary;
    }

    /** What the check compared, per Aetherium mixin that shares a class (for diagnosing a miss). */
    private static final List<String> DETAILS = Collections.synchronizedList(new ArrayList<String>());

    public static List<String> details() {
        synchronized (DETAILS) {
            return new ArrayList<String>(DETAILS);
        }
    }

    /** Called from {@code acceptTargets}: every config is prepared, nothing is applied yet. */
    static void resolve(final Set<String> myTargets) {
        scanned = 0;
        shared = 0;
        firstError = null;
        for (final String target : new ArrayList<String>(myTargets)) {
            try {
                resolveTarget(myTargets, target);
            } catch (final Throwable error) {
                // The mixin library changed shape: this class keeps the old behaviour.
                note(error);
            }
        }
        summary = scanned + " classes, " + shared + " also changed by other mods, " + YIELDED.size()
                + " options stepped aside" + (firstError == null ? "" : ", first error: " + firstError);
    }

    private static void note(final Throwable error) {
        if (firstError == null) {
            final StackTraceElement[] trace = error.getStackTrace();
            firstError = error + (trace.length > 0 ? " at " + trace[0] : "");
        }
    }

    private static void resolveTarget(final Set<String> myTargets, final String target) {
        final Collection<IMixinInfo> all = mixinsFor(target);
        scanned++;
        if (all.isEmpty()) {
            return;
        }
        final List<IMixinInfo> ours = new ArrayList<IMixinInfo>();
        final List<IMixinInfo> others = new ArrayList<IMixinInfo>();
        for (final IMixinInfo info : all) {
            (info.getClassName().startsWith(OWN_PACKAGE) ? ours : others).add(info);
        }
        if (others.isEmpty()) {
            return;
        }
        shared++;
        int stepped = 0;
        for (final IMixinInfo mine : ours) {
            final String[] row = row(simpleName(mine.getClassName()));
            if (row == null) {
                stepped += guard(mine, target, others);
                continue;
            }
            final Set<String> owned = ownedTargets(mine, row);
            if (owned.isEmpty() && ANY.equals(row[2])) {
                // The handler's targets could not be read: with another mod on the same class,
                // stepping aside is the safe side.
                YIELDED.add(row[1]);
                stepped++;
                final boolean dropped = drop(mine, target);
                NOTES.add(row[1] + ": " + simpleName(mine.getClassName()) + " steps aside, " + others.size()
                        + " other mixins change " + target + " (targets unreadable)" + (dropped ? "" : " (hooks left inert)"));
                continue;
            }
            final boolean replacedOnly = REPLACED.equals(row[2]);
            final StringBuilder detail = new StringBuilder(simpleName(mine.getClassName())).append(" owns ").append(owned);
            for (final IMixinInfo other : others) {
                detail.append("; ").append(simpleName(other.getClassName())).append(' ').append(describe(other, target));
            }
            DETAILS.add(detail.toString());
            for (final IMixinInfo other : others) {
                String hit;
                try {
                    hit = firstShared(other, target, owned, replacedOnly);
                } catch (final Throwable error) {
                    note(error);
                    hit = null;
                }
                if (hit == null) {
                    continue;
                }
                YIELDED.add(row[1]);
                stepped++;
                final boolean dropped = drop(mine, target);
                NOTES.add(row[1] + ": " + simpleName(mine.getClassName()) + " steps aside, " + other.getClassName()
                        + " already changes " + target + "." + hit + (dropped ? "" : " (hooks left inert)"));
                break;
            }
        }
        if (stepped > 0 && stepped == ours.size()) {
            // Every Aetherium mixin of this class stepped aside: make sure the class is left alone.
            myTargets.remove(target);
        }
    }

    /**
     * Safety net for the Aetherium mixins without an option row: an injector into a method another
     * mod replaces wholesale cannot apply ("merged by"), so such a mixin is taken out beforehand.
     */
    private static int guard(final IMixinInfo mine, final String target, final List<IMixinInfo> others) {
        try {
            final Set<String> owned = ownedTargets(mine, null);
            if (owned.isEmpty()) {
                return 0;
            }
            for (final IMixinInfo other : others) {
                final String hit = firstShared(other, target, owned, true);
                if (hit != null) {
                    final boolean dropped = drop(mine, target);
                    NOTES.add(simpleName(mine.getClassName()) + " steps aside, " + other.getClassName()
                            + " replaces " + target + "." + hit + (dropped ? "" : " (hooks left inert)"));
                    return 1;
                }
            }
        } catch (final Throwable error) {
            note(error);
        }
        return 0;
    }

    // ------------------------------------------------------------------ the other mixins

    /** The method of {@code owned} the other mixin replaces or hooks, or null. */
    private static String firstShared(final IMixinInfo other, final String target, final Set<String> owned,
                                      final boolean replacedOnly) {
        if (owned.isEmpty()) {
            return null;
        }
        final ClassNode node = other.getClassNode(0);
        if (node == null || node.methods == null) {
            return null;
        }
        for (final MethodNode method : node.methods) {
            if (has(method.invisibleAnnotations, OVERWRITE) || has(method.visibleAnnotations, OVERWRITE)) {
                final String name = runtimeName(target, method);
                if (owned.contains(name)) {
                    return name;
                }
            }
            if (replacedOnly) {
                continue;
            }
            final String hit = injectorHit(other, method.visibleAnnotations, owned);
            if (hit != null) {
                return hit;
            }
            final String hidden = injectorHit(other, method.invisibleAnnotations, owned);
            if (hidden != null) {
                return hidden;
            }
        }
        return null;
    }

    /** The game methods the listed handlers (all injectors when row is null) of an Aetherium mixin hook (runtime names). */
    private static Set<String> ownedTargets(final IMixinInfo mine, final String[] row) {
        final Set<String> handlers = new HashSet<String>();
        for (int i = 3; row != null && i < row.length; i++) {
            handlers.add(row[i]);
        }
        final Set<String> owned = new HashSet<String>();
        final ClassNode node = mine.getClassNode(0);
        if (node == null || node.methods == null) {
            return owned;
        }
        for (final MethodNode method : node.methods) {
            if (row != null && !handlers.contains(method.name)) {
                continue;
            }
            for (final List<AnnotationNode> list : java.util.Arrays.asList(method.visibleAnnotations,
                    method.invisibleAnnotations)) {
                if (list == null) {
                    continue;
                }
                for (final AnnotationNode annotation : list) {
                    if (annotation.desc != null && (annotation.desc.startsWith(INJECTORS)
                            || annotation.desc.contains(EXTRA_INJECTORS))) {
                        for (final String selector : strings(annotation, "method")) {
                            final String name = methodName(remap(mine, selector));
                            if (name != null) {
                                owned.add(name);
                            }
                        }
                    }
                }
            }
        }
        return owned;
    }

    /** The methods another mixin replaces (=name) and hooks (name), for the details list. */
    private static String describe(final IMixinInfo other, final String target) {
        try {
            final ClassNode node = other.getClassNode(0);
            final StringBuilder out = new StringBuilder("[");
            if (node != null && node.methods != null) {
                for (final MethodNode method : node.methods) {
                    if (has(method.invisibleAnnotations, OVERWRITE) || has(method.visibleAnnotations, OVERWRITE)) {
                        out.append(" =").append(runtimeName(target, method));
                    }
                    for (final List<AnnotationNode> list : java.util.Arrays.asList(method.visibleAnnotations,
                            method.invisibleAnnotations)) {
                        if (list == null) {
                            continue;
                        }
                        for (final AnnotationNode annotation : list) {
                            if (annotation.desc != null && (annotation.desc.startsWith(INJECTORS)
                                    || annotation.desc.contains(EXTRA_INJECTORS))) {
                                for (final String selector : strings(annotation, "method")) {
                                    out.append(' ').append(methodName(remap(other, selector)));
                                }
                            }
                        }
                    }
                }
            }
            return out.append(" ]").toString();
        } catch (final Throwable error) {
            return "[" + error + "]";
        }
    }

    private static String injectorHit(final IMixinInfo other, final List<AnnotationNode> annotations,
                                      final Set<String> owned) {
        if (annotations == null) {
            return null;
        }
        for (final AnnotationNode annotation : annotations) {
            final String desc = annotation.desc;
            if (desc == null || !(desc.startsWith(INJECTORS) || desc.contains(EXTRA_INJECTORS))) {
                continue;
            }
            for (final String selector : strings(annotation, "method")) {
                final String name = methodName(remap(other, selector));
                if (name != null && owned.contains(name)) {
                    return name;
                }
            }
        }
        return null;
    }

    private static boolean has(final List<AnnotationNode> annotations, final String desc) {
        if (annotations != null) {
            for (final AnnotationNode annotation : annotations) {
                if (desc.equals(annotation.desc)) {
                    return true;
                }
            }
        }
        return false;
    }

    private static List<String> strings(final AnnotationNode annotation, final String key) {
        final List<String> out = new ArrayList<String>();
        final List<Object> values = annotation.values;
        if (values == null) {
            return out;
        }
        for (int i = 0; i + 1 < values.size(); i += 2) {
            if (!key.equals(values.get(i))) {
                continue;
            }
            final Object value = values.get(i + 1);
            if (value instanceof String) {
                out.add((String) value);
            } else if (value instanceof List) {
                for (final Object item : (List<?>) value) {
                    if (item instanceof String) {
                        out.add((String) item);
                    }
                }
            }
        }
        return out;
    }

    // ------------------------------------------------------------------ names

    /**
     * The bare method name of a mixin target selector ({@code Lowner;name(desc)}, {@code name(desc)},
     * {@code name}); null for wildcards, which say nothing about a particular method.
     */
    static String methodName(final String selector) {
        if (selector == null) {
            return null;
        }
        String s = selector.trim();
        final int semicolon = s.indexOf(';');
        if (s.startsWith("L") && semicolon > 0) {
            s = s.substring(semicolon + 1);
        }
        int end = s.length();
        for (final char stop : new char[] {'(', '*', '{', ':'}) {
            final int at = s.indexOf(stop);
            if (at >= 0 && at < end) {
                end = at;
            }
        }
        s = s.substring(0, end).trim();
        return s.isEmpty() ? null : s;
    }

    /** The selector as the running game names it (through the mixin's own refmap). */
    private static String remap(final IMixinInfo info, final String selector) {
        try {
            final Object config = info.getConfig();
            final Method getter = config.getClass().getMethod("getReferenceMapper");
            getter.setAccessible(true);
            final Object mapper = getter.invoke(config);
            if (mapper instanceof org.spongepowered.asm.mixin.refmap.IReferenceMapper) {
                final String mapped = ((org.spongepowered.asm.mixin.refmap.IReferenceMapper) mapper)
                        .remap(info.getClassRef(), selector);
                if (mapped != null) {
                    return mapped;
                }
            }
        } catch (final Throwable ignored) {
            // No refmap: the selector is already in runtime names (development) or unmapped.
        }
        return selector;
    }

    /**
     * The name a replacing method has in the running game: the mixin library maps it through the
     * environment's remappers (identity in a normal game, intermediary to development names in a
     * development run) before it looks the method up, so the same is done here.
     */
    private static String runtimeName(final String target, final MethodNode method) {
        try {
            final Class<?> environment = Class.forName("org.spongepowered.asm.mixin.MixinEnvironment");
            final Object current = environment.getMethod("getCurrentEnvironment").invoke(null);
            final Object remappers = environment.getMethod("getRemappers").invoke(current);
            final Method map = remappers.getClass().getMethod("mapMethodName", String.class, String.class, String.class);
            map.setAccessible(true);
            final Object mapped = map.invoke(remappers, target.replace('.', '/'), method.name, method.desc);
            if (mapped instanceof String) {
                return (String) mapped;
            }
        } catch (final Throwable ignored) {
            // No remappers: the name is already the running game's.
        }
        return method.name;
    }

    private static String[] row(final String simpleName) {
        for (final String[] row : TABLE) {
            if (row[0].equals(simpleName)) {
                return row;
            }
        }
        return null;
    }

    private static String simpleName(final String className) {
        final int dot = className.lastIndexOf('.');
        return dot < 0 ? className : className.substring(dot + 1);
    }

    // ------------------------------------------------------------------ mixin library internals

    /** Every mixin (any mod) that targets the class, after all configs were prepared. */
    @SuppressWarnings("unchecked")
    private static Collection<IMixinInfo> mixinsFor(final String target) {
        try {
            final Class<?> classInfo = Class.forName("org.spongepowered.asm.mixin.transformer.ClassInfo");
            final Object info = classInfo.getMethod("fromCache", String.class).invoke(null, target.replace('.', '/'));
            if (info == null) {
                return Collections.emptyList();
            }
            final Object mixins = allMixins(classInfo, info);
            final List<IMixinInfo> out = new ArrayList<IMixinInfo>();
            if (mixins instanceof Collection) {
                for (final Object mixin : (Collection<Object>) mixins) {
                    if (mixin instanceof IMixinInfo) {
                        out.add((IMixinInfo) mixin);
                    }
                }
            }
            return out;
        } catch (final Throwable error) {
            note(error);
            return Collections.emptyList();
        }
    }

    /** ClassInfo's set of every mixin targeting it (method or field, whichever this library has). */
    private static Object allMixins(final Class<?> classInfo, final Object info) throws Exception {
        try {
            final Method getter = classInfo.getDeclaredMethod("getMixins");
            getter.setAccessible(true);
            return getter.invoke(info);
        } catch (final NoSuchMethodException missing) {
            // fall through to the field
        }
        try {
            final Field field = classInfo.getDeclaredField("mixins");
            field.setAccessible(true);
            return field.get(info);
        } catch (final NoSuchFieldException missing) {
            // fall through to the search
        }
        final StringBuilder seen = new StringBuilder();
        for (final Field field : classInfo.getDeclaredFields()) {
            seen.append(' ').append(field.getName());
            if (Set.class.isAssignableFrom(field.getType()) && field.getName().toLowerCase(java.util.Locale.ROOT).contains("mixin")
                    && !field.getName().toLowerCase(java.util.Locale.ROOT).contains("applied")) {
                field.setAccessible(true);
                return field.get(info);
            }
        }
        throw new NoSuchFieldException("no mixin set in ClassInfo; fields:" + seen);
    }

    /** Takes one Aetherium mixin out of one target, so it is never applied there. */
    @SuppressWarnings("unchecked")
    private static boolean drop(final IMixinInfo mine, final String target) {
        try {
            final Object config = mine.getConfig();
            final Field field = config.getClass().getDeclaredField("mixinMapping");
            field.setAccessible(true);
            final Map<String, List<Object>> mapping = (Map<String, List<Object>>) field.get(config);
            final List<Object> list = mapping.get(target);
            if (list == null || !list.remove(mine)) {
                return false;
            }
            if (list.isEmpty()) {
                mapping.remove(target);
            }
            return true;
        } catch (final Throwable error) {
            return false;
        }
    }
}
