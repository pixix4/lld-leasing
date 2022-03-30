#include <stdio.h>
#include <stdlib.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include <dqlite.h>
#include <string.h>

int main()
{
  /* dqlite interface to talk with client */
  const char *ip = getenv("SERVER_ADDRESS");
  char dir[] = "/tmp/dqlite-rs";
  char *id = getenv("NODE_ID");
  const char *port = getenv("PORT");

  strncat(dir, id, 1);

  const char *address = malloc(strlen(ip) + strlen(port) + 3);
  sprintf(address, "%s%s%s", ip, ":", port);
  int node_id = atoi(id);
  dqlite_node *node;
  int rv;
  mkdir(dir, 0755);
  rv = dqlite_node_create(node_id, address, dir, &node);
  if (rv != 0)
  {
    printf("dqlite_node_create: %d\n", rv);
  }
  else
  {
    printf("dqlite node created\n");
  }

  rv = dqlite_node_set_bind_address(node, address);
  if (rv != 0)
  {
    printf("dqlite_bind_address: %d\n", rv);
    perror("Error: \n");
  }
  else
  {
    printf("dqlite address bound\n");
  }

  rv = dqlite_node_start(node);
  if (rv != 0)
  {
    printf("dqlite_node_start: %d\n", rv);
  }
  else
  {
    printf("dqlite node started at address: %s\n", address);
  }
  // getchar();
  pause();
}
