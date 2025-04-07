ASTERINAS_DIR := ./asterinas

PATCHES := \
	https://github.com/Cai1Hsu/asterinas/pull/1.patch \
	https://github.com/oscomp/asterinas/pull/7.patch \
	https://github.com/oscomp/asterinas/pull/3.patch \

patch:
	@echo "Applying patches to $(ASTERINAS_DIR)"
	@cd $(ASTERINAS_DIR) && { \
		for p in $(PATCHES); do \
			echo "Downloading and applying $$p"; \
			curl -sSL $$p | patch -p1; \
		done \
	}

build:
	cp asterinas/osdk/src/base_crate/loongarch64.ld.template my_kernel-run-base/loongarch64.ld
	cd my_kernel-run-base && RUSTFLAGS="-C link-arg=-Tloongarch64.ld -C relocation-model=static -C relro-level=off --check-cfg cfg(ktest) -C no-redzone=y" \
		cargo build \
			--target loongarch64-unknown-none \
			-Zbuild-std=core,alloc,compiler_builtins \
			-Zbuild-std-features=compiler-builtins-mem \
			--profile=dev
	cp my_kernel-run-base/target/loongarch64-unknown-none/debug/my_kernel-osdk-bin kernel.bin

run:
	qemu-system-loongarch64 \
        -machine virt \
        -nographic \
        -no-reboot \
        -m 1G \
        -kernel kernel.bin

clean:
	cd my_kernel-run-base && cargo clean
	cd my_kernel && cargo clean
	cd asterinas && cargo clean
	cd asterinas && git reset --hard HEAD
