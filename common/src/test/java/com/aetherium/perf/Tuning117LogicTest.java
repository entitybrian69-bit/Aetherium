package com.aetherium.perf;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.aetherium.config.AetheriumConfig;
import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/** 1.17.2 performance features: HUD refresh rate, crash guard. */
final class Tuning117LogicTest {

    @Test
    @DisplayName("HUD refresh rate: direct when off or slow, then refresh / reuse at the chosen rate")
    void hudRate() {
        HudRate.reset();
        long now = 1_000_000_000L;
        final long frame = 1_000_000_000L / 200; // 200 fps
        assertEquals(HudRate.DIRECT, HudRate.decide(now, 100, 100, 1L, false), "off");
        HudRate.setFps(30);
        assertEquals(30, HudRate.fps());
        int refresh = 0;
        int reuse = 0;
        for (int i = 0; i < 400; i++) {
            now += frame;
            final int d = HudRate.decide(now, 100, 100, 1L, false);
            if (d == HudRate.REFRESH) {
                refresh++;
            } else if (d == HudRate.REUSE) {
                reuse++;
            }
        }
        assertTrue(refresh >= 55 && refresh <= 62, "about 30 refreshes a second over 2 s: " + refresh);
        assertEquals(400 - refresh, reuse);
        now += frame;
        assertEquals(HudRate.REFRESH, HudRate.decide(now, 100, 100, 2L, false), "state change refreshes at once");
        now += frame;
        assertEquals(HudRate.REFRESH, HudRate.decide(now, 120, 100, 2L, false), "resize refreshes at once");
        now += frame;
        assertEquals(HudRate.DIRECT, HudRate.decide(now, 120, 100, 2L, true), "live HUD is drawn directly");
        HudRate.setFps(HudRate.MAX + 50);
        assertEquals(HudRate.MAX, HudRate.fps());
        HudRate.setCapturing(true);
        HudRate.reset();
        assertFalse(HudRate.capturing());
    }

    @Test
    @DisplayName("HUD refresh rate: picture alpha stays right for normal and additive blending")
    void hudAlpha() {
        assertEquals(HudRate.GL_ONE << 16 | HudRate.GL_ONE_MINUS_SRC_ALPHA,
                HudRate.alphaFactorsFor(HudRate.GL_ONE_MINUS_SRC_ALPHA));
        assertEquals(HudRate.GL_ZERO << 16 | HudRate.GL_ONE, HudRate.alphaFactorsFor(HudRate.GL_ONE));
    }

    @Test
    @DisplayName("pipeline crash guard: a crash while drawing packed terrain turns only packing off")
    void packedCrash() throws Exception {
        final Path dir = Files.createTempDirectory("aetherium-guard");
        final AetheriumConfig config = AetheriumConfig.createDefaults();
        config.aetheriumRenderer.set(Boolean.TRUE);
        config.packedTerrain.set(Boolean.TRUE);
        assertFalse(PipelineCrashGuard.checkAtStartup(dir, config));
        PipelineCrashGuard.arm(PipelineCrashGuard.MODE_PACKED);
        assertTrue(PipelineCrashGuard.checkAtStartup(dir, config));
        assertEquals(Boolean.FALSE, config.packedTerrain.get());
        assertEquals(Boolean.TRUE, config.regionBatching.get(), "batching stays on");
        assertEquals(Boolean.TRUE, config.aetheriumRenderer.get(), "the pipeline stays on");
        assertTrue(PipelineCrashGuard.wasPackedTurnedOff());
        assertFalse(PipelineCrashGuard.wasBatchingTurnedOff());

        // Packed already off: the next packed-mode crash turns batching off.
        PipelineCrashGuard.arm(PipelineCrashGuard.MODE_PACKED);
        assertTrue(PipelineCrashGuard.checkAtStartup(dir, config));
        assertEquals(Boolean.FALSE, config.regionBatching.get());
        assertTrue(PipelineCrashGuard.wasBatchingTurnedOff());
        assertFalse(PipelineCrashGuard.wasPackedTurnedOff());
        PipelineCrashGuard.disarm();
    }
}
