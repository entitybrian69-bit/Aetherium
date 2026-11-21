package com.aetherium.mixin.core;

import com.aetherium.perf.TextWidths;
import net.minecraft.client.StringSplitter;
import net.minecraft.client.gui.Font;
import net.minecraft.util.Mth;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Text measurement cache (1.19.0, every version): {@code Font.width(String)} answers from a small
 * per-font cache of recently measured strings ({@link TextWidths}). A miss measures exactly like
 * vanilla ({@code ceil(splitter.stringWidth(text))}, the same body on every version).
 */
@Mixin(Font.class)
public abstract class TuningTextWidthMixin {
    @Shadow
    @Final
    private StringSplitter splitter;

    @Unique
    private final TextWidths aetherium$widths = new TextWidths();

    @Inject(method = "width(Ljava/lang/String;)I", at = @At("HEAD"), cancellable = true, require = 0)
    private void aetherium$cachedWidth(final String text, final CallbackInfoReturnable<Integer> cir) {
        if (TextWidths.usable(text)) {
            final Integer known = this.aetherium$widths.get(text);
            if (known != null) {
                cir.setReturnValue(known);
                return;
            }
            final int width = Mth.ceil(this.splitter.stringWidth(text));
            this.aetherium$widths.put(text, width);
            cir.setReturnValue(Integer.valueOf(width));
        }
    }
}
