
CC = x86_64-elf-gcc
AR = x86_64-elf-ar
CFLAGS = -m32 -c -fno-stack-protector -nostdlib -ffreestanding -fno-pie \
	-mno-sse -mno-sse2 -mno-mmx -msoft-float -g
SRC = $(wildcard *.c)
OBJS = $(SRC:.c=.o) port.o
LIB = ../../bin/libklib.a

.PHONY: build clean

build:$(LIB)
	rm -f *.o
	
%.o: %.c 
	$(CC) $(CFLAGS) $< -o $@

port.o: port.s
	nasm -f elf32 -g -o $@ $<

$(LIB): $(OBJS)
	$(AR) rcs $(LIB) $(OBJS)

clean:
	rm -f *.o
