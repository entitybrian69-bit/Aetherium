package com.aetherium.perf;

/**
 * Simpler distant item drops (1.18.0): a dropped stack further away than {@link #DISTANCE} blocks is
 * drawn as one copy of its item instead of up to five stacked copies. At that distance the copies
 * cover each other within a pixel or two; up close nothing changes.
 */
public final class DropDetail {
    public static final double DISTANCE = 16.0;
    private static final double DISTANCE_SQ = DISTANCE * DISTANCE;

    /** Render thread: squared camera distance of the dropped stack being drawn, or -1 outside one. */
    private static double current = -1.0;

    private DropDetail() {
    }

    /** The stack about to be drawn is this far away (versions whose copy count is computed deeper down). */
    public static void enter(final double distanceSq) {
        current = distanceSq;
    }

    public static void leave() {
        current = -1.0;
    }

    /** {@link #copies} for the stack between {@link #enter} and {@link #leave}; anything else is unchanged. */
    public static int copiesHere(final int copies) {
        final double d = current;
        return d < 0.0 ? copies : copies(copies, d);
    }

    /** How many copies to draw for a stack vanilla would draw {@code copies} times. */
    public static int copies(final int copies, final double distanceSq) {
        return Tuning.distantDrops && copies > 1 && distanceSq > DISTANCE_SQ ? 1 : copies;
    }
}
