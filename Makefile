GHC_VERSION := $(shell ghc --numeric-version)
GHC_LIB_DIR := $(shell ghc --print-libdir)

.PHONY: all clean bench compile-hs compile-go compile-go-wasm compile-go-tinygo compile-nim compile-zig 

all: compile-hs compile-go compile-go-wasm compile-go-tinygo compile-nim compile-zig 

compile-hs: foreign/hs/Add.hs
	cd foreign/hs && \
		ghc \
		-shared \
		-dynamic \
		-fPIC \
		Add.hs \
		-o libadd_hs.so \
		-no-hs-main \
		-L$(GHC_LIB_DIR)/lib/x86_64-linux-ghc$(GHC_VERSION) \
		-lHSrts-1.0.2_thr-ghc$(GHC_VERSION) \
		-I../_include

compile-go: foreign/go/add.go
	cd foreign/go && \
		CGO_CFLAGS="-I../_include" \
		go build \
		-ldflags="-s -w" \
		-buildmode=c-shared \
		-o libadd_go.so \
		add.go

compile-go-wasm: foreign/go/add_wasm.go
	cd foreign/go && \
		GOOS=wasip1 GOARCH=wasm \
		go build \
		-buildmode=c-shared \
		-o libadd_go.wasm \
		add_wasm.go

compile-go-tinygo: foreign/go/add_tinygo.go
	cd foreign/go && \
		tinygo build \
		-target=wasip1 \
		-o libadd_tinygo.wasm \
		add_tinygo.go

compile-nim: foreign/nim/add.nim
	nim c \
		--app:lib \
		--out:foreign/nim/libadd_nim.so \
		foreign/nim/add.nim

compile-zig: foreign/zig/add.zig
	cd foreign/zig && \
		zig build && \
		cp zig-out/lib/libadd_zig.so ./

bench: all
	cargo bench

clean:
	cd foreign/hs && rm -f *.hi *.o *_stub.[ch] *.so
	cd foreign/go && rm -f *.h *.so *.wasm
	cd foreign/nim && rm -f *.so
	cd foreign/zig && rm -f *.so && rm -rf .zig-cache zig-out

clean-all: clean
	rm -rf benches_result/
	cargo clean


