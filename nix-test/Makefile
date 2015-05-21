OUT    = libnixtest.a
CFLAGS = -fPIC -D$(OS)
OBJS   = const.o \
		 sizes.o

$(OUT): $(OBJS)
	ar -rcs $@ $^

clean:
	rm -f $(OBJS)
	rm -f $(OUT)
