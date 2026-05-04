#ifndef COMPAT_H
#define COMPAT_H

#include <stdint.h>
#include <sys/types.h>
#include <limits.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <errno.h>

#ifndef boolean_t
typedef int boolean_t;
#endif

#ifndef TRUE
#define TRUE 1
#define FALSE 0
#endif

typedef unsigned int u_int;
typedef unsigned int natural_t;

typedef u_int ipc_flags_t;

typedef unsigned int mach_msg_descriptor_type_t;
typedef unsigned int mach_msg_type_name_t;
typedef unsigned int mach_port_t;
typedef unsigned int mach_msg_type_number_t;

typedef struct {
    unsigned int msg_bits;
    unsigned int msg_size;
    unsigned int msg_remote_port;
    unsigned int msg_local_port;
    unsigned int msg_voucher_port;
    int msg_id;
} mach_msg_header_t;

typedef struct {
    unsigned int msgh_descriptor_count;
} mach_msg_body_t;

#define MACH_MSG_TYPE_MOVE_RECEIVE      16      /* Must hold receive rights */
#define MACH_MSG_TYPE_MOVE_SEND         17      /* Must hold send rights */
#define MACH_MSG_TYPE_MOVE_SEND_ONCE    18      /* Must hold sendonce rights */
#define MACH_MSG_TYPE_COPY_SEND         19      /* Must hold send rights */
#define MACH_MSG_TYPE_MAKE_SEND         20      /* Must hold receive rights */
#define MACH_MSG_TYPE_MAKE_SEND_ONCE    21      /* Must hold receive rights */
#define MACH_MSG_TYPE_COPY_RECEIVE      22      /* Must hold receive rights */
#define MACH_MSG_TYPE_PORT_RECEIVE      MACH_MSG_TYPE_MOVE_RECEIVE
#define MACH_MSG_TYPE_PORT_SEND         MACH_MSG_TYPE_MOVE_SEND
#define MACH_MSG_TYPE_PORT_SEND_ONCE    MACH_MSG_TYPE_MOVE_SEND_ONCE
#define MACH_MSG_TYPE_PORT_NAME         15

#define MACH_MSG_TYPE_POLYMORPHIC       ((mach_msg_type_name_t) -1)

#define MACH_MSG_PORT_DESCRIPTOR 0
#define MACH_MSG_OOL_DESCRIPTOR  1
#define MACH_MSG_OOL_PORTS_DESCRIPTOR 2
#define MACH_MSG_OOL_VOLATILE_DESCRIPTOR  3

#define MIG_VERSION "mig-138"

#define bool _mig_bool

#define __private_extern__ extern

#define MACH_MSG_TYPE_PORT_ANY(x) ((x) >= 16 && (x) <= 22)

typedef struct {
    char data[8];
} NDR_record_t;

#endif
