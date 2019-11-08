#include <stdint.h>
#include <stdbool.h>

struct CDevAccountResource account_resource_from_lcs(const uint8_t *buf, size_t len);

void account_resource_free(struct CDevAccountResource *point);
