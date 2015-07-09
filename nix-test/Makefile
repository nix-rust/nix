OUT    = libnixtest.a
CFLAGS = -fPIC -D$(OS) -m$(ARCH)
OBJS   = const.o \
		 sizes.o

$(OUT): $(OBJS)
	ar -rcs $@ $^

clean:
	rm -f $(OBJS)
	rm -f $(OUT)
