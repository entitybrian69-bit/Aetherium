package com.aetherium.perf;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/** 1.19.0 performance features (every version). */
final class Tuning119LogicTest {

    @Test
    @DisplayName("Sound culling: audible within the sound's fading distance plus margin, louder carries further")
    void soundRange() {
        assertTrue(SoundBudget.audible(16.0 * 16.0, 1.0F, 16));
        assertTrue(SoundBudget.audible(21.0 * 21.0, 1.0F, 16), "margin past the fading distance");
        assertFalse(SoundBudget.audible(23.0 * 23.0, 1.0F, 16));
        assertTrue(SoundBudget.audible(60.0 * 60.0, 4.0F, 16), "volume 4 carries four times as far");
        assertTrue(SoundBudget.audible(20.0 * 20.0, 0.2F, 16), "quiet sounds still use the full distance, as vanilla");
    }

    @Test
    @DisplayName("Same sound limit: counts per sound within one tick window, unlimited at 33")
    void sameSound() {
        SoundBudget.reset();
        final long t = 10_000_000_000L;
        for (int i = 0; i < 4; i++) {
            assertTrue(SoundBudget.allowSame("a", 4, t));
        }
        assertFalse(SoundBudget.allowSame("a", 4, t), "fifth in the same tick");
        assertTrue(SoundBudget.allowSame("b", 4, t), "another sound has its own count");
        assertTrue(SoundBudget.allowSame("a", 4, t + 60_000_000L), "next tick starts over");
        for (int i = 0; i < 100; i++) {
            assertTrue(SoundBudget.allowSame("c", SoundBudget.SAME_UNLIMITED, t));
        }
    }

    @Test
    @DisplayName("Distant drops: one copy beyond 16 blocks, unchanged up close and outside a drop")
    void drops() {
        final boolean was = Tuning.distantDrops;
        Tuning.distantDrops = false;
        assertEquals(5, DropDetail.copies(5, 30.0 * 30.0), "off");
        Tuning.distantDrops = true;
        try {
            drops(true);
        } finally {
            Tuning.distantDrops = was;
        }
    }

    private static void drops(final boolean on) {
        assertTrue(on);
        assertEquals(5, DropDetail.copies(5, 15.0 * 15.0));
        assertEquals(1, DropDetail.copies(5, 17.0 * 17.0));
        assertEquals(1, DropDetail.copies(1, 40.0 * 40.0));
        assertEquals(4, DropDetail.copiesHere(4), "no drop being drawn");
        DropDetail.enter(30.0 * 30.0);
        assertEquals(1, DropDetail.copiesHere(4));
        DropDetail.leave();
        assertEquals(4, DropDetail.copiesHere(4));
    }

    @Test
    @DisplayName("Chunk unpacking: hand-over only at the exact offset, once per section")
    void unpack() {
        ChunkUnpack.reset();
        final byte[] data = new byte[100];
        final Object s0 = new Object();
        final Object s1 = new Object();
        ChunkUnpack.offer(data, new ChunkUnpack.Prepared(new int[] {0, 40}, new int[] {40, 100}, new Object[] {s0, s1}));
        assertNull(ChunkUnpack.take(new byte[100], 0), "another packet's bytes");
        assertNull(ChunkUnpack.take(data, 3), "not where a section starts");
        final ChunkUnpack.Taken first = ChunkUnpack.take(data, 0);
        assertNotNull(first);
        assertTrue(s0 == first.section);
        assertEquals(40, first.end);
        assertNull(ChunkUnpack.take(data, 0), "handed out once");
        final ChunkUnpack.Taken second = ChunkUnpack.take(data, 40);
        assertNotNull(second);
        assertTrue(s1 == second.section);
        assertEquals(100, second.end);
    }

    @Test
    @DisplayName("Text widths: cached per owner thread, emptied by a refresh")
    void textWidths() {
        final boolean was = Tuning.textWidthCache;
        try {
            Tuning.textWidthCache = true;
            TextWidths.setOwner(Thread.currentThread());
            final TextWidths widths = new TextWidths();
            assertTrue(TextWidths.usable("hello"));
            assertNull(widths.get("hello"));
            widths.put("hello", 24);
            assertEquals(Integer.valueOf(24), widths.get("hello"));
            for (int i = 0; i < TextWidths.REFRESH_TICKS; i++) {
                TextWidths.tick();
            }
            assertNull(widths.get("hello"), "refreshed once a second");
            final StringBuilder longText = new StringBuilder();
            for (int i = 0; i <= TextWidths.MAX_LENGTH; i++) {
                longText.append('x');
            }
            assertFalse(TextWidths.usable(longText.toString()));
            TextWidths.setOwner(null);
            assertFalse(TextWidths.usable("hello"), "other threads measure normally");
        } finally {
            Tuning.textWidthCache = was;
        }
    }

    @Test
    @DisplayName("Texture copies: transparency map bits, freed bytes over mip levels, quiet period")
    void textureCopies() {
        final long[] mask = TextureCopies.newMask(16, 16);
        assertEquals(4, mask.length);
        TextureCopies.set(mask, 16, 3, 5);
        TextureCopies.set(mask, 16, 15, 15);
        assertTrue(TextureCopies.bit(mask, 16, 3, 5));
        assertTrue(TextureCopies.bit(mask, 16, 15, 15));
        assertFalse(TextureCopies.bit(mask, 16, 4, 5));
        assertEquals(16 * 16 * 4 + 8 * 8 * 4 + 4 * 4 * 4, TextureCopies.bytes(16, 16, 3));
        assertTrue(TextureCopies.missing("minecraft:missingno"));
        assertFalse(TextureCopies.missing("minecraft:block/stone"));

        TextureCopies.reset();
        final boolean was = Tuning.freeTextureCopies;
        try {
            Tuning.freeTextureCopies = true;
            final int[] freed = new int[1];
            TextureCopies.queue((FreeableTexture) () -> {
                freed[0]++;
                return 1024L;
            });
            for (int i = 0; i < TextureCopies.QUIET_TICKS - 1; i++) {
                TextureCopies.tick();
            }
            assertEquals(0, freed[0], "still within the quiet period");
            TextureCopies.tick();
            assertEquals(1, freed[0]);
            TextureCopies.tick();
            assertEquals(1, freed[0], "freed once");
        } finally {
            Tuning.freeTextureCopies = was;
        }
    }
}
