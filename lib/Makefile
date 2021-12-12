SRC:=$(sort $(wildcard *.c))
OBJ:=$(SRC:.c=.o)

INCLUDE_DIR=./include

all: libdqlitec.so libdqlitec.a

libdqlitec.a: $(OBJ)
	ar rcs $@ $^

libdqlitec.so: $(OBJ)
	$(CC) -shared $^ -o $@ -I $(INCLUDE_DIR)

%.o: %.c
	$(CC) $(CFLAGS) -ffunction-sections -O2 -fPIC -ggdb3 -Wall -c $^ -o $@ -I$(INCLUDE_DIR) -I/usr/local/include

clean:
	rm *.so *.o *.a

install: libdqlitec.so libdqlitec.a
	cp $^ /usr/local/lib
	cp include/*.h /usr/local/include
