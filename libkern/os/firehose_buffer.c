/*
 * Minimal in-kernel Firehose buffer implementation.
 *
 * Apple ships this code in the non-public libfirehose_kernel.  XNU owns the
 * backing allocation and exposes it to logd, so keep the buffer protocol here
 * rather than depending on a host-provided library.
 */

#include <mach/vm_types.h>
#include <kern/assert.h>

#include <firehose/chunk_private.h>
#include <os/firehose_buffer_private.h>

extern vm_offset_t kernel_firehose_addr;
extern uint8_t __firehose_buffer_kernel_chunk_count;
extern uint8_t __firehose_num_kernel_io_pages;

static firehose_chunk_t firehose_chunks;
static uint8_t firehose_chunk_count;

static firehose_chunk_t
firehose_chunk_for_stream(firehose_stream_t stream)
{
	if (stream >= _firehose_stream_max || !firehose_chunks || !firehose_chunk_count) {
		return NULL;
	}

	/* One chunk per stream avoids changing a chunk's stream while a writer owns it. */
	return &firehose_chunks[stream % firehose_chunk_count];
}

firehose_buffer_t
__firehose_buffer_create(size_t *size)
{
	firehose_chunk_t chunks = (firehose_chunk_t)kernel_firehose_addr;
	uint8_t count = __firehose_buffer_kernel_chunk_count;

	if (!size || !chunks || !__firehose_kernel_configuration_valid(count,
	    __firehose_num_kernel_io_pages)) {
		return NULL;
	}

	*size = (size_t)count * FIREHOSE_CHUNK_SIZE;
	firehose_chunks = chunks;
	firehose_chunk_count = count;

	for (uint8_t i = 0; i < count; i++) {
		firehose_chunk_pos_u pos = {
			.fcp_next_entry_offs = offsetof(struct firehose_chunk_s, fc_data),
			.fcp_private_offs = FIREHOSE_CHUNK_SIZE,
			.fcp_stream = i % _firehose_stream_max,
		};

		chunks[i].fc_timestamp = 0;
		chunks[i].fc_pos.fcp_pos = pos.fcp_pos;
	}

	return (firehose_buffer_t)chunks;
}

firehose_tracepoint_t
__firehose_buffer_tracepoint_reserve(uint64_t stamp, firehose_stream_t stream,
    uint16_t pubsize, uint16_t privsize, uint8_t **privptr)
{
	firehose_chunk_t chunk = firehose_chunk_for_stream(stream);
	long offset;

	if (!chunk) {
		return NULL;
	}

	if (chunk->fc_timestamp == 0) {
		chunk->fc_timestamp = stamp;
	}

	offset = firehose_chunk_tracepoint_try_reserve(chunk, stamp, stream, 0,
	    pubsize, privsize, privptr);
	if (offset <= 0) {
		return NULL;
	}

	return firehose_chunk_tracepoint_begin(chunk, stamp, pubsize, 0, offset);
}

void
__firehose_buffer_tracepoint_flush(firehose_tracepoint_t ft,
    firehose_tracepoint_id_u ftid)
{
	firehose_chunk_t chunk;

	if (!ft) {
		return;
	}

	chunk = firehose_chunk_for_address(ft);
	if (firehose_chunk_tracepoint_end(chunk, ft, ftid)) {
		__firehose_buffer_push_to_logd((firehose_buffer_t)firehose_chunks, false);
	}
}

bool
__firehose_merge_updates(firehose_push_reply_t update)
{
	/*
	 * The QEMU port has no logd chunk-recycling protocol yet.  Returning false
	 * prevents oslogselect() from spinning after LOGFLUSHED while preserving the
	 * valid, read-only buffer exported by LOGBUFFERMAP.
	 */
	return update.fpr_mem_flushed_pos != 0 || update.fpr_io_flushed_pos != 0;
}

int
__firehose_kernel_configuration_valid(uint8_t chunk_count, uint8_t io_pages)
{
	return chunk_count >= FIREHOSE_BUFFER_KERNEL_MIN_CHUNK_COUNT &&
	    chunk_count <= FIREHOSE_BUFFER_KERNEL_MAX_CHUNK_COUNT &&
	    io_pages != 0 && io_pages <= chunk_count;
}
