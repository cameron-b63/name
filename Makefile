AS_DIR = name-as
LD_DIR = name-ld
EMU_DIR = name-emu
EXT_DIR = name-ext

BIN_DIR = $(EXT_DIR)/bin

all: build compile

build:
	mkdir -p $(BIN_DIR)
	cd $(AS_DIR) && cargo build --release
	mv -n target/release/name-as $(BIN_DIR)/
	cd $(LD_DIR) && cargo build --release
	mv -n target/release/name-ld $(BIN_DIR)/
	cd $(EMU_DIR) && cargo build --release
	mv -n target/release/name-emu $(BIN_DIR)/

compile:
	cd $(EXT_DIR) && npm run compile

purge:
	rm -rf $(EXT_DIR)/bin