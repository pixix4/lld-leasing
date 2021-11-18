#define MAXQUERY	256
#define AESGCMTAGSIZE	16
//TODO: Find out real tables
#define FSPFTABLE	"FSPF"
#define CASCONFIGTABLE	"CASConfig"
#define ERROR		"ERROR"
#define IPSFILE		"test_ips.csv"

#include <dqlite/sqlite3.h>
#include "c_client.h"
#include "net.h"

//typedef struct sqlite3_stmt {
//    char *text;
//}
//
//typedef struct sqlite3 {
//    int fd;
//}
//
//typedef struct sqlite3_value {
//    void *value;
//    int size;
//}
//
//typedef struct sqlite3_context{
//    sqlite3 *conn;
//    char *err;
//}

void *sqlite3_malloc(int size) {
    return malloc(size); 
}


void sqlite3_free(void *ptr){
    free(ptr);
}

void *sqlite3_realloc(void *ptr, size_t size) {
    return realloc(ptr, size);
}

void *sqlite3_malloc64(long size) {
    return malloc((int) size);
}



int sqlite_bind_text(sqlite3_stmt *stmt, int index, const char *text, void *text_utf8, void *text_utf16) {
     
}

sqlite3 *sqlite3_context_db_handle(sqlite3_context *ctx) {
    return ctx->conn;
}

const void *sqlite3_value_blob(sqlite3_value *value) {
    return value->value;
}

double sqlite3_value_double(sqlite3_value *value) {
    double *ret;
    ret = (double *) value->value;
    return *ret;
}

int sqlite3_value_int(sqlite3_value *value) {
    int *ret;
    ret = (int *) value->value;
    return *ret;
}

long sqlite3_value_int64(sqlite_value *value) {
    long *ret;
    ret = (long *) value->value;
    return *ret;
}

const unsigned char *sqlite3_value_text(sqlite3_value *value){
    const unsigned char *text;
    text = (char *) value->value;
    return text;
}

int sqlite3_value_bytes(sqlite3_value *value) {
    return value->size;    
}

void *sqlite3_aggregate_context(sqlite3_context *ctx, int n_bytes) {
    void *space = (void *) malloc(n_bytes);
    memset(space, 0, n_bytes);
    return space;
}


int sqlite3_close(sqlite3 *conn) {
    close_sockets();
}

int sqlite3_open_v2(char *url, sqlite3 *conn, int flags, const char *vfs) { 
    init_ips(IPSFILE);
    int res = connect_socket(conn, url);
    return res;
}


int sqlite3_exec(sqlite3 conn, char *stmt, int callback, void *cb_args, char **errmsg) {
    /* conn is unused */
    int res = exec(stmt); 
    if(res != 0) {
        errmsg[0] = (char *) ERROR;
    }
    return res;
}


int sqlite3_changes(sqlite3 *conn) {
    return n_changed;
}


//TODO: Partial Certificate Chain
