#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <unistd.h>

#define IPSFILE		"ips.csv"
#define MAXSERVERS	32
#define MAXLINESIZE	32
#define DEFAULTPORT	6000

char **ips;
int init = 0;
int n_servers = 0;
int retries = 3;

void init_ips(char *ips_file) {
    FILE *fp;
    char *line;
    int lines_cnt = 0;
    // fix
    int n = 1;
    ips = (char **) malloc(sizeof(char *) * MAXSERVERS);
    line = (char *) malloc(sizeof(char) * MAXLINESIZE); 
    fp = fopen(ips_file, "r");
    while(fgets(line, MAXLINESIZE, fp) || n == 0) { 
        ips[lines_cnt] = (char *) malloc(sizeof(char*) * MAXLINESIZE);
        strcpy(ips[lines_cnt], line);
        lines_cnt++;
	n--;
    }
    n_servers = lines_cnt;
    fclose(fp);
    init = 1;
    free(line);
}

int get_n_servers(void) { 
    return n_servers;
}

char *get_ip(int index) {
    return ips[index];
}
