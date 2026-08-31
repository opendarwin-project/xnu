#define XNU_KERNEL_PRIVATE 1
#include <kern/startup.h>
#include <libkern/libkern.h>
#include <libkern/section_keywords.h>
#include <libkern/amfi/amfi.h>

SECURITY_READ_ONLY_LATE(const amfi_t*) amfi = NULL;
SECURITY_READ_ONLY_LATE(const CEKernelAPI_t*) libCoreEntitlements = NULL;

void
amfi_interface_register(const amfi_t *mfi)
{
	if (amfi) {
		panic("AppleMobileFileIntegrity interface already set");
	}
	amfi = mfi;
}

void
amfi_core_entitlements_register(const CEKernelAPI_t *implementation)
{
	if (libCoreEntitlements) {
		panic("libCoreEntitlements interface already set");
	}
	libCoreEntitlements = implementation;
}

static TCReturn_t
stub_tc_loadModule(TrustCacheRuntime_t *runtime, const TCType_t type, TrustCache_t *trustCache, const uintptr_t dataAddr, const size_t dataSize)
{
	(void)runtime; (void)type; (void)trustCache; (void)dataAddr; (void)dataSize;
	return buildTCRet(kTCComponentLoadModule, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_load(TrustCacheRuntime_t *runtime, TCType_t type, TrustCache_t *trustCache, const uintptr_t payloadAddr, const size_t payloadSize, const uintptr_t manifestAddr, const size_t manifestSize)
{
	(void)runtime; (void)type; (void)trustCache; (void)payloadAddr; (void)payloadSize; (void)manifestAddr; (void)manifestSize;
	return buildTCRet(kTCComponentLoad, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_query(const TrustCacheRuntime_t *runtime, TCQueryType_t queryType, const uint8_t CDHash[kTCEntryHashSize], TrustCacheQueryToken_t *queryToken)
{
	(void)runtime; (void)queryType; (void)CDHash; (void)queryToken;
	return buildTCRet(kTCComponentQuery, kTCReturnNotFound, 0);
}

static TCReturn_t
stub_tc_getCapabilities(const TrustCache_t *trustCache, TCCapabilities_t *capabilities)
{
	(void)trustCache;
	if (capabilities) {
		*capabilities = 0;
	}
	return buildTCRet(kTCComponentModuleCapabilities, kTCReturnSuccess, 0);
}

static TCReturn_t
stub_tc_queryGetTCType(const TrustCacheQueryToken_t *queryToken, TCType_t *typeRet)
{
	(void)queryToken; (void)typeRet;
	return buildTCRet(kTCComponentQueryTCType, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_queryGetCapabilities(const TrustCacheQueryToken_t *queryToken, TCCapabilities_t *capabilities)
{
	(void)queryToken; (void)capabilities;
	return buildTCRet(kTCComponentQuery, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_queryGetHashType(const TrustCacheQueryToken_t *queryToken, uint8_t *hashTypeRet)
{
	(void)queryToken; (void)hashTypeRet;
	return buildTCRet(kTCComponentQueryHashType, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_queryGetFlags(const TrustCacheQueryToken_t *queryToken, uint64_t *flagsRet)
{
	(void)queryToken; (void)flagsRet;
	return buildTCRet(kTCComponentQueryFlags, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_queryGetConstraintCategory(const TrustCacheQueryToken_t *queryToken, uint8_t *constraintCategoryRet)
{
	(void)queryToken; (void)constraintCategoryRet;
	return buildTCRet(kTCComponentQueryConstraintCategory, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_queryGetUUID(const TrustCacheQueryToken_t *queryToken, uint8_t returnUUID[kUUIDSize])
{
	(void)queryToken; (void)returnUUID;
	return buildTCRet(kTCComponentGetUUID, kTCReturnError, 0);
}

static TCReturn_t
stub_tc_checkRuntimeForUUID(const TrustCacheRuntime_t *runtime, const uint8_t checkUUID[kUUIDSize], const TrustCache_t **trustCacheRet)
{
	(void)runtime; (void)checkUUID; (void)trustCacheRet;
	return buildTCRet(kTCComponentCheckRuntimeForUUID, kTCReturnNotFound, 0);
}

static kern_return_t
stub_oe_adjustContext(void *os_entitlements, struct cs_blob *code_signing_blob, const CEContext_t *ce_ctx)
{
	(void)os_entitlements; (void)code_signing_blob; (void)ce_ctx;
	return KERN_SUCCESS;
}

static kern_return_t
stub_oe_adjustContextWithMonitor(void *os_entitlements, const CEQueryContext_t ce_ctx, const void *monitor_sig_obj, const char *identity, const uint32_t code_signing_flags)
{
	(void)os_entitlements; (void)ce_ctx; (void)monitor_sig_obj; (void)identity; (void)code_signing_flags;
	return KERN_SUCCESS;
}

static kern_return_t
stub_oe_adjustContextWithoutMonitor(void *os_entitlements, struct cs_blob *code_signing_blob)
{
	(void)os_entitlements; (void)code_signing_blob;
	return KERN_SUCCESS;
}

static kern_return_t
stub_oe_queryEntitlementBoolean(const void *os_entitlements, const char *entitlement_name)
{
	(void)os_entitlements; (void)entitlement_name;
	return KERN_FAILURE;
}

static kern_return_t
stub_oe_queryEntitlementBooleanWithProc(const proc_t proc, const char *entitlement_name)
{
	(void)proc; (void)entitlement_name;
	return KERN_FAILURE;
}

static kern_return_t
stub_oe_queryEntitlementString(const void *os_entitlements, const char *entitlement_name, const char *entitlement_value)
{
	(void)os_entitlements; (void)entitlement_name; (void)entitlement_value;
	return KERN_FAILURE;
}

static kern_return_t
stub_oe_queryEntitlementStringWithProc(const proc_t proc, const char *entitlement_name, const char *entitlement_value)
{
	(void)proc; (void)entitlement_name; (void)entitlement_value;
	return KERN_FAILURE;
}

static kern_return_t
stub_oe_copyEntitlementAsOSObject(const void *os_entitlements, const char *entitlement_name, void **entitlement_object)
{
	(void)os_entitlements; (void)entitlement_name; (void)entitlement_object;
	return KERN_FAILURE;
}

static kern_return_t
stub_oe_copyEntitlementAsOSObjectWithProc(const proc_t proc, const char *entitlement_name, void **entitlement_object)
{
	(void)proc; (void)entitlement_name; (void)entitlement_object;
	return KERN_FAILURE;
}

static const amfi_t g_builtin_amfi = {
	.TrustCache = {
		.version = TRUST_CACHE_INTERFACE_VERSION,
		.loadModule = stub_tc_loadModule,
		.load = stub_tc_load,
		.query = stub_tc_query,
		.getCapabilities = stub_tc_getCapabilities,
		.queryGetTCType = stub_tc_queryGetTCType,
		.queryGetCapabilities = stub_tc_queryGetCapabilities,
		.queryGetHashType = stub_tc_queryGetHashType,
		.queryGetFlags = stub_tc_queryGetFlags,
		.queryGetConstraintCategory = stub_tc_queryGetConstraintCategory,
		.queryGetUUID = stub_tc_queryGetUUID,
		.checkRuntimeForUUID = stub_tc_checkRuntimeForUUID,
	},
	.OSEntitlements = {
		.version = OSENTITLEMENTS_INTERFACE_VERSION,
		.adjustContext = stub_oe_adjustContext,
		.adjustContextWithMonitor = stub_oe_adjustContextWithMonitor,
		.adjustContextWithoutMonitor = stub_oe_adjustContextWithoutMonitor,
		.queryEntitlementBoolean = stub_oe_queryEntitlementBoolean,
		.queryEntitlementBooleanWithProc = stub_oe_queryEntitlementBooleanWithProc,
		.queryEntitlementString = stub_oe_queryEntitlementString,
		.queryEntitlementStringWithProc = stub_oe_queryEntitlementStringWithProc,
		.copyEntitlementAsOSObject = stub_oe_copyEntitlementAsOSObject,
		.copyEntitlementAsOSObjectWithProc = stub_oe_copyEntitlementAsOSObjectWithProc,
	},
};

static void
amfi_early_init(void)
{
	if (amfi == NULL) {
		amfi_interface_register(&g_builtin_amfi);
	}
}
STARTUP(EARLY_BOOT, STARTUP_RANK_FOURTH, amfi_early_init);
