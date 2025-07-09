AS_DIR = name-as
LD_DIR = name-ld
EMU_DIR = name-emu
EXT_DIR = name-ext
DEBUG_ADAPTER_DIR = name-debug-adapter
BIN_DIR = $(EXT_DIR)/bin

TARGET_WIN = x86_64-pc-windows-gnu
TARGET_MAC_INTEL = x86_64-apple-darwin
TARGET_MAC_ARM64 = aarch64-apple-darwin
TARGET_LINUX = x86_64-unknown-linux-gnu

all: build-linux build-windows extension-compile

linux: build-linux extension-compile

windows: build-windows extension-compile

build-linux:
	mkdir -p $(BIN_DIR)
	cd $(AS_DIR) && cargo build --release --target $(TARGET_LINUX)
	mv -n target/$(TARGET_LINUX)/release/name-as $(BIN_DIR)/
	cd $(LD_DIR) && cargo build --release --target $(TARGET_LINUX)
	mv -n target/$(TARGET_LINUX)/release/name-ld $(BIN_DIR)/
	cd $(EMU_DIR) && cargo build --release --target $(TARGET_LINUX)
	mv -n target/$(TARGET_LINUX)/release/name-emu $(BIN_DIR)/
	cd $(DEBUG_ADAPTER_DIR) && cargo build --release --target $(TARGET_LINUX)
	mv -n target/$(TARGET_LINUX)/release/name-debug-adapter $(BIN_DIR)/

build-windows:
	mkdir -p $(BIN_DIR)
	cd $(AS_DIR) && cargo build --release --target $(TARGET_WIN)
	mv -n target/$(TARGET_WIN)/release/name-as.exe $(BIN_DIR)/
	cd $(LD_DIR) && cargo build --release --target $(TARGET_WIN)
	mv -n target/$(TARGET_WIN)/release/name-ld.exe $(BIN_DIR)/
	cd $(EMU_DIR) && cargo build --release --target $(TARGET_WIN)
	mv -n target/$(TARGET_WIN)/release/name-emu.exe $(BIN_DIR)/
	cd $(DEBUG_ADAPTER_DIR) && cargo build --release --target $(TARGET_WIN)
	mv -n target/$(TARGET_WIN)/release/name-debug-adapter.exe $(BIN_DIR)/

build-mac-x86_64:
	mkdir -p $(BIN_DIR)
	cd $(AS_DIR) && cargo build --release --target $(TARGET_MAC_INTEL)
	mv -n target/$(TARGET_MAC_INTEL)/release/name-as_x86_64.app $(BIN_DIR)/
	cd $(LD_DIR) && cargo build --release --target $(TARGET_MAC_INTEL)
	mv -n target/$(TARGET_MAC_INTEL)/release/name-ld_x86_64.app $(BIN_DIR)/
	cd $(EMU_DIR) && cargo build --release --target $(TARGET_MAC_INTEL)
	mv -n target/$(TARGET_MAC_INTEL)/release/name-emu_x86_64.app $(BIN_DIR)/
	cd $(DEBUG_ADAPTER_DIR) && cargo build --release --target $(TARGET_MAC_INTEL)
	mv -n target/$(TARGET_MAC_INTEL)/release/name-debug-adapter_x86_64.app $(BIN_DIR)/

build-mac-arm64:
	mkdir -p $(BIN_DIR)
	cd $(AS_DIR) && cargo build --release --target $(TARGET_MAC_ARM64)
	mv -n target/$(TARGET_MAC_ARM64)/release/name-as_arm64.app $(BIN_DIR)/
	cd $(LD_DIR) && cargo build --release --target $(TARGET_MAC_ARM64)
	mv -n target/$(TARGET_MAC_ARM64)/release/name-ld_arm64.app $(BIN_DIR)/
	cd $(EMU_DIR) && cargo build --release --target $(TARGET_MAC_ARM64)
	mv -n target/$(TARGET_MAC_ARM64)/release/name-emu_arm64.app $(BIN_DIR)/
	cd $(DEBUG_ADAPTER_DIR) && cargo build --release --target $(TARGET_MAC_ARM64)
	mv -n target/$(TARGET_MAC_ARM64)/release/name-debug-adapter_arm64.app $(BIN_DIR)/

extension-compile:
	cd $(EXT_DIR) && npm run compile

purge:
	rm -rf $(BIN_DIR)

setup:
	@echo "Please ensure you have `mingw-w64` and `rustup` installed."
	read -n 1 -s -r -p "Press any key to continue"
	rustup target add $(TARGET_WIN)
	rustup target add $(TARGET_LINUX)
	rustup target add $(TARGET_MAC_INTEL)
	rustup target add $(TARGET_MAC_ARM64)
	@echo "Please install nodejs, npm, and tsc to compile the extension."
