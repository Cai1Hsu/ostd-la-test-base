# ostd for LoongArch test base

This repository contains the minimal test base for LoongArch OSTD.

To test the LoongArch OSTD, following the steps below:

1. Clone asterinas repository in this repository, eg:

```bash
git clone https://github.com/cai1hsu/asterinas -b feat/loongarch-minimal-support
```

2. Patch the PRs that are not merged into the asterinas repository.

I have made a makefile target to do this, you can run the following command:

```bash
make patch
```

3. Build the kernel:
```bash
make build
```

4. Run the kernel:
```bash
make run
```

## Test with osdk

When testing with osdk, the run base(my_kernel-run-base) is not needed as the osdk generates a run base for you.

1. Build osdk from the repo you just cloned

```bash
cd asterinas/osdk
cargo build
```

2. Go to the kernel crate and build it with osdk

```bash
cd my_kernel
../asterinas/osdk/target/debug/cargo-osdk osdk build \
    --target-arch loongarch64 \
    --boot-method qemu-direct
```

The build artifact is located in `target/osdk/my_kernel-osdk-bin.qemu_elf`

3. Run the kernel with QEMU

```bash
qemu-system-loongarch64 \
    -machine virt \
    -nographic \
    -no-reboot \
    -m 1G \
    -kernel target/osdk/my_kernel-osdk-bin.qemu_elf
```
