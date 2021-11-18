#define __STDC_FORMAT_MACROS 1
#include <inttypes.h>

#include "node_info.h"
#include "stdlib.h"

#include <dqlite/client.h>
#include <dqlite/request.h>
#include <dqlite/response.h>
#include <dqlite/message.h>
//#include "sync-clocks/master.h"
#include <uv.h>

#define EMPTY_ROW 2

#define MAX_CLIENTS 32
#define DIAL_ATTEMPTS 3

#define MAX_ADDRESS_SIZE 80
#define USERS_PORT 18000
#define USER_BUFFER_LEN 1024

#define MIN_ID 1
#define MIN_NON_LEADER_ID 2
#define FIRST_LEADER_ID 1

#define STANDBY_ROLE 0
#define VOTER_ROLE 1
#define SPARE_ROLE 2

#define LEASE_ID_SIZE 8
#define START_MAX_SIZE 16
#define END_MAX_SIZE 16

#define TYPE_QUERY 0
#define TYPE_INSERT 1

#define SERVER_PORT 24000

#define DATABASENAME "our_database2"

/* How large is the buffer currently */
#define SIZE2(B) (B->n_pages * B->page_size)

/* How many remaining bytes the buffer currently */
#define CAP2(B) (SIZE2(B) - B->offset)

/* Write out a request. */
#define REQUEST2(LOWER, UPPER)                              \
    {                                                       \
        struct message message;                             \
        size_t n;                                           \
        size_t n1;                                          \
        size_t n2;                                          \
        void *cursor;                                       \
        ssize_t rv;                                         \
        n1 = message__sizeof(&message);                     \
        n2 = request_##LOWER##__sizeof(&request);           \
        n = n1 + n2;                                        \
        buffer__reset(&c->write);                           \
        cursor = buffer__advance(&c->write, n);             \
        if (cursor == NULL)                                 \
        {                                                   \
            return DQLITE_NOMEM;                            \
        }                                                   \
        assert(n2 % 8 == 0);                                \
        message.type = DQLITE_REQUEST_##UPPER;              \
        message.words = (uint32_t)(n2 / 8);                 \
        message__encode(&message, &cursor);                 \
        request_##LOWER##__encode(&request, &cursor);       \
        rv = write(c->fd, buffer__cursor(&c->write, 0), n); \
        if (rv != (int)n)                                   \
        {                                                   \
            return DQLITE_ERROR;                            \
        }                                                   \
    }

