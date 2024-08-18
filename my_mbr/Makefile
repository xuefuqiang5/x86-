include ../first.mk
EXECUTABLE=$(OUTPUT_DIR)/my_mbr.bin
EXECUTABLE2= my_mbr

# Compilation rule
$(EXECUTABLE): *.asm
	$(NASM) $< -o $(EXECUTABLE)
run: 
	$(QEMU) -drive format=raw,file=$(EXECUTABLE2)
release:
	$(NASM) *.asm -o $(EXECUTABLE2)

debug:
	$(QEMU) -drive format=raw,file=$(EXECUTABLE2) -S -s &
	gdb -ex "target remote localhost:1234" -ex "break *0x7C00" -ex "continue" $(EXECUTABLE2)

# Clean rule
clean:
	rm -f $(EXECUTABLE) *.o