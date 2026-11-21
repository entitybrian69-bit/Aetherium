package com.aetherium.mixin.core;

import com.aetherium.client.SoundCulling;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.SoundEngine;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
// @era:t-sound-begin v16
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
// @era:t-sound-else r216|r2111
//~ import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
// @era:t-sound-end

/**
 * Sound culling (1.19.0, every version): a short world sound that cannot be heard from where the
 * camera is, or that already started too often this tick, does not get an audio channel
 * ({@link SoundCulling}). The check sits right before the channel is requested, so vanilla's own
 * early exits, the subtitles and every other hook at the start of {@code play} are unchanged.
 */
@Mixin(SoundEngine.class)
public abstract class TuningSoundMixin {
    // @era:t-sound-begin v16
    @Inject(method = "play", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/client/sounds/ChannelAccess;createHandle(Lcom/mojang/blaze3d/audio/Library$Pool;)Ljava/util/concurrent/CompletableFuture;"),
            cancellable = true, require = 0)
    private void aetherium$cull(final SoundInstance sound, final CallbackInfo ci) {
        if (SoundCulling.skip(sound, sound.getLocation())) {
            ci.cancel();
        }
    }
    // @era:t-sound-else r216
    //~ @Inject(method = "play", at = @At(value = "INVOKE",
            //~ target = "Lnet/minecraft/client/sounds/ChannelAccess;createHandle(Lcom/mojang/blaze3d/audio/Library$Pool;)Ljava/util/concurrent/CompletableFuture;"),
            //~ cancellable = true, require = 0)
    //~ private void aetherium$cull(final SoundInstance sound, final CallbackInfoReturnable<SoundEngine.PlayResult> cir) {
        //~ if (SoundCulling.skip(sound, sound.getLocation())) {
            //~ cir.setReturnValue(SoundEngine.PlayResult.NOT_STARTED);
        //~ }
    //~ }
    // @era:t-sound-else r2111
    //~ @Inject(method = "play", at = @At(value = "INVOKE",
            //~ target = "Lnet/minecraft/client/sounds/ChannelAccess;createHandle(Lcom/mojang/blaze3d/audio/Library$Pool;)Ljava/util/concurrent/CompletableFuture;"),
            //~ cancellable = true, require = 0)
    //~ private void aetherium$cull(final SoundInstance sound, final CallbackInfoReturnable<SoundEngine.PlayResult> cir) {
        //~ if (SoundCulling.skip(sound, sound.getIdentifier())) {
            //~ cir.setReturnValue(SoundEngine.PlayResult.NOT_STARTED);
        //~ }
    //~ }
    // @era:t-sound-end
}
