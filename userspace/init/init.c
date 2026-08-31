/*
 * Minimal freestanding userspace init for XNU mockfs boot test.
 * Issues syscalls directly via svc #0x80 (no libc/libSystem).
 */

#define SYS_exit   1
#define SYS_open   5
#define SYS_close  6
#define SYS_write  4
#define SYS_dup2   90
#define SYS_reboot 55

#define O_RDWR 0x0002

static inline long
syscall3(long num, long arg1, long arg2, long arg3)
{
	register long x16 __asm__("x16") = num;
	register long x0 __asm__("x0") = arg1;
	register long x1 __asm__("x1") = arg2;
	register long x2 __asm__("x2") = arg3;
	__asm__ volatile (
		"svc #0x80"
		: "+r"(x0)
		: "r"(x16), "r"(x1), "r"(x2)
		: "memory", "cc"
	);
	return x0;
}

static inline long
syscall2(long num, long arg1, long arg2)
{
	register long x16 __asm__("x16") = num;
	register long x0 __asm__("x0") = arg1;
	register long x1 __asm__("x1") = arg2;
	__asm__ volatile (
		"svc #0x80"
		: "+r"(x0)
		: "r"(x16), "r"(x1)
		: "memory", "cc"
	);
	return x0;
}

static inline long
syscall1(long num, long arg1)
{
	register long x16 __asm__("x16") = num;
	register long x0 __asm__("x0") = arg1;
	__asm__ volatile (
		"svc #0x80"
		: "+r"(x0)
		: "r"(x16)
		: "memory", "cc"
	);
	return x0;
}

static long g_out_fd = 1;

static void
print_str(const char * s)
{
	long len = 0;
	while (s[len]) {
		len++;
	}
	syscall3(SYS_write, g_out_fd, (long)s, len);
}

/*
 * Open /dev/console and wire it up as stdin/stdout/stderr (fds 0/1/2),
 * mirroring what real launchd does in userspace on boot. Process 1 starts
 * with no open file descriptors at all, so without this, writes to fd 1
 * silently fail (fd not open) and nothing shows up on the serial console.
 */
static void
open_console(void)
{
	long fd = syscall3(SYS_open, (long) "/dev/console", O_RDWR, 0);
	if (fd < 0) {
		/* Nothing we can do to report this without a console... */
		return;
	}

	if (fd != 0) {
		syscall2(SYS_dup2, fd, 0);
	}
	if (fd != 1) {
		syscall2(SYS_dup2, fd, 1);
	}
	if (fd != 2) {
		syscall2(SYS_dup2, fd, 2);
	}
	if (fd > 2) {
		syscall1(SYS_close, fd);
	}

	g_out_fd = 1;
}

void
_start(void)
{
	open_console();

	print_str("\n========================================\n");
	print_str("  HELLO FROM XNU USERSPACE PROCESS 1 !  \n");
	print_str("  mockfs + ramdisk init boot successful \n");
	print_str("========================================\n\n");

	/*
	 * TEMPORARY liveness probe: reboot instead of spinning. If QEMU exits on
	 * its own, we know the kernel booted all the way to userspace and only the
	 * serial console is broken (rather than the kernel having hung early).
	 */
	syscall1(SYS_reboot, 0);

	while (1) {
		/* Loop forever so process 1 doesn't panic on exit */
	}
}
