include ../../first.mk
EXECUTABLE=$(OUTPUT_DIR)/user_program.bin
QEMU = qemu-system-x86_64
BIN = 
# Compilation rule
$(EXECUTABLE): *.asm
	$(NASM) $< -o $(EXECUTABLE)
begin:

# Clean rule
clean:
	rm -f $(EXECUTABLE) *.o