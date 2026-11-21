package com.aetherium.perf;

/**
 * Cached inventory icons (1.18.0), the pixel arithmetic: every icon is drawn twice by the game's
 * own item renderer, once over black and once over white. Whatever blending the item used (solid,
 * cut-out or see-through), "over black" is its colour times its coverage and "over white" adds the
 * uncovered part, so the pair gives back the exact colour and coverage of every pixel. Drawing the
 * stored pixel with normal alpha blending then gives the same result as drawing the item itself.
 */
public final class IconPixels {
    private IconPixels() {
    }

    /**
     * @param black ABGR or ARGB pixel drawn over black (only the three colour bytes are used)
     * @param white the same pixel drawn over white
     * @return the pixel with straight (not premultiplied) colour and recovered alpha, same byte order,
     *         alpha in the top byte
     */
    public static int combine(final int black, final int white) {
        final int b0 = black & 0xFF;
        final int b1 = black >>> 8 & 0xFF;
        final int b2 = black >>> 16 & 0xFF;
        final int w0 = white & 0xFF;
        final int w1 = white >>> 8 & 0xFF;
        final int w2 = white >>> 16 & 0xFF;
        // uncovered = white - black per channel (equal in theory; average the three against rounding)
        int uncovered = (Math.max(0, w0 - b0) + Math.max(0, w1 - b1) + Math.max(0, w2 - b2) + 1) / 3;
        if (uncovered > 255) {
            uncovered = 255;
        }
        final int alpha = 255 - uncovered;
        if (alpha <= 0) {
            return 0;
        }
        if (alpha >= 255) {
            return 0xFF000000 | b2 << 16 | b1 << 8 | b0;
        }
        return alpha << 24 | unpremultiply(b2, alpha) << 16 | unpremultiply(b1, alpha) << 8 | unpremultiply(b0, alpha);
    }

    private static int unpremultiply(final int c, final int alpha) {
        final int v = (c * 255 + alpha / 2) / alpha;
        return v > 255 ? 255 : v;
    }

    /** Largest per-channel difference between two pixels (the self-check's comparison). */
    public static int difference(final int a, final int b) {
        int max = 0;
        for (int shift = 0; shift < 32; shift += 8) {
            final int d = Math.abs((a >>> shift & 0xFF) - (b >>> shift & 0xFF));
            if (d > max) {
                max = d;
            }
        }
        return max;
    }
}
