BINS=charmap-cellphone_white_0.bin charmap-futuristic_black_0.bin charmap-oldschool_white.bin

fontgen: fontgen.rs libstb_image.a
	rustc -L. fontgen.rs

libstb_image.a: stb_image.o
	ar -crs libstb_image.a stb_image.o

stb_image.o: stb_image.h
	cc -x c -DSTB_IMAGE_IMPLEMENTATION -o stb_image.o -c stb_image.h

.PHONY: test
test: $(BINS)
	sha256sum -c SHA256SUM

%.bin: %.png fontgen
	./fontgen -f bin $< > $@
