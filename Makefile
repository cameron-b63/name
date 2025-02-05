AS_DIR = name-as/
LD_DIR = name-ld/
EMU_DIR = name-emu/

build:
	cd $(AS_DIR) && cargo build --release
	mv target/release/name-as bin/
	cd $(LD_DIR) && cargo build --release
	mv target/release/name-ld bin/
	cd $(EMU_DIR) && cargo build --release
	mv target/release/name-emu bin/
