package com.aetherium.mixin.core;

import java.util.Map;

import com.aetherium.perf.TextureCopies;
import com.aetherium.perf.Tuning;
import net.minecraft.client.renderer.texture.TextureAtlas;
import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
// @era:t-tex-atlas-begin c193
import net.minecraft.client.renderer.texture.SpriteLoader;
// @era:t-tex-atlas-else v16
//~ import org.spongepowered.asm.mixin.Final;
// @era:t-tex-atlas-end

/**
 * Free texture copies (1.19.0, every version): when an atlas has sent its sprites to the
 * graphics card, they are queued to have their main-memory copies freed once loading is over
 * ({@link TextureCopies}).
 */
@Mixin(TextureAtlas.class)
public abstract class TuningTextureAtlasMixin {
    // @era:t-tex-atlas-begin c193
    @Shadow
    private Map<?, TextureAtlasSprite> texturesByName;

    @Inject(method = "upload", at = @At("TAIL"), require = 0)
    private void aetherium$queueCopies(final SpriteLoader.Preparations preparations, final CallbackInfo ci) {
        if (Tuning.freeTextureCopies) {
            for (final TextureAtlasSprite sprite : this.texturesByName.values()) {
                TextureCopies.queue(sprite.contents());
            }
        }
    }
    // @era:t-tex-atlas-else v16
    //~ @Shadow
    //~ @Final
    //~ private Map<?, TextureAtlasSprite> texturesByName;

    //~ @Inject(method = "reload", at = @At("TAIL"), require = 0)
    //~ private void aetherium$queueCopies(final TextureAtlas.Preparations preparations, final CallbackInfo ci) {
        //~ if (Tuning.freeTextureCopies) {
            //~ for (final TextureAtlasSprite sprite : this.texturesByName.values()) {
                //~ TextureCopies.queue(sprite);
            //~ }
        //~ }
    //~ }
    // @era:t-tex-atlas-end
}
