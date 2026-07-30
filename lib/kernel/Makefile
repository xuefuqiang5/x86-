
AS = nasm
OBJ = port.o
LIB = ../../bin/libklib.a

.PHONY: build clean

build:$(LIB)
	rm -f *.o
	
port.o: port.s
	$(AS) -f elf32 -g -o $@ $<

$(LIB): $(OBJ)
	x86_64-elf-ar rcs $(LIB) $(OBJ)

clean:
	rm -f *.o
