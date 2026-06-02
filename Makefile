PREFIXES ?= rv32ui-p #rv32mi-p

.PHONY: build-tests

build-tests: build-riscv-tests

build-riscv-tests:
	cd core/data/riscv-tests && \
	git submodule update --init --recursive && \
	autoconf && \
	./configure --prefix=$$(pwd)/build && \
	make -k isa XLEN=32 || true && \
	mkdir -p ../bins/riscv && \
	$(foreach p,$(PREFIXES),find isa -maxdepth 1 -name '$(p)-*' ! -name '*.dump' -exec cp {} ../bins/riscv/ \; &&) true
	@for p in $(PREFIXES); do \
		if ! ls core/data/bins/riscv/$$p-* >/dev/null 2>&1; then \
			echo "ERROR: no binaries found for prefix $$p"; exit 1; \
		fi; \
	done