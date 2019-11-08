#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>
#include "account_resource.h"
#include "data.h"

int hexchr2bin(const char hex, char *out)
{
	if (out == NULL)
		return 0;

	if (hex >= '0' && hex <= '9') {
		*out = hex - '0';
	} else if (hex >= 'A' && hex <= 'F') {
		*out = hex - 'A' + 10;
	} else if (hex >= 'a' && hex <= 'f') {
		*out = hex - 'a' + 10;
	} else {
		return 0;
	}

	return 1;
}

size_t hexs2bin(const char *hex, unsigned char **out)
{
	size_t len;
	char   b1;
	char   b2;
	size_t i;

	if (hex == NULL || *hex == '\0' || out == NULL)
		return 0;

	len = strlen(hex);
	if (len % 2 != 0)
		return 0;
	len /= 2;

	*out = malloc(len);
	memset(*out, 0 , len);
	for (i=0; i<len; i++) {
		if (!hexchr2bin(hex[i*2], &b1) || !hexchr2bin(hex[i*2+1], &b2)) {
			return 0;
		}
		(*out)[i] = (b1 << 4) | b2;
	}
	return len;
}

int main(int argc, const char **argv)
{
    const char* blob = "020000002100000001674deac5e7fca75f00ca92b1ba3697f5f01ef585011beea7b361150f4504638f0800000002000000000000002100000001a208df134fefed8442b1f01fab59071898f5a1af5164e12c594de55a7004a91c8e0000002000000036ccb9ba8b4f0cd1f3e2d99338806893dff7478c69acee9b8e1247c053783a4800e876481700000000000200000000000000200000000b14ed4f5af8f8f077c7ec4313c6d395b9a7eb5f41eab9ec15367215ca9e420a01000000000000002000000032f56f77b09773aa64c78ee39943da7ec73f91cd757e325098e11b3edc4eccb10100000000000000";

    uint8_t * result;
    int len = hexs2bin(blob, &result);

    struct CDevAccountResource account_resource = account_resource_from_lcs((const uint8_t *) result, len);
    printf("balance: %llu \n", account_resource.balance);
    printf("sequence: %llu \n", account_resource.sequence);
    printf("delegated_key_rotation_capability: %d \n", account_resource.delegated_key_rotation_capability);

    struct CEventHandle sent_events = account_resource.sent_events;
    printf("sent events count: %llu \n", sent_events.count);

    printf("sent events key: ");
    for(int i = 0; i < 32; i++) {
        printf("%d ", sent_events.key[i]);
    }

    struct CEventHandle received_events = account_resource.received_events;
    printf("received events count: %llu \n", received_events.count);

    printf("received events key: ");
    for(int i = 0; i < 32; i++) {
        printf("%d ", received_events.key[i]);
    }

    return 0;
}