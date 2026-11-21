package com.aetherium.perf;

/**
 * A sprite whose pixel copy in main memory can be freed once the graphics card has it
 * ({@code TuningTextureSpriteMixin}). After freeing, the sprite still answers "is this pixel
 * transparent" from a one-bit-per-pixel map made from the same pixels, which is all the game asks
 * of a static sprite's pixels after loading.
 */
public interface FreeableTexture {

    /** Frees the copy; returns the bytes freed, or 0 when this sprite keeps it (animated, already freed, the missing texture). */
    long aetheriumFreeCopies();
}
