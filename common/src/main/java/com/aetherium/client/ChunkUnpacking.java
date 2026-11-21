package com.aetherium.client;

import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;

import com.aetherium.perf.ChunkUnpack;
import com.aetherium.perf.Tuning;
import com.aetherium.perf.UnpackedSection;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.world.level.chunk.LevelChunkSection;
// @era:t-unpack-make-begin r20|r193
import net.minecraft.core.registries.Registries;
// @era:t-unpack-make-else v16|f219
// @era:t-unpack-make-else r18
//~ import net.minecraft.core.Registry;
// @era:t-unpack-make-else l212
//~ import net.minecraft.core.registries.Registries;
// @era:t-unpack-make-end

/**
 * Background chunk unpacking (1.19.0, every version), the Minecraft side of {@link ChunkUnpack}.
 *
 * <p>When a chunk packet reaches the client's network thread (vanilla handles it there first and
 * then hands it to the main thread), {@link #early} queues its block data for one background
 * thread, which reads every section from the packet's bytes with vanilla's own
 * {@code LevelChunkSection.read} into scratch sections. When the main thread applies the packet,
 * {@link #take} hands each prepared section over at exactly the byte offset it was read from.
 * Nothing is prepared when the queue is full, no world exists yet, or reading fails; the main
 * thread then reads those bytes itself, as vanilla does.</p>
 */
public final class ChunkUnpacking {
    /** Sections in one packet at most (the tallest worlds have 256). */
    private static final int MAX_SECTIONS = 256;

    private static final ThreadPoolExecutor WORKER = new ThreadPoolExecutor(1, 1, 30L, TimeUnit.SECONDS,
            new ArrayBlockingQueue<Runnable>(512), runnable -> {
                final Thread thread = new Thread(runnable, "Aetherium chunk unpacking");
                thread.setDaemon(true);
                thread.setPriority(Thread.NORM_PRIORITY - 1);
                return thread;
            }, new ThreadPoolExecutor.DiscardPolicy());

    static {
        WORKER.allowCoreThreadTimeOut(true);
    }

    private ChunkUnpacking() {
    }

    /** Network thread: a chunk packet with this block data arrived (the buffer is a fresh view over the packet's bytes). */
    public static void early(final FriendlyByteBuf data) {
        if (!Tuning.chunkUnpacking || data == null || !data.hasArray()) {
            return;
        }
        final Minecraft mc = Minecraft.getInstance();
        if (mc.isSameThread()) {
            return; // already on the main thread: nothing to win
        }
        final ClientLevel level = mc.level;
        if (level == null) {
            return;
        }
        WORKER.execute(() -> unpack(level, data));
    }

    private static void unpack(final ClientLevel level, final FriendlyByteBuf in) {
        try {
            final byte[] bytes = in.array();
            final int base = in.arrayOffset();
            final int[] start = new int[MAX_SECTIONS];
            final int[] end = new int[MAX_SECTIONS];
            final Object[] sections = new Object[MAX_SECTIONS];
            int n = 0;
            while (in.readableBytes() > 0 && n < MAX_SECTIONS) {
                final int at = base + in.readerIndex();
                final LevelChunkSection section = scratch(level);
                section.read(in);
                start[n] = at;
                end[n] = base + in.readerIndex();
                sections[n] = section;
                n++;
            }
            if (n == 0 || in.readableBytes() > 0) {
                return; // not the layout we know: let the main thread read it
            }
            ChunkUnpack.offer(bytes, new ChunkUnpack.Prepared(java.util.Arrays.copyOf(start, n),
                    java.util.Arrays.copyOf(end, n), java.util.Arrays.copyOf(sections, n)));
        } catch (final Throwable error) {
            ChunkUnpack.failed(error);
        }
    }

    /**
     * Main thread, in place of {@code section.read(buffer)}: true when the section took over the
     * data prepared from exactly these bytes and the buffer now stands where the read would end.
     */
    public static boolean take(final Object section, final FriendlyByteBuf buffer) {
        if (!Tuning.chunkUnpacking || !(section instanceof UnpackedSection) || !buffer.hasArray()) {
            return false;
        }
        final int base = buffer.arrayOffset();
        final ChunkUnpack.Taken taken = ChunkUnpack.take(buffer.array(), base + buffer.readerIndex());
        if (taken == null || !(taken.section instanceof UnpackedSection) || taken.end - base > buffer.writerIndex()) {
            return false;
        }
        ((UnpackedSection) section).aetheriumAdopt((UnpackedSection) taken.section);
        buffer.readerIndex(taken.end - base);
        return true;
    }

    /** An empty section built the way the client's chunks build theirs on this version. */
    private static LevelChunkSection scratch(final ClientLevel level) {
        // @era:t-unpack-make-begin r20
        return new LevelChunkSection(level.registryAccess().registryOrThrow(Registries.BIOME));
        // @era:t-unpack-make-else v16
        //~ return new LevelChunkSection(0);
        // @era:t-unpack-make-else r18
        //~ return new LevelChunkSection(0, level.registryAccess().registryOrThrow(Registry.BIOME_REGISTRY));
        // @era:t-unpack-make-else r193
        //~ return new LevelChunkSection(0, level.registryAccess().registryOrThrow(Registries.BIOME));
        // @era:t-unpack-make-else l212
        //~ return new LevelChunkSection(level.registryAccess().lookupOrThrow(Registries.BIOME));
        // @era:t-unpack-make-else f219
        //~ return new LevelChunkSection(level.palettedContainerFactory());
        // @era:t-unpack-make-end
    }
}
