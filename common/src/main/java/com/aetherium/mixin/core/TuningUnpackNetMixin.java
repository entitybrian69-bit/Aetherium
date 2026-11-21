package com.aetherium.mixin.core;

import com.aetherium.client.ChunkUnpacking;
import net.minecraft.client.multiplayer.ClientPacketListener;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
// @era:t-unpack-net-begin d18|r263
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
// @era:t-unpack-net-else v16
//~ import net.minecraft.network.protocol.game.ClientboundLevelChunkPacket;
// @era:t-unpack-net-end

/**
 * Background chunk unpacking (1.19.0, every version), network thread: vanilla calls the chunk
 * packet handler there first (it then re-queues itself for the main thread), which is where the
 * packet's block data is handed to the background reader ({@link ChunkUnpacking#early}).
 */
@Mixin(ClientPacketListener.class)
public abstract class TuningUnpackNetMixin {
    // @era:t-unpack-net-begin d18
    @Inject(method = "handleLevelChunkWithLight", at = @At("HEAD"), require = 0)
    private void aetherium$early(final ClientboundLevelChunkWithLightPacket packet, final CallbackInfo ci) {
        ChunkUnpacking.early(packet.getChunkData().getReadBuffer());
    }
    // @era:t-unpack-net-else v16
    //~ @Inject(method = "handleLevelChunk", at = @At("HEAD"), require = 0)
    //~ private void aetherium$early(final ClientboundLevelChunkPacket packet, final CallbackInfo ci) {
        //~ ChunkUnpacking.early(packet.getReadBuffer());
    //~ }
    // @era:t-unpack-net-else r263
    //~ @Inject(method = "handleLevelChunkWithLight", at = @At("HEAD"), require = 0)
    //~ private void aetherium$early(final ClientboundLevelChunkWithLightPacket packet, final CallbackInfo ci) {
        //~ ChunkUnpacking.early(packet.chunkData().getReadBuffer());
    //~ }
    // @era:t-unpack-net-end
}
