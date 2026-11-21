package com.aetherium.perf;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Text measurement cache (1.18.0): remembers the width of plain strings a font measured recently.
 * Name tags, the scoreboard, the debug screen and many menus measure the same strings every frame;
 * a hit is one hash lookup instead of a glyph lookup per character.
 *
 * <p>One instance per font, used from the client thread only (other threads measure normally).
 * Cleared when resources reload or the Unicode font setting changes ({@link #invalidateAll}).</p>
 */
public final class TextWidths {
    static final int MAX = 1024;
    /** Longer strings are rare and unlikely to repeat; they are measured normally. */
    static final int MAX_LENGTH = 256;
    static final int REFRESH_TICKS = 20;

    private static volatile int generation;
    private static volatile Thread owner;

    private final Map<String, Integer> widths = new LinkedHashMap<String, Integer>(256, 0.75F, true) {
        private static final long serialVersionUID = 1L;

        @Override
        protected boolean removeEldestEntry(final Map.Entry<String, Integer> eldest) {
            return size() > MAX;
        }
    };
    private int seen = -1;

    /** The client thread; set by the client tick so other threads never touch the caches. */
    public static void setOwner(final Thread thread) {
        owner = thread;
    }

    public static void invalidateAll() {
        generation++;
    }

    /** Client ticks since the caches were last emptied. */
    private static int ticks;

    /**
     * Client tick: the caches are emptied once a second, so a font change of any kind (resource
     * packs, the Unicode font option, a mod swapping glyphs) shows up within a second.
     */
    public static void tick() {
        if (++ticks >= REFRESH_TICKS) {
            ticks = 0;
            generation++;
        }
    }

    /** Whether this call may use the cache (enabled, client thread, short enough). */
    public static boolean usable(final String text) {
        return Tuning.textWidthCache && text != null && text.length() <= MAX_LENGTH && Thread.currentThread() == owner;
    }

    /** Cached width, or null when not known. */
    public Integer get(final String text) {
        if (this.seen != generation) {
            this.widths.clear();
            this.seen = generation;
            return null;
        }
        return this.widths.get(text);
    }

    public void put(final String text, final int width) {
        if (this.seen == generation) {
            this.widths.put(text, Integer.valueOf(width));
        }
    }

    int size() {
        return this.widths.size();
    }
}
