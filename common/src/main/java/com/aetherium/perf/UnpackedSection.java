package com.aetherium.perf;

/**
 * A chunk section that can take over the block (and biome) data of another section that was
 * unpacked from the same bytes ({@code TuningUnpackSectionMixin}). Background chunk unpacking
 * (1.19.0) hands sections over through this, so the section object vanilla created stays the one
 * in the chunk.
 */
public interface UnpackedSection {

    /** Takes over the counts and data containers of {@code from}, exactly as a read of the same bytes would leave them. */
    void aetheriumAdopt(UnpackedSection from);

    /** The block count read from the packet. */
    short aetheriumBlockCount();

    /** The fluid count read from the packet (0 before Minecraft 26.1, which has none). */
    short aetheriumFluidCount();

    /** The block state container. */
    Object aetheriumStates();

    /** The biome container (null before Minecraft 1.18, where biomes are not part of a section). */
    Object aetheriumBiomes();
}
