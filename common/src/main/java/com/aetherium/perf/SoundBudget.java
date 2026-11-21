package com.aetherium.perf;

import java.util.HashMap;
import java.util.Map;

/**
 * Sound culling (1.18.0): decides, before a sound gets an audio channel, whether it can be heard at
 * all and whether the same sound has already started too often in this tick.
 *
 * <p>Only one-shot world sounds are ever skipped: positional (not relative to the listener), with
 * linear fading, not streamed, not looping and not updated every tick. Anything that keeps playing
 * (minecarts, beacons, music discs, rain loops, music) always starts, because the listener could
 * walk into its range while it plays. A one-shot sound that starts out of range would be silent for
 * its whole (short) life in vanilla too; the reach keeps a 25% + 2 block margin for that.</p>
 *
 * <p>Render-thread only (sounds are started from the client thread).</p>
 */
public final class SoundBudget {
    /** "Same sound per tick" at this value or above means no limit. */
    public static final int SAME_UNLIMITED = 33;
    /** Length of one counting window: a game tick. */
    private static final long WINDOW_NANOS = 50_000_000L;

    private static long window = Long.MIN_VALUE;
    private static final Map<Object, int[]> STARTED = new HashMap<Object, int[]>();
    private static long skippedFar;
    private static long skippedSame;

    private SoundBudget() {
    }

    /**
     * Whether a one-shot sound at this squared distance can be heard.
     *
     * @param distanceSq          squared distance from the listener to the sound
     * @param volume              the sound instance's volume (above 1 makes it carry further, as in vanilla)
     * @param attenuationDistance the sound's own fading distance in blocks (16 for most sounds)
     */
    public static boolean audible(final double distanceSq, final float volume, final int attenuationDistance) {
        final double reach = Math.max(volume, 1.0F) * (double) attenuationDistance * 1.25 + 2.0;
        return distanceSq <= reach * reach;
    }

    /**
     * Counts one start of {@code sound} and says whether it is still within {@code cap} starts of the
     * same sound in the current tick.
     */
    public static boolean allowSame(final Object sound, final int cap, final long nowNanos) {
        if (cap >= SAME_UNLIMITED || sound == null) {
            return true;
        }
        final long w = nowNanos / WINDOW_NANOS;
        if (w != window) {
            window = w;
            STARTED.clear();
        }
        int[] n = STARTED.get(sound);
        if (n == null) {
            n = new int[1];
            STARTED.put(sound, n);
        }
        return ++n[0] <= cap;
    }

    /** Called by the hook for every sound it skipped (statistics for the smoke test and the log). */
    public static void skipped(final boolean far) {
        if (far) {
            skippedFar++;
        } else {
            skippedSame++;
        }
    }

    public static String stats() {
        return skippedFar + " out of range, " + skippedSame + " over the per-tick limit";
    }

    /** Tests. */
    static void reset() {
        window = Long.MIN_VALUE;
        STARTED.clear();
        skippedFar = 0;
        skippedSame = 0;
    }
}
