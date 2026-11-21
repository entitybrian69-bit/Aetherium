package com.aetherium.perf;

import java.util.ArrayList;
import java.util.List;

/**
 * Free texture copies (1.19.0, every version). Every texture atlas that finishes uploading queues
 * its sprites here ({@code TuningTextureAtlasMixin}); once no atlas has uploaded for two seconds
 * (the resource reload is over and every model is built), the client tick frees the main-memory
 * pixel copies of the sprites that never change ({@link FreeableTexture}).
 */
public final class TextureCopies {
    /** Client ticks without a new atlas upload before the queue is freed. */
    static final int QUIET_TICKS = 40;

    private static final List<FreeableTexture> PENDING = new ArrayList<FreeableTexture>();
    private static int quiet;
    private static long freedBytes;
    private static int freedSprites;

    private TextureCopies() {
    }

    /** Render thread, right after an atlas uploaded: one of its sprites. */
    public static void queue(final Object sprite) {
        if (sprite instanceof FreeableTexture) {
            synchronized (PENDING) {
                PENDING.add((FreeableTexture) sprite);
                quiet = 0;
            }
        }
    }

    /** Client tick. */
    public static void tick() {
        final List<FreeableTexture> now;
        synchronized (PENDING) {
            if (PENDING.isEmpty() || ++quiet < QUIET_TICKS) {
                return;
            }
            now = new ArrayList<FreeableTexture>(PENDING);
            PENDING.clear();
        }
        if (!Tuning.freeTextureCopies) {
            return; // read at reload time: switching it on applies at the next reload
        }
        for (final FreeableTexture sprite : now) {
            try {
                final long bytes = sprite.aetheriumFreeCopies();
                if (bytes > 0) {
                    freedBytes += bytes;
                    freedSprites++;
                }
            } catch (final Throwable ignored) {
                // a sprite that cannot be freed simply keeps its copy
            }
        }
    }

    /** Bytes of a sprite's pixel copies: every mip level, four bytes a pixel. */
    public static long bytes(final int width, final int height, final int levels) {
        long sum = 0;
        for (int level = 0; level < levels; level++) {
            sum += (long) Math.max(1, width >> level) * Math.max(1, height >> level) * 4L;
        }
        return sum;
    }

    /** One bit per pixel, row by row: set where the pixel is fully transparent. */
    public static boolean bit(final long[] mask, final int width, final int x, final int y) {
        final int i = y * width + x;
        return (mask[i >>> 6] >>> (i & 63) & 1L) != 0L;
    }

    public static long[] newMask(final int width, final int height) {
        return new long[(width * height + 63) >>> 6];
    }

    public static void set(final long[] mask, final int width, final int x, final int y) {
        final int i = y * width + x;
        mask[i >>> 6] |= 1L << (i & 63);
    }

    /** The missing-texture sprite shares its pixels with the next reload on some versions: never freed. */
    public static boolean missing(final Object name) {
        return "minecraft:missingno".equals(String.valueOf(name));
    }

    public static String stats() {
        return "textures freed " + freedSprites + " (" + (freedBytes >> 10) + " KiB)";
    }

    static void reset() {
        synchronized (PENDING) {
            PENDING.clear();
            quiet = 0;
        }
        freedBytes = 0;
        freedSprites = 0;
    }
}
