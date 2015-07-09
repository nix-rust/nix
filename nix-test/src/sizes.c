#include "sys/socket.h"
#include "sys/uio.h"

#define SIZE_OF_T(TYPE)                   \
    do {                                  \
        if (0 == strcmp(type, #TYPE)) {   \
            return sizeof(TYPE);          \
        }                                 \
    } while (0)

#define SIZE_OF_S(TYPE)                   \
    do {                                  \
        if (0 == strcmp(type, #TYPE)) {   \
            return sizeof(struct TYPE);   \
        }                                 \
    } while (0)

size_t
size_of(const char* type) {
    // Builtin
    SIZE_OF_T(long);

    // sys/socket
    SIZE_OF_S(sockaddr_storage);

    // sys/uio
    SIZE_OF_S(iovec);

    return 0;
}
