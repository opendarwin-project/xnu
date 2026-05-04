#include <stddef.h>

size_t
strlen(const char *s)
{
	size_t n = 0;

	while (s[n] != '\0') {
		n++;
	}
	return n;
}

size_t
strnlen(const char *s, size_t maxlen)
{
	size_t n = 0;

	while (n < maxlen && s[n] != '\0') {
		n++;
	}
	return n;
}

int
strcmp(const char *a, const char *b)
{
	while (*a != '\0' && *a == *b) {
		a++;
		b++;
	}
	return (unsigned char)*a - (unsigned char)*b;
}

int
strncmp(const char *a, const char *b, size_t n)
{
	for (size_t i = 0; i < n; i++) {
		unsigned char ca = (unsigned char)a[i];
		unsigned char cb = (unsigned char)b[i];

		if (ca != cb) {
			return ca - cb;
		}
		if (ca == '\0') {
			return 0;
		}
	}
	return 0;
}

void *
memset(void *dst, int c, size_t n)
{
	unsigned char *p = dst;

	for (size_t i = 0; i < n; i++) {
		p[i] = (unsigned char)c;
	}
	return dst;
}

void *
memmove(void *dst, const void *src, size_t n)
{
	unsigned char *d = dst;
	const unsigned char *s = src;

	if (d == s || n == 0) {
		return dst;
	}
	if (d < s) {
		for (size_t i = 0; i < n; i++) {
			d[i] = s[i];
		}
	} else {
		for (size_t i = n; i != 0; i--) {
			d[i - 1] = s[i - 1];
		}
	}
	return dst;
}

int
bcmp(const void *a, const void *b, size_t n)
{
	const unsigned char *pa = a;
	const unsigned char *pb = b;

	for (size_t i = 0; i < n; i++) {
		if (pa[i] != pb[i]) {
			return pa[i] - pb[i];
		}
	}
	return 0;
}

size_t
strlcpy(char *dst, const char *src, size_t size)
{
	size_t src_len = strlen(src);

	if (size != 0) {
		size_t copy_len = src_len < (size - 1) ? src_len : (size - 1);
		for (size_t i = 0; i < copy_len; i++) {
			dst[i] = src[i];
		}
		dst[copy_len] = '\0';
	}
	return src_len;
}
