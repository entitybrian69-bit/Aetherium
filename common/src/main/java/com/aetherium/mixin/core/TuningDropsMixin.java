package com.aetherium.mixin.core;

import com.aetherium.perf.DropDetail;
import net.minecraft.client.renderer.entity.ItemEntityRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
// @era:t-drops-begin c205
import com.aetherium.client.ClientHooks;
import com.llamalad7.mixinextras.injector.ModifyReturnValue;
import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.item.ItemEntity;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
// @era:t-drops-else v16
//~ import com.aetherium.client.ClientHooks;
//~ import com.llamalad7.mixinextras.injector.ModifyExpressionValue;
//~ import com.llamalad7.mixinextras.sugar.Local;
//~ import net.minecraft.client.Minecraft;
//~ import net.minecraft.world.entity.item.ItemEntity;
// @era:t-drops-else c212
//~ import com.llamalad7.mixinextras.injector.ModifyReturnValue;
//~ import net.minecraft.client.renderer.entity.state.ItemEntityRenderState;
//~ import org.spongepowered.asm.mixin.injection.Inject;
//~ import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
// @era:t-drops-else s214
//~ import net.minecraft.client.renderer.entity.state.ItemEntityRenderState;
//~ import net.minecraft.world.entity.item.ItemEntity;
//~ import org.spongepowered.asm.mixin.injection.Inject;
//~ import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
// @era:t-drops-end

/**
 * Simpler distant item drops (1.19.0, every version): a dropped stack further than 16 blocks from
 * the camera is drawn as one copy of its item instead of up to five ({@link DropDetail}).
 *
 * <ul>
 *   <li>up to 1.20.4 the copy count comes from {@code getRenderAmount} inside {@code render};</li>
 *   <li>1.20.5 - 1.21.3 from the static {@code getRenderedAmount}, which is also used by other
 *       renderers, so only calls made while a dropped item is being drawn are changed;</li>
 *   <li>1.21.4 and newer store it in the render state ({@code count}).</li>
 * </ul>
 */
@Mixin(ItemEntityRenderer.class)
public abstract class TuningDropsMixin {
    // @era:t-drops-begin c205
    @Inject(method = "render(Lnet/minecraft/world/entity/item/ItemEntity;FFLcom/mojang/blaze3d/vertex/PoseStack;Lnet/minecraft/client/renderer/MultiBufferSource;I)V",
            at = @At("HEAD"), require = 0)
    private void aetherium$enter(final ItemEntity entity, final float yaw, final float partialTick,
                                 final com.mojang.blaze3d.vertex.PoseStack pose,
                                 final net.minecraft.client.renderer.MultiBufferSource buffers, final int light,
                                 final CallbackInfo ci) {
        final net.minecraft.client.Camera camera = ClientHooks.mainCamera(Minecraft.getInstance());
        DropDetail.enter(camera == null ? -1.0 : entity.distanceToSqr(ClientHooks.cameraPosition(camera)));
    }

    @Inject(method = "render(Lnet/minecraft/world/entity/item/ItemEntity;FFLcom/mojang/blaze3d/vertex/PoseStack;Lnet/minecraft/client/renderer/MultiBufferSource;I)V",
            at = @At("RETURN"), require = 0)
    private void aetherium$leave(final ItemEntity entity, final float yaw, final float partialTick,
                                 final com.mojang.blaze3d.vertex.PoseStack pose,
                                 final net.minecraft.client.renderer.MultiBufferSource buffers, final int light,
                                 final CallbackInfo ci) {
        DropDetail.leave();
    }

    @ModifyReturnValue(method = "getRenderedAmount(I)I", at = @At("RETURN"), require = 0)
    private static int aetherium$copies(final int copies) {
        return DropDetail.copiesHere(copies);
    }
    // @era:t-drops-else v16
    //~ @ModifyExpressionValue(method = "render(Lnet/minecraft/world/entity/item/ItemEntity;FFLcom/mojang/blaze3d/vertex/PoseStack;Lnet/minecraft/client/renderer/MultiBufferSource;I)V",
            //~ at = @At(value = "INVOKE",
                    //~ target = "Lnet/minecraft/client/renderer/entity/ItemEntityRenderer;getRenderAmount(Lnet/minecraft/world/item/ItemStack;)I"),
            //~ require = 0)
    //~ private int aetherium$copies(final int copies, @Local(argsOnly = true) final ItemEntity entity) {
        //~ final net.minecraft.client.Camera camera = ClientHooks.mainCamera(Minecraft.getInstance());
        //~ return camera == null ? copies : DropDetail.copies(copies, entity.distanceToSqr(ClientHooks.cameraPosition(camera)));
    //~ }
    // @era:t-drops-else c212
    //~ @Inject(method = "render(Lnet/minecraft/client/renderer/entity/state/ItemEntityRenderState;Lcom/mojang/blaze3d/vertex/PoseStack;Lnet/minecraft/client/renderer/MultiBufferSource;I)V",
            //~ at = @At("HEAD"), require = 0)
    //~ private void aetherium$enter(final ItemEntityRenderState state, final com.mojang.blaze3d.vertex.PoseStack pose,
                                 //~ final net.minecraft.client.renderer.MultiBufferSource buffers, final int light,
                                 //~ final CallbackInfo ci) {
        //~ DropDetail.enter(state.distanceToCameraSq);
    //~ }

    //~ @Inject(method = "render(Lnet/minecraft/client/renderer/entity/state/ItemEntityRenderState;Lcom/mojang/blaze3d/vertex/PoseStack;Lnet/minecraft/client/renderer/MultiBufferSource;I)V",
            //~ at = @At("RETURN"), require = 0)
    //~ private void aetherium$leave(final ItemEntityRenderState state, final com.mojang.blaze3d.vertex.PoseStack pose,
                                 //~ final net.minecraft.client.renderer.MultiBufferSource buffers, final int light,
                                 //~ final CallbackInfo ci) {
        //~ DropDetail.leave();
    //~ }

    //~ @ModifyReturnValue(method = "getRenderedAmount(I)I", at = @At("RETURN"), require = 0)
    //~ private static int aetherium$copies(final int copies) {
        //~ return DropDetail.copiesHere(copies);
    //~ }
    // @era:t-drops-else s214
    //~ @Inject(method = "extractRenderState(Lnet/minecraft/world/entity/item/ItemEntity;Lnet/minecraft/client/renderer/entity/state/ItemEntityRenderState;F)V",
            //~ at = @At("TAIL"), require = 0)
    //~ private void aetherium$copies(final ItemEntity entity, final ItemEntityRenderState state, final float partialTick,
                                  //~ final CallbackInfo ci) {
        //~ state.count = DropDetail.copies(state.count, state.distanceToCameraSq);
    //~ }
    // @era:t-drops-end
}
