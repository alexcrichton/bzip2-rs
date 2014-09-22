ifneq ($(findstring i686,$(TARGET)),)
export CFLAGS := $(CFLAGS) -m32 -ffunction-sections -fdata-sections
else
export CFLAGS := $(CFLAGS) -fPIC -m64 -ffunction-sections -fdata-sections
endif

all:
	rm -rf "$(OUT_DIR)/bzip2-1.0.6"
	cp -r bzip2-1.0.6 "$(OUT_DIR)"
	make -C "$(OUT_DIR)/bzip2-1.0.6" -j$(NUM_JOBS) libbz2.a
	cp "$(OUT_DIR)/bzip2-1.0.6/libbz2.a" "$(OUT_DIR)"
