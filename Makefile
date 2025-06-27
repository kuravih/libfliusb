CC = gcc
EDCFLAGS = -Wall -O2 -D__LIBUSB__ -pthread -I ./ -I ./unix $(CFLAGS)
EDLDFLAGS = -lusb-1.0 -lpthread -lm $(LDFLAGS)

SRCS = libfli.o libfli-camera.o libfli-camera-parport.o libfli-camera-usb.o libfli-mem.o libfli-raw.o libfli-filter-focuser.o unix/libfli-usb.o unix/libfli-debug.o unix/libfli-serial.o unix/libfli-sys.o unix/libusb/libfli-usb-sys.o

OBJS = $(SRCS:.c=.o)

LIBTARGET = libfli-usb.a

all: $(LIBTARGET)

$(LIBTARGET): $(OBJS)
	ar rcs $@ $(OBJS)

%.o: %.c
	$(CC) -c -o $@ $< $(EDCFLAGS)

clean:
	rm -f $(OBJS) $(LIBTARGET)
