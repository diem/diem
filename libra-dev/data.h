#include <stdint.h>
#include <stdbool.h>

struct CEventHandle {
    uint64_t count;
    uint8_t key[32];
};

struct CDevAccountResource {
    uint64_t balance;
    uint64_t sequence;
    uint8_t* authentication_key;
    bool delegated_key_rotation_capability;
    bool delegated_withdrawal_capability;
    struct CEventHandle sent_events;
    struct CEventHandle received_events;
};
