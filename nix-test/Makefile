OUT    = libnixtest.a
CFLAGS = -fPIC -D$(OS)
OBJS   = errno.o

$(OUT): $(OBJS)
	ar -rcs $@ $^

clean:
	rm -f $(OBJS)
	rm -f $(OUT)
