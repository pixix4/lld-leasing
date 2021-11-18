SRC:=$(sort $(wildcard *.c))
OBJ:=$(SRC:.c=.o)

INCLUDE_DIR=./include

all: libdqlitec.so

libdqlitec.so: $(OBJ)
	$(CC) -shared $^ -o $@ -I $(INCLUDE_DIR)

%.o: %.c
	$(CC) $(CFLAGS) -ffunction-sections -O2 -fPIC -ggdb3 -Wall -c $^ -o $@ -I $(INCLUDE_DIR)

clean:
	rm *.so *.o

install: libdqlitec.so
	cp $^ /usr/local/lib
	cp include/*.h /usr/local/include