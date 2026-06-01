PREFIXES ?= rv32ui-p

.PHONY: build-tests

build-tests: build-riscv-tests

build-riscv-tests:
	cd core/data/riscv-tests && \
	git submodule update --init --recursive && \
	autoconf && \
	./configure --prefix=$$(pwd)/build && \
	make isa XLEN=32 && \
	mkdir -p ../bins/riscv && \
	$(foreach p,$(PREFIXES),find isa -maxdepth 1 -name '$(p)-*' ! -name '*.dump' -exec cp {} ../bins/riscv/ \; &&) true