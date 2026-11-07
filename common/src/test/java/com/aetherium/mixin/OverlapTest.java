package com.aetherium.mixin;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.HashSet;
import java.util.Set;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/** Stepping aside when another mod changes the same method. */
final class OverlapTest {

    @Test
    @DisplayName("selectors reduce to the bare method name")
    void methodNames() {
        assertEquals("method_26227", Overlap.methodName(
                "Lnet/minecraft/class_4970$class_4971;method_26227()Lnet/minecraft/class_3610;"));
        assertEquals("render", Overlap.methodName("render(FJZ)V"));
        assertEquals("populateNeighbours", Overlap.methodName("populateNeighbours"));
        assertEquals("<init>", Overlap.methodName("<init>"));
        assertEquals("tick", Overlap.methodName("tick*"));
        assertEquals("decode", Overlap.methodName(" decode{2} "));
        assertNull(Overlap.methodName("*"));
        assertNull(Overlap.methodName(""));
        assertNull(Overlap.methodName(null));
    }

    @Test
    @DisplayName("without the mixin library's internals nothing steps aside and targets stay")
    void noLibrary() {
        final Set<String> targets = new HashSet<String>();
        targets.add("net.minecraft.world.level.block.state.StateHolder");
        Overlap.resolve(targets);
        assertTrue(targets.contains("net.minecraft.world.level.block.state.StateHolder"));
        assertFalse(Overlap.yielded("fast_network"));
        assertTrue(Overlap.yieldedOptions().isEmpty());
        assertTrue(Overlap.notes().isEmpty());
        assertFalse(Overlap.yielded(null));
    }
}
