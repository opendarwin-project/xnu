#define XNU_KERNEL_PRIVATE 1
#include <kern/startup.h>
#include <libkern/libkern.h>
#include <libkern/section_keywords.h>
#include <libkern/img4/interface.h>

#if defined(SECURITY_READ_ONLY_LATE)
SECURITY_READ_ONLY_LATE(const img4_interface_t *) img4if = NULL;
#else
const img4_interface_t *img4if = NULL;
#endif

void
img4_interface_register(const img4_interface_t *i4)
{
	if (img4if) {
		panic("img4 interface already set");
	}
	img4if = i4;
}

struct _img4_chip { int unused; };
struct _img4_nonce_domain { int unused; };
struct _img4_object_spec { int unused; };

static const img4_runtime_t g_img4_runtime_default = { .i4rt_version = 0 };
static const img4_runtime_t g_img4_runtime_pmap_cs = { .i4rt_version = 0 };
static const struct _img4_nonce_domain g_img4_nonce_domain_trust_cache = {0};
static const struct _img4_chip g_img4_chip_ap_sha1 = {0};
static const struct _img4_chip g_img4_chip_ap_sha2_384 = {0};
static const struct _img4_chip g_img4_chip_ap_hybrid = {0};
static const struct _img4_chip g_img4_chip_ap_reduced = {0};
static const struct _img4_chip g_img4_chip_ap_permissive = {0};
static const struct _img4_object_spec g_img4_firmware_spec = {0};
static const struct _img4_object_spec g_img4_chip_spec = {0};
static const struct _img4_object_spec g_img4_pmap_data_spec = {0};

static const img4_interface_t g_builtin_img4_interface = {
	.i4if_version = IMG4_INTERFACE_VERSION,
	.i4if_v1 = {
		.nonce_domain_trust_cache = (const img4_nonce_domain_t *)&g_img4_nonce_domain_trust_cache,
	},
	.i4if_v7 = {
		.chip_ap_sha1 = (const img4_chip_t *)&g_img4_chip_ap_sha1,
		.chip_ap_sha2_384 = (const img4_chip_t *)&g_img4_chip_ap_sha2_384,
		.chip_ap_hybrid = (const img4_chip_t *)&g_img4_chip_ap_hybrid,
		.chip_ap_reduced = (const img4_chip_t *)&g_img4_chip_ap_reduced,
		.firmware_spec = (const img4_object_spec_t *)&g_img4_firmware_spec,
		.chip_spec = (const img4_object_spec_t *)&g_img4_chip_spec,
		.runtime_default = &g_img4_runtime_default,
		.runtime_pmap_cs = &g_img4_runtime_pmap_cs,
	},
	.i4if_v8 = {
		.chip_ap_permissive = (const img4_chip_t *)&g_img4_chip_ap_permissive,
	},
	.i4if_v13 = {
		.pmap_data_spec = (const img4_object_spec_t *)&g_img4_pmap_data_spec,
	},
};

static void
img4_early_init(void)
{
	if (img4if == NULL) {
		img4_interface_register(&g_builtin_img4_interface);
	}
}
STARTUP(EARLY_BOOT, STARTUP_RANK_FOURTH, img4_early_init);
