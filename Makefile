TARGET=x86_64-unknown-none
NAME=$(shell echo "${PWD##*/}")
ROOT=./generated
RUSTFLAGS=\
		  -C link-args=-e \
		  -C link-args=entry \
		  -C link-args=-z \
		  -C link-args=execstack
CARGO=RUSTFLAGS='$(RUSTFLAGS)' cargo
BIN_PATH_DEBUG=$(shell cargo metadata --format-version 1 | jq -r .target_directory)/debug/$(NAME)
APP_BUILD_ARG=-v --target $(TARGET) --release

.PHONY : build
build :
	rustup target add x86_64-unknown-none
	$(CARGO) build $(APP_BUILD_ARG)

.PHONY : test
test :
	rustup target add x86_64-unknown-none
	$(CARGO)  test -v --target $(TARGET)
.PHONY : clippy
clippy :
	rustup target add $(TARGET)
	cargo clippy --all-features --target=$(TARGET) -- -D warnings
	cargo clippy --all-features -- -D warnings

.PHONY : objdump
objdump :
	cargo install cargo-binutils
	rustup component add llvm-tools-preview
	$(CARGO) objdump -- -d

.PHONY : nm
nm :
	cargo install cargo-binutils
	rustup component add llvm-tools-preview
	$(CARGO) nm

.PHONY : readelf
readelf : build
	readelf -l $(BIN_PATH_DEBUG)

.PHONY : hexdump
hexdump : build
	hexdump -C $(BIN_PATH_DEBUG)

.PHONY : run
run :
	make -C ../../ run

.PHONY : clean
clean :
	rm -rf target
	rm -rf Cargo.lock
	rm -rf ./core/target
	rm -rf ./core/Cargo.lock
	rm -rf $(ROOT)
	rm -rf build
	cargo clean
