package com.aetherium.perf;

import java.util.Collections;
import java.util.Map;
import java.util.WeakHashMap;

/**
 * Background chunk unpacking (1.18.0): the block data of a chunk packet is unpacked into fresh
 * chunk sections on the network thread, as soon as the packet arrives, by vanilla's own section
 * reader. When the main thread then applies the packet, each section read is replaced by the section
 * that was already unpacked from exactly the same bytes, and the read position jumps to where
 * vanilla's own read would have ended.
 *
 * <p>Every hand-over is checked: the prepared section is only used when the main thread is about to
 * read the same packet buffer at exactly the byte offset where that section started. Anything else
 * (a mod that reads the buffer differently, a packet applied twice, a world change in between) reads
 * the bytes normally, exactly like vanilla.</p>
 *
 * <p>Thread safety: {@link #offer} runs on the network thread, {@link #take} on the client thread;
 * the map is synchronized, which also publishes the prepared sections safely.</p>
 */
public final class ChunkUnpack {
    /** Sections unpacked from one packet's bytes, in reading order. */
    public static final class Prepared {
        final int[] start;
        final int[] end;
        final Object[] sections;
        int next;

        public Prepared(final int[] start, final int[] end, final Object[] sections) {
            this.start = start;
            this.end = end;
            this.sections = sections;
        }
    }

    /** Result of a successful {@link #take}: the section and the offset where its bytes end. */
    public static final class Taken {
        public final Object section;
        public final int end;

        Taken(final Object section, final int end) {
            this.section = section;
            this.end = end;
        }
    }

    // Keyed by the packet's byte array (identity), so an entry disappears with its packet.
    private static final Map<Object, Prepared> BY_DATA = Collections.synchronizedMap(new WeakHashMap<Object, Prepared>());
    private static volatile long prepared;
    private static volatile long used;
    private static volatile long fallback;
    private static volatile String failure;

    private ChunkUnpack() {
    }

    public static void offer(final Object data, final Prepared sections) {
        if (data != null && sections != null && sections.sections.length > 0) {
            BY_DATA.put(data, sections);
            prepared += sections.sections.length;
        }
    }

    /**
     * The section prepared from {@code data} starting at {@code offset}, or null to read normally.
     * Each prepared section is handed out once.
     */
    public static Taken take(final Object data, final int offset) {
        if (data == null) {
            return null;
        }
        final Prepared p = BY_DATA.get(data);
        if (p == null) {
            return null;
        }
        synchronized (p) {
            for (int i = p.next; i < p.sections.length; i++) {
                if (p.start[i] == offset && p.sections[i] != null) {
                    final Object section = p.sections[i];
                    p.sections[i] = null;
                    p.next = i + 1;
                    if (p.next >= p.sections.length) {
                        BY_DATA.remove(data);
                    }
                    used++;
                    return new Taken(section, p.end[i]);
                }
            }
        }
        fallback++;
        return null;
    }

    /** The unpacking itself failed (logged once by the caller); the feature keeps reading normally. */
    public static void failed(final Throwable error) {
        if (failure == null) {
            failure = String.valueOf(error);
        }
    }

    public static String stats() {
        return prepared + " sections unpacked early, " + used + " used, " + fallback + " read normally"
                + (failure == null ? "" : ", failure: " + failure);
    }

    /** Tests. */
    static void reset() {
        BY_DATA.clear();
        prepared = 0;
        used = 0;
        fallback = 0;
        failure = null;
    }
}
