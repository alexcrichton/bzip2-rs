ifneq ($(findstring i686,$(TARGET)),)
CFLAGS := $(CFLAGS) -m32
else
CFLAGS := $(CFLAGS) -fPIC -m64
endif

CFLAGS := $(CFLAGS) -DBZ_NO_STDIO -g -O2 -ffunction-sections -fdata-sections

all:
	rm -rf "$(OUT_DIR)/bzip2-1.0.6"
	cp -r bzip2-1.0.6 "$(OUT_DIR)"
	$(MAKE) -C "$(OUT_DIR)/bzip2-1.0.6" -j$(NUM_JOBS) libbz2.a \
		CFLAGS="$(CFLAGS)" CC="$(CC)" AR="$(AR)"
	cp "$(OUT_DIR)/bzip2-1.0.6/libbz2.a" "$(OUT_DIR)"
