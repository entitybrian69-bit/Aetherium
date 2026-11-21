package com.aetherium.mixin.core;

import com.aetherium.client.ChunkUnpacking;
import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.LevelChunkSection;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/**
 * Background chunk unpacking (1.19.0, every version), main thread: each section read while a
 * chunk packet is applied first asks for the section prepared from the same bytes
 * ({@link ChunkUnpacking#take}); only when there is none does vanilla read it.
 */
@Mixin(LevelChunk.class)
public abstract class TuningUnpackChunkMixin {
    @WrapOperation(method = "replaceWithPacketData", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/level/chunk/LevelChunkSection;read(Lnet/minecraft/network/FriendlyByteBuf;)V"),
            require = 0)
    private void aetherium$prepared(final LevelChunkSection section, final FriendlyByteBuf buffer,
                                    final Operation<Void> original) {
        if (!ChunkUnpacking.take(section, buffer)) {
            original.call(section, buffer);
        }
    }
}
