#ifndef __TARGET_CONDITIONALS_H__
#define __TARGET_CONDITIONALS_H__

#define TARGET_OS_MAC 1
#define TARGET_OS_OSX 1
#define TARGET_OS_IPHONE 0
#define TARGET_OS_IOS 0
#define TARGET_OS_TV 0
#define TARGET_OS_WATCH 0
#define TARGET_OS_BRIDGE 0
#define TARGET_OS_DRIVERKIT 0

#if defined(__x86_64__)
#define TARGET_CPU_X86 0
#define TARGET_CPU_X86_64 1
#define TARGET_CPU_ARM 0
#define TARGET_CPU_ARM64 0
#elif defined(__arm64__)
#define TARGET_CPU_X86 0
#define TARGET_CPU_X86_64 0
#define TARGET_CPU_ARM 0
#define TARGET_CPU_ARM64 1
#endif

#endif /* __TARGET_CONDITIONALS_H__ */
