#include <stddef.h>
#include <stdint.h>

uint32_t
crc32(uint32_t crc, const void *buf, size_t size)
{
	static uint32_t table[256];
	static int table_init;
	const uint8_t *p = buf;

	if (!table_init) {
		for (uint32_t i = 0; i < 256; i++) {
			uint32_t c = i;

			for (int j = 0; j < 8; j++) {
				c = (c & 1U) ? (0xedb88320U ^ (c >> 1)) : (c >> 1);
			}
			table[i] = c;
		}
		table_init = 1;
	}

	crc ^= 0xffffffffU;
	for (size_t i = 0; i < size; i++) {
		crc = table[(crc ^ p[i]) & 0xffU] ^ (crc >> 8);
	}
	return crc ^ 0xffffffffU;
}
