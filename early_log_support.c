#include <stdarg.h>
#include <stddef.h>
#include "osfmk/libsa/string.h"
#include <os/log_private.h>

struct os_log_s {
	char _unused;
};

struct os_log_s _os_log_default;

void
os_log_with_args(__unused os_log_t oslog, __unused os_log_type_t type,
    __unused const char *format, __unused va_list args, __unused void *ret_addr)
{
}

void
log_putc(__unused char c)
{
}
