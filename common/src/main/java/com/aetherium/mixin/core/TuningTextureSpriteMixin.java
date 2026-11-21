package com.aetherium.mixin.core;

import com.aetherium.perf.FreeableTexture;
import com.aetherium.perf.TextureCopies;
import com.mojang.blaze3d.platform.NativeImage;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
// @era:t-tex-sprite-begin c193|c2111
import net.minecraft.client.renderer.texture.SpriteContents;
// @era:t-tex-sprite-else v16|s17
//~ import net.minecraft.client.renderer.texture.TextureAtlasSprite;
// @era:t-tex-sprite-else f261
//~ import com.mojang.blaze3d.platform.Transparency;
//~ import net.minecraft.client.renderer.texture.SpriteContents;
// @era:t-tex-sprite-end

/**
 * Free texture copies (1.19.0, every version): a sprite's pixels stay in main memory after they
 * were sent to the graphics card. For sprites that never change (not animated), this frees that
 * copy ({@link TextureCopies}) and keeps a one-bit-per-pixel transparency map made from vanilla's
 * own {@code isTransparent}, which then answers in its place. The class that owns the pixels is
 * the sprite up to 1.19.2 and its contents from 1.19.3.
 */
// @era:t-tex-sprite-begin c193|c2111
@Mixin(SpriteContents.class)
// @era:t-tex-sprite-else v16|s17
//~ @Mixin(TextureAtlasSprite.class)
// @era:t-tex-sprite-else f261
//~ @Mixin(SpriteContents.class)
// @era:t-tex-sprite-end
public abstract class TuningTextureSpriteMixin implements FreeableTexture {
    @Unique
    private long[] aetherium$transparent;
    @Unique
    private int aetherium$width;
    @Unique
    private int aetherium$height;

    // @era:t-tex-sprite-begin c193
    @Shadow
    @Final
    private NativeImage originalImage;
    @Shadow
    NativeImage[] byMipLevel;

    @Shadow
    private int getFrameCount() {
        throw new AssertionError();
    }

    @Shadow
    public abstract int width();

    @Shadow
    public abstract int height();

    @Shadow
    public abstract net.minecraft.resources.ResourceLocation name();
    // @era:t-tex-sprite-else v16
    //~ @Shadow
    //~ @Final
    //~ protected NativeImage[] mainImage;

    //~ @Shadow
    //~ public abstract int getFrameCount();

    //~ @Shadow
    //~ public abstract int getWidth();

    //~ @Shadow
    //~ public abstract int getHeight();

    //~ @Shadow
    //~ public abstract net.minecraft.resources.ResourceLocation getName();
    // @era:t-tex-sprite-else s17
    //~ @Shadow
    //~ @Final
    //~ protected NativeImage[] mainImage;

    //~ @Shadow
    //~ private int getFrameCount() {
        //~ throw new AssertionError();
    //~ }

    //~ @Shadow
    //~ public abstract int getWidth();

    //~ @Shadow
    //~ public abstract int getHeight();

    //~ @Shadow
    //~ public abstract net.minecraft.resources.ResourceLocation getName();
    // @era:t-tex-sprite-else c2111
    //~ @Shadow
    //~ @Final
    //~ private NativeImage originalImage;
    //~ @Shadow
    //~ NativeImage[] byMipLevel;

    //~ @Shadow
    //~ private int getFrameCount() {
        //~ throw new AssertionError();
    //~ }

    //~ @Shadow
    //~ public abstract int width();

    //~ @Shadow
    //~ public abstract int height();

    //~ @Shadow
    //~ public abstract net.minecraft.resources.Identifier name();
    // @era:t-tex-sprite-else f261
    //~ @Shadow
    //~ @Final
    //~ private NativeImage originalImage;
    //~ @Shadow
    //~ private NativeImage[] byMipLevel;

    //~ @Shadow
    //~ private int getFrameCount() {
        //~ throw new AssertionError();
    //~ }

    //~ @Shadow
    //~ public abstract int width();

    //~ @Shadow
    //~ public abstract int height();

    //~ @Shadow
    //~ public abstract net.minecraft.resources.Identifier name();

    //~ @Shadow
    //~ public abstract Transparency transparency();

    //~ /** Asked of a part of the sprite; once the pixels are gone, the whole sprite's (precomputed) answer stands in. */
    //~ @Inject(method = "computeTransparency(FFFF)Lcom/mojang/blaze3d/platform/Transparency;", at = @At("HEAD"),
            //~ cancellable = true, require = 0)
    //~ private void aetherium$freedTransparency(final float u0, final float v0, final float u1, final float v1,
                                             //~ final CallbackInfoReturnable<Transparency> cir) {
        //~ if (this.aetherium$transparent != null) {
            //~ cir.setReturnValue(this.transparency());
        //~ }
    //~ }
    // @era:t-tex-sprite-end

    @Shadow
    public abstract boolean isTransparent(int frame, int x, int y);

    @Inject(method = "isTransparent(III)Z", at = @At("HEAD"), cancellable = true, require = 0)
    private void aetherium$freedPixels(final int frame, final int x, final int y, final CallbackInfoReturnable<Boolean> cir) {
        final long[] mask = this.aetherium$transparent;
        if (mask != null && x >= 0 && y >= 0 && x < this.aetherium$width && y < this.aetherium$height) {
            cir.setReturnValue(Boolean.valueOf(TextureCopies.bit(mask, this.aetherium$width, x, y)));
        }
    }

    @Override
    public long aetheriumFreeCopies() {
        // @era:t-tex-sprite-begin c193|c2111|f261
        final NativeImage[] images = this.byMipLevel;
        final int width = this.width();
        final int height = this.height();
        final Object name = this.name();
        // @era:t-tex-sprite-else v16|s17
        //~ final NativeImage[] images = this.mainImage;
        //~ final int width = this.getWidth();
        //~ final int height = this.getHeight();
        //~ final Object name = this.getName();
        // @era:t-tex-sprite-end
        if (this.aetherium$transparent != null || images == null || images.length == 0 || this.getFrameCount() > 1
                || TextureCopies.missing(name) || width <= 0 || height <= 0) {
            return 0L;
        }
        final long[] mask = TextureCopies.newMask(width, height);
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                if (this.isTransparent(0, x, y)) {
                    TextureCopies.set(mask, width, x, y);
                }
            }
        }
        this.aetherium$width = width;
        this.aetherium$height = height;
        this.aetherium$transparent = mask;
        for (final NativeImage image : images) {
            if (image != null) {
                image.close();
            }
        }
        // @era:t-tex-sprite-begin c193|c2111|f261
        if (this.originalImage != null) {
            this.originalImage.close();
        }
        // @era:t-tex-sprite-else v16|s17
        // @era:t-tex-sprite-end
        return TextureCopies.bytes(width, height, images.length);
    }
}
