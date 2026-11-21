package com.aetherium.client;

import com.aetherium.perf.SoundBudget;
import com.aetherium.perf.Tuning;
import net.minecraft.client.Camera;
import net.minecraft.client.Minecraft;
import net.minecraft.client.resources.sounds.Sound;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.resources.sounds.TickableSoundInstance;
import net.minecraft.world.phys.Vec3;

/**
 * Sound culling (1.19.0, every version): the Minecraft side of {@link SoundBudget}. Called from
 * {@code SoundEngine.play} right before the sound would get an audio channel, after vanilla has
 * resolved it and told the subtitle overlay about it.
 */
public final class SoundCulling {
    private SoundCulling() {
    }

    /**
     * True when this sound should not get a channel.
     *
     * @param event the sound event's id (counts the same sound across its variants)
     */
    public static boolean skip(final SoundInstance instance, final Object event) {
        if (!Tuning.soundCulling && Tuning.sameSoundLimit >= SoundBudget.SAME_UNLIMITED) {
            return false;
        }
        final Sound sound = instance.getSound();
        // Only one-shot world sounds: positional, fading with distance, short, not looping and
        // not following anything. Everything else always plays.
        if (sound == null || instance.isRelative() || instance.isLooping() || sound.shouldStream()
                || instance instanceof TickableSoundInstance
                || instance.getAttenuation() != SoundInstance.Attenuation.LINEAR) {
            return false;
        }
        if (Tuning.soundCulling) {
            final Camera camera = ClientHooks.mainCamera(Minecraft.getInstance());
            if (camera != null) {
                final Vec3 eye = ClientHooks.cameraPosition(camera);
                final double dx = instance.getX() - eye.x;
                final double dy = instance.getY() - eye.y;
                final double dz = instance.getZ() - eye.z;
                if (!SoundBudget.audible(dx * dx + dy * dy + dz * dz, instance.getVolume(), sound.getAttenuationDistance())) {
                    SoundBudget.skipped(true);
                    return true;
                }
            }
        }
        if (!SoundBudget.allowSame(event, Tuning.sameSoundLimit, System.nanoTime())) {
            SoundBudget.skipped(false);
            return true;
        }
        return false;
    }
}
