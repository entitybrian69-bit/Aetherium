package com.aetherium.mixin.core;

import com.aetherium.perf.UnpackedSection;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainer;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
// @era:t-unpack-sec-begin b19|f261
import net.minecraft.world.level.chunk.PalettedContainerRO;
// @era:t-unpack-sec-else v16|b18
// @era:t-unpack-sec-end

/**
 * Background chunk unpacking (1.19.0, every version): lets a chunk section take over what a
 * scratch section read from the same packet bytes on a background thread ({@link UnpackedSection}).
 * The containers are moved whole, so each keeps its own palette and resize callbacks.
 *
 * <p>What {@code LevelChunkSection.read} sets, per version: the block count and the block states;
 * 1.18+ also the biomes; 26.1+ also the fluid count. Nothing else is touched by a read, so nothing
 * else is copied.</p>
 */
@SuppressWarnings({"rawtypes", "unchecked"})
@Mixin(LevelChunkSection.class)
public abstract class TuningUnpackSectionMixin implements UnpackedSection {
    @Shadow
    private short nonEmptyBlockCount;
    @Shadow
    @Final
    @Mutable
    private PalettedContainer states;
    // @era:t-unpack-sec-begin b19
    @Shadow
    private PalettedContainerRO biomes;

    @Override
    public void aetheriumAdopt(final UnpackedSection from) {
        this.nonEmptyBlockCount = from.aetheriumBlockCount();
        this.states = (PalettedContainer) from.aetheriumStates();
        this.biomes = (PalettedContainerRO) from.aetheriumBiomes();
    }

    @Override
    public short aetheriumFluidCount() {
        return 0;
    }
    // @era:t-unpack-sec-else v16
    //~ @Override
    //~ public void aetheriumAdopt(final UnpackedSection from) {
        //~ this.nonEmptyBlockCount = from.aetheriumBlockCount();
        //~ this.states = (PalettedContainer) from.aetheriumStates();
    //~ }

    //~ @Override
    //~ public short aetheriumFluidCount() {
        //~ return 0;
    //~ }

    //~ @Override
    //~ public Object aetheriumBiomes() {
        //~ return null;
    //~ }
    // @era:t-unpack-sec-else b18
    //~ @Shadow
    //~ @Final
    //~ @Mutable
    //~ private PalettedContainer biomes;

    //~ @Override
    //~ public void aetheriumAdopt(final UnpackedSection from) {
        //~ this.nonEmptyBlockCount = from.aetheriumBlockCount();
        //~ this.states = (PalettedContainer) from.aetheriumStates();
        //~ this.biomes = (PalettedContainer) from.aetheriumBiomes();
    //~ }

    //~ @Override
    //~ public short aetheriumFluidCount() {
        //~ return 0;
    //~ }
    // @era:t-unpack-sec-else f261
    //~ @Shadow
    //~ private PalettedContainerRO biomes;
    //~ @Shadow
    //~ private short fluidCount;

    //~ @Override
    //~ public void aetheriumAdopt(final UnpackedSection from) {
        //~ this.nonEmptyBlockCount = from.aetheriumBlockCount();
        //~ this.fluidCount = from.aetheriumFluidCount();
        //~ this.states = (PalettedContainer) from.aetheriumStates();
        //~ this.biomes = (PalettedContainerRO) from.aetheriumBiomes();
    //~ }

    //~ @Override
    //~ public short aetheriumFluidCount() {
        //~ return this.fluidCount;
    //~ }
    // @era:t-unpack-sec-end

    @Override
    public short aetheriumBlockCount() {
        return this.nonEmptyBlockCount;
    }

    @Override
    public Object aetheriumStates() {
        return this.states;
    }

    // @era:t-unpack-sec-begin b19|b18|f261
    @Override
    public Object aetheriumBiomes() {
        return this.biomes;
    }
    // @era:t-unpack-sec-else v16
    // @era:t-unpack-sec-end
}
