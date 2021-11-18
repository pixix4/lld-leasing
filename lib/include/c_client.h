#include <dqlite/client.h>

#ifndef C_CLIENT_H_
#define C_CLIENT_H_

int connect_socket(int *fd, char *raw_str_address);

int exec(char *insert_stmt);
int raw_query(struct rows *rows, char *query_stmt);
int send_open();
int createTable();
int removeServer(struct client *c, unsigned id);
int addServer(struct client *c, unsigned id, char *address);
int clientSendAdd2(struct client *c, unsigned id, const char *address);

void set_n_clients(int new_n_clients);


struct client *get_client(int index);
struct node_info *get_node_info(int index);
int *get_client_socket(int index);

#endif
