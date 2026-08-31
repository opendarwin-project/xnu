#define XNU_KERNEL_PRIVATE 1
#include <mach/kmod.h>
#include <mach/mach_types.h>
#include <libkern/libkern.h>
#include <kern/startup.h>
#include <corecrypto/module_id.h>
#include <corecrypto/ccdigest.h>
#include <corecrypto/ccsha2.h>

extern struct mach_header_64 _mh_execute_header;
extern kern_return_t corecrypto_kext_start(kmod_info_t *ki, void *d);
extern kern_return_t corecrypto_kext_stop(kmod_info_t *ki, void *d);

kmod_info_t g_corecrypto_kmod_info = {
	.next = NULL,
	.info_version = KMOD_INFO_VERSION,
	.id = (uint32_t)-1,
	.name = "com.apple.kec.corecrypto",
	.version = "1.0",
	.reference_count = -1,
	.reference_list = NULL,
	.address = 0,
	.size = 0,
	.hdr_size = 0,
	.start = corecrypto_kext_start,
	.stop = corecrypto_kext_stop,
};

static void
corecrypto_kmod_early_init(void)
{
	if (g_corecrypto_kmod_info.address == 0) {
		g_corecrypto_kmod_info.address = (vm_address_t)&_mh_execute_header;
	}
	if (g_corecrypto_kmod_info.start) {
		g_corecrypto_kmod_info.start(&g_corecrypto_kmod_info, NULL);
	}
}

STARTUP(EARLY_BOOT, STARTUP_RANK_FIRST, corecrypto_kmod_early_init);

const char *cc_module_id(enum cc_module_id_format outformat)
{
	switch (outformat) {
	case cc_module_id_Name:
		return "Apple corecrypto Module (Kernel)";
	case cc_module_id_Version:
		return "1.0";
	case cc_module_id_Full:
	default:
		return "Apple corecrypto Module v1.0 [Apple Silicon, Kernel, Software, SL1, N/A]";
	}
}

extern void *IOMallocData_external(vm_size_t size);

void *IOMallocData(vm_size_t size)
{
	return IOMallocData_external(size);
}

extern void AccelerateCrypto_SHA256_compress(uint32_t *c, size_t num, const void *p);
extern void ccdigest_final_64be(const struct ccdigest_info *di, ccdigest_ctx_t ctx, unsigned char *digest);
extern const uint32_t ccsha256_initial_state[8];

static void ccsha256_ltc_compress_shim(ccdigest_state_t c, size_t num, const void *p)
{
	AccelerateCrypto_SHA256_compress((uint32_t *)c, num, p);
}

const struct ccdigest_info ccsha256_ltc_di = {
	.output_size = 32,
	.state_size = 32,
	.block_size = 64,
	.oid_size = 9,
	.oid = (const unsigned char *)"\x06\x09\x60\x86\x48\x01\x65\x03\x04\x02\x01",
	.initial_state = ccsha256_initial_state,
	.compress = ccsha256_ltc_compress_shim,
	.final = ccdigest_final_64be,
	.impl = 3,
};

int fipspost_post_indicator(uint32_t fips_mode)
{
	(void)fips_mode;
	return 0;
}

int fipspost_post_integrity(uint32_t fips_mode, struct mach_header *pmach_header)
{
	(void)fips_mode;
	(void)pmach_header;
	return 0;
}
