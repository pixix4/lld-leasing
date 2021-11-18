#include <stdint.h>

#ifndef NODE_INFO_H_
#define NODE_INFO_H_

struct node_info {
	uint64_t id;
	char *address;
	int nodeRole;
};
#endif