/* Read a response without decoding it. */
#define READ2(LOWER, UPPER)                           \
    {                                                 \
        struct message _message;                      \
        struct cursor _cursor;                        \
        size_t _n;                                    \
        void *_p;                                     \
        ssize_t _rv;                                  \
        _n = message__sizeof(&_message);              \
        buffer__reset(&c->read);                      \
        _p = buffer__advance(&c->read, _n);           \
        assert(_p != NULL);                           \
        _rv = read(c->fd, _p, _n);                    \
        if (_rv != (int)_n)                           \
        {                                             \
            return DQLITE_ERROR;                      \
        }                                             \
        _cursor.p = _p;                               \
        _cursor.cap = _n;                             \
        _rv = message__decode(&_cursor, &_message);   \
        assert(_rv == 0);                             \
        if (_message.type != DQLITE_RESPONSE_##UPPER) \
        {                                             \
            return DQLITE_ERROR;                      \
        }                                             \
        buffer__reset(&c->read);                      \
        _n = _message.words * 8;                      \
        _p = buffer__advance(&c->read, _n);           \
        if (_p == NULL)                               \
        {                                             \
            return DQLITE_ERROR;                      \
        }                                             \
        _rv = read(c->fd, _p, _n);                    \
        if (_rv != (int)_n)                           \
        {                                             \
            return DQLITE_ERROR;                      \
        }                                             \
    }

/* Decode a response. */
#define DECODE2(LOWER)                                       \
    {                                                        \
        int rv;                                              \
        struct cursor cursor;                                \
        cursor.p = buffer__cursor(&c->read, 0);              \
        cursor.cap = buffer__offset(&c->read);               \
        rv = response_##LOWER##__decode(&cursor, &response); \
        if (rv != 0)                                         \
        {                                                    \
            return DQLITE_ERROR;                             \
        }                                                    \
    }

#define RESPONSE2(LOWER, UPPER) \
    READ2(LOWER, UPPER);        \
    DECODE2(LOWER)

struct client clients[MAX_CLIENTS];
int client_sockets[MAX_CLIENTS];
struct node_info node_infos[MAX_CLIENTS];

int debug = 0;

int n_clients, n_changed;

int get_n_clients()
{
    return n_clients;
}

int connect_socket(int *fd, char *raw_str_address)
{
    int raw_str_size = strlen(raw_str_address);
    char *str_address = (char *)calloc(sizeof(char), raw_str_size);
    int done = 0;
    int char_index = 0;
    /* if the ip given comes with a port, ignore it and use the standard 24000. *
     * Here we copy only the ipv4 part into new string */
    while (!done)
    {
        if (raw_str_address[char_index] == ':' || raw_str_address[char_index] == '\0' || char_index > raw_str_size)
        {
            done = 1;
        }
        else
        {
            str_address[char_index] = raw_str_address[char_index];
        }
        char_index++;
    }
    printf("c_client - ip: %s\n", str_address);
    int res;
    struct sockaddr_in server_addr;
    memset(&server_addr, 0, sizeof(server_addr));
    *fd = socket(AF_INET, SOCK_STREAM, 0);
    if (*fd < 0)
    {
        printf("ERROR - creating socket\n");
        return 1;
    }
    server_addr.sin_family = AF_INET;
    server_addr.sin_addr.s_addr = inet_addr(str_address);
    server_addr.sin_port = htons(SERVER_PORT);
    res = connect(*fd, (struct sockaddr *)&server_addr, sizeof(server_addr));
    if (res != 0)
    {
        printf("ERROR - connecting\n");
        return res;
    }
    return 0;
}

struct client get_leader()
{
    int res;
    uint64_t leader_id;
    int done = 0;
    int client_index = 1;
    struct node_info leader;
    char *leader_address = (char *)calloc(sizeof(char), MAX_ADDRESS_SIZE);
    while (!done && client_index < n_clients)
    {
        res = clientSendLeader(&clients[client_index]);
        if (res != 0)
        {
            printf("[ERROR] Failed to send leader request to client %d\n", client_index);
        }
        else
        {
            res = clientRecvLeader(&(clients[client_index]), leader_address, &leader_id);
            if (res != 0)
            {
                printf("[ERROR] Failed to receive request from client %d\n", client_index);
                exit(1);
            }
            else
            {
                /* for the case that nodes_infos still has the uninitialized sequential id, we set it here */
                leader.id = leader_id;
                leader.address = leader_address;
                done = 1;
            }
        }
        client_index++;
    }
    return clients[client_index];
}

int exec(char *sql_stmt)
{
    int res, res0, res1, res2, res3;
    int leader_index = 0;
    int done = 0;
    int client_index = n_clients - 1;
    unsigned stmt_id, last_insert_id, rows_affected;
    uint64_t leader_id = 1000;
    printf("exec - creating leader address\n");
    char *leader_address = (char *)calloc(sizeof(char), MAX_ADDRESS_SIZE);
    while (!done && client_index <= n_clients)
    {
        res = clientSendLeader(&(clients[client_index]));
        if (res != 0)
        {
            printf("[ERROR] Failed to send request to client %d\n", client_index);
            exit(1);
        }
        else
        {
            res = clientRecvLeader(&(clients[client_index]), leader_address, &leader_id);
            if (res != 0)
            {
                printf("[ERROR] Failed to receive request from client %d\n", client_index);
                exit(1);
            }
            else
            {
                /* for the case that nodes_infos still has the uninitialized sequential id, we set it here */
                node_infos[client_index].id = leader_id;
                leader_index = client_index;
                if (debug)
                {
                    printf("[DEBUG] Leader id was: %" PRIu64 " - id on index is: %" PRIu64 "\n", leader_id, node_infos[client_index].id);
                    printf("[DEBUG] Leader address was: %s - address on index is: %s\n", leader_address, node_infos[client_index].address);
                }
                done = 1;
            }
        }
        client_index++;
    }
    if (client_index > MAX_CLIENTS)
    {
        return -1;
    }
    printf("exec - sending prepare - leader index: %d - sql statement: %s\n", leader_index, sql_stmt);
    res0 = clientSendPrepare(&(clients[leader_index]), sql_stmt);
    if (res0 != 0)
    {
        printf("ERROR - Failed to prepare statement\n");
    }
    res1 = clientRecvStmt(&(clients[leader_index]), &stmt_id);
    if (res1 != 0)
    {
        printf("ERROR - Failed to receive statement\n");
    }
    res2 = clientSendExec(&(clients[leader_index]), stmt_id);
    if (res2 != 0)
    {
        printf("ERROR - Failed to execute statement\n");
    }
    res3 = clientRecvResult(&(clients[leader_index]), &last_insert_id, &rows_affected);
    if (res3 != 0)
    {
        printf("ERROR - Failed to receive results\n");
    }
    res = res0 && res1 && res2 && res3;
    free(leader_address);
    n_changed = (int)rows_affected;
    return res;
}

int raw_query(struct rows *rows, char *sql_stmt)
{
    int res, res0, res1, res2, res3, leader_index;
    int done = 0;
    uint64_t leader_id;
    int client_index = n_clients - 1;
    unsigned stmt_id, last_insert_id, rows_affected;
    struct row *row;
    char *leader_address = (char *)calloc(sizeof(char), MAX_ADDRESS_SIZE);
    while (!done && client_index <= n_clients)
    {
        res = clientSendLeader(&(clients[client_index]));
        if (res != 0)
        {
            printf("[ERROR] Failed to send request to client %d\n", client_index);
            exit(1);
        }
        else
        {
            res = clientRecvLeader(&(clients[client_index]), leader_address, &leader_id);
            if (res != 0)
            {
                printf("[ERROR] Failed to receive request from client %d\n", client_index);
                exit(1);
            }
            else
            {
                /* for the case that nodes_infos still has the uninitialized sequential id, we set it here */
                node_infos[client_index].id = leader_id;
                leader_index = client_index;
                if (debug)
                {
                    printf("[DEBUG] Leader id was: %" PRIu64 " - id on index is: %" PRIu64 "\n", leader_id, node_infos[client_index].id);
                    printf("[DEBUG] Leader address was: %s - address on index is: %s\n", leader_address, node_infos[client_index].address);
                }
                done = 1;
            }
        }
        client_index++;
    }
    if (client_index > MAX_CLIENTS)
    {
        return -1;
    }
    printf("raw_exec - sending prepare - leader index: %d - sql statement: %s\n", leader_index, sql_stmt);
    res0 = clientSendPrepare(&(clients[leader_index]), sql_stmt);
    if (res0 != 0)
    {
        printf("ERROR - Failed to prepare statement\n");
    }
    res1 = clientRecvStmt(&(clients[leader_index]), &stmt_id);
    if (res1 != 0)
    {
        printf("ERROR - Failed to receive statement\n");
    }
    res2 = clientSendQuery(&(clients[leader_index]), stmt_id);
    if (res2 != 0)
    {
        printf("ERROR - Failed to execute statement\n");
    }
    res3 = clientRecvRows(&(clients[leader_index]), rows);
    if (res3 != 0)
    {
        printf("ERROR - Failed to receive results\n");
    }
    //    if(rows->next == NULL) {
    //        res = EMPTY_ROW;
    //	printf("Error - Empty row\n");
    //    }
    res = res0 && res1 && res2 && res3;
    return res;
}

int send_open(char *database_name)
{
    int open = 0;
    int failed = 0;
    int remaining_servers = n_clients - 1;
    //TODO: remaining servers must be n_clients
    int remaining_attempts = DIAL_ATTEMPTS;
    while (!open && !failed)
    {
        printf("send open: Sending open request to server %d\n", remaining_servers);
        int res = clientSendOpen(&(clients[remaining_servers]), database_name);
        if (res != 0)
        {
            printf("Open request failed\n");
            exit(1);
        }
        res = clientRecvDb(&(clients[remaining_servers]));
        if (res != 0)
        {
            remaining_attempts--;
            if (remaining_attempts == 0)
            {
                remaining_servers--;
                remaining_attempts = DIAL_ATTEMPTS;
            }
            if (remaining_servers == 0)
            {
                failed = 1;
                printf("Failed to recvDb after open. Result: %d\n", res);
            }
        }
        else
        {
            open = 1;
        }
    }
    return failed;
}

int createTable(char *create_sql)
{
    //TODO: get leader and querry it instead of assume it's clients[1]
    int res, res0, res1, res2, res3;
    struct client *main_client = &clients[1];
    unsigned stmt_id;
    unsigned last_insert_id;
    unsigned rows_affected;
    res0 = clientSendPrepare(main_client, create_sql);
    if (res0 != 0)
    {
        printf("ERROR - Failed to prepare statement\n");
    }
    res1 = clientRecvStmt(main_client, &stmt_id);
    if (res1 != 0)
    {
        printf("ERROR - Failed to receive statement\n");
    }
    res2 = clientSendExec(main_client, stmt_id);
    if (res2 != 0)
    {
        printf("ERROR - Failed to execute statement\n");
    }
    res3 = clientRecvResult(main_client, &last_insert_id, &rows_affected);
    if (res3 != 0)
    {
        printf("ERROR - Failed to receive results\n");
    }
    res = res0 && res2 && res3;
    return res;
}

int clientSendAdd2(struct client *c, unsigned id, const char *address)
{
    struct request_add request;
    request.id = id;
    request.address = address;
    REQUEST2(add, ADD);
    return 0;
}

int addServer(struct client *c, unsigned id, char *address)
{
    printf("[ADD SERVER] %d %s \n", id, address);

    int res;
    res = clientSendAdd(c, id, address);
    if (res != 0)
    {
        printf("[ERROR] Failed to send Add request1 %d \n", res);
        perror("Error:");
        return -1;
    }
    res = clientRecvEmpty(c);
    if (res != 0)
    {
        printf("[ERROR] Failed to send Add request2 %d \n", res);
        perror("Error:");
        return -1;
    }
    return res;
}

int removeServer(struct client *c, unsigned id)
{
    int res;
    res = clientSendRemove(c, id);
    if (res != 0)
    {
        printf("[ERROR] Failed to send Remove request\n");
        perror("Error:");
        return -1;
    }
    res = clientRecvEmpty(c);
    if (res != 0)
    {
        printf("[ERROR] Failed to send Remove request\n");
        perror("Error:");
        return -1;
    }
    return res;
}

int close_sockets()
{
    for (int i = 0; i < get_n_clients(); i++)
    {
        clientClose(&(clients[i]));
    }
    return 0;
}

struct node_info *get_node_info(int index)
{
    return &(node_infos[index]);
}

struct client *get_client(int index)
{
    return &(clients[index]);
}

int *get_client_socket(int index)
{
    return &(client_sockets[index]);
}

void set_n_clients(int new_n_clients)
{
    n_clients = new_n_clients;
}
