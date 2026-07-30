LD = x86_64-elf-ld
AS = nasm
TARGET_DIR = ../bin
LDFLAGS = -m elf_i386 -T linker.ld
RUST_MANIFEST = ../Cargo.toml
RUST_TARGET = ../targets/i686-x86-os.json
RUST_TARGET_DIR = ../target
RUST_LIBRARY = $(RUST_TARGET_DIR)/i686-x86-os/release/libkernel_rs.a
RUST_CARGO = $(shell rustup which cargo)
RUST_RUSTC = $(shell rustup which rustc)

ASM_SRC = arch/x86/isr_entry.asm arch/x86/switch_to.asm arch/x86/lowlevel.asm
OBJ = $(patsubst arch/x86/%.asm,$(TARGET_DIR)/%.o,$(ASM_SRC))
TARGET = $(TARGET_DIR)/kernel

.PHONY: build clean

build: $(TARGET)

clean:
	rm -f $(OBJ) $(TARGET_DIR)/kernel $(TARGET_DIR)/kernel.sym
	$(RUST_CARGO) clean --manifest-path $(RUST_MANIFEST)

$(TARGET_DIR):
	mkdir -p $(TARGET_DIR)

$(TARGET_DIR)/%.o: arch/x86/%.asm | $(TARGET_DIR)
	@echo "Assembling $<"
	$(AS) -f elf32 -g -o $@ $<

$(RUST_LIBRARY): $(shell find src -type f) \
		$(RUST_MANIFEST) ../Cargo.lock Cargo.toml \
		$(RUST_TARGET) ../rust-toolchain.toml
	RUSTC=$(RUST_RUSTC) $(RUST_CARGO) build \
		-Z build-std=core,compiler_builtins \
		-Z json-target-spec \
		--manifest-path $(RUST_MANIFEST) \
		--target $(RUST_TARGET) \
		--target-dir $(RUST_TARGET_DIR) \
		--release

$(TARGET): $(OBJ) $(RUST_LIBRARY)
	@echo "Linking $@"
	$(LD) $(LDFLAGS) -o $@ $^

gdb_symbol:
	@echo "generate symbol table...."
	x86_64-elf-objcopy --only-keep-debug $(TARGET_DIR)/kernel $(TARGET_DIR)/kernel.sym
