DISK_IMG = ./disk_img_file/c.img
TARGET_DIR = ./bin
QEMU ?= qemu-system-i386
QEMUFLAGS ?= -m 64M -drive file=$(DISK_IMG),format=raw,if=ide -boot c
KERNEL_SECTOR_COUNT = 256

.PHONY: all build write_into run run-headless clean

all: write_into

$(TARGET_DIR)/mbr.bin: loader/mbr.asm loader/boot.inc
	$(MAKE) -C loader build

$(TARGET_DIR)/loader.bin: loader/loader.asm loader/boot.inc
	$(MAKE) -C loader build

$(TARGET_DIR)/kernel: $(shell find kernel lib/kernel rust/kernel-rs/src -type f) \
		rust/Cargo.toml rust/Cargo.lock rust/kernel-rs/Cargo.toml \
		rust/targets/i686-x86-os.json rust-toolchain.toml
	$(MAKE) -C kernel build

write_into: $(TARGET_DIR)/mbr.bin $(TARGET_DIR)/loader.bin $(TARGET_DIR)/kernel
	@kernel_size=$$(wc -c < $(TARGET_DIR)/kernel); \
	max_size=$$(( $(KERNEL_SECTOR_COUNT) * 512 )); \
	if [ $$kernel_size -gt $$max_size ]; then \
		echo "kernel is $$kernel_size bytes; loader limit is $$max_size bytes"; \
		exit 1; \
	fi
	dd if=$(TARGET_DIR)/mbr.bin of=$(DISK_IMG) bs=512 count=1 conv=notrunc
	dd if=$(TARGET_DIR)/loader.bin of=$(DISK_IMG) bs=512 seek=1 conv=notrunc
	dd if=$(TARGET_DIR)/kernel of=$(DISK_IMG) bs=512 count=$(KERNEL_SECTOR_COUNT) seek=9 conv=notrunc

build: write_into

run: write_into
	$(QEMU) $(QEMUFLAGS)

run-headless: write_into
	$(QEMU) $(QEMUFLAGS) -display none -serial stdio -monitor none

clean:
	$(MAKE) -C loader clean
	$(MAKE) -C kernel clean
