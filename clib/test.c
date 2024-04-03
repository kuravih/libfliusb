#include "libfli.h"
#include <stdio.h>
#include <stdlib.h>

int main()
{
    char **list = NULL;
    int res = FLIList(FLIDOMAIN_USB | FLIDEVICE_CAMERA, &list);
    if (res < 0)
    {
        printf("FLIList failed with error code %d\n", res);
        goto cleanup;
    }
    int count = 0;
    while (list[count])
    {
        printf("Found device: %s\n", list[count]);
        count++;
    }
    if (count > 0)
    {
        flidev_t dev;
        res = FLIOpen(&dev, "FLI-04", FLIDOMAIN_USB | FLIDEVICE_CAMERA);
        if (res < 0)
        {
            printf("FLIOpen failed with error code %d\n", res);
            goto cleanup;
        }
        printf("Opened device: %s\n", list[0]);

        int res = FLISetExposureTime(dev, 10000);
        if (res < 0)
        {
            printf("FLISetExposureTime failed with error code %d\n", res);
            goto close;
        }
        res = FLISetFrameType(dev, FLI_FRAME_TYPE_NORMAL);
        if (res < 0)
        {
            printf("FLISetFrameType failed with error code %d\n", res);
            goto close;
        }
        long w, h, ox, oy, bx, by;
        res = FLIGetReadoutDimensions(dev, &w, &ox, &bx, &h, &oy, &by);
        if (res < 0)
        {
            printf("FLIGetReadoutDimensions failed with error code %d\n", res);
            goto close;
        }
        printf("Readout dimensions: %ldx%ld, offset: %ldx%ld, binned: %ldx%ld\n", w, h, ox, oy, bx, by);
        res = FLISetImageArea(dev, ox, oy, w, h);
        if (res < 0)
        {
            printf("FLISetImageArea failed with error code %d\n", res);
            goto close;
        }
        res = FLISetHBin(dev, 1);
        if (res < 0)
        {
            printf("FLISetHBin failed with error code %d\n", res);
            goto close;
        }
        res = FLISetVBin(dev, 1);
        if (res < 0)
        {
            printf("FLISetVBin failed with error code %d\n", res);
            goto close;
        }
        unsigned short *buf = malloc(w * h * sizeof(unsigned short));
        if (!buf)
        {
            printf("Failed to allocate buffer\n");
            goto close;
        }
        res = FLIExposeFrame(dev);
        if (res < 0)
        {
            printf("FLIExposeFrame failed with error code %d\n", res);
            goto freemem;
        }
        long timeout = 1000;
        while (1)
        {
            sleep(1);
            res = FLIGetExposureStatus(dev, &timeout);
            if (res != 0)
            {
                printf("FLIGetExposureStatus failed with error code %d\n", res);
                goto freemem;
            }
            else
            {
                printf("Exposure remaining: %ld\n", timeout);
            }
            if (timeout == 0)
            {
                break;
            }
        }
        res = FLIEndExposure(dev);
        if (res < 0)
        {
            printf("FLIEndExposure failed with error code %d\n", res);
            goto freemem;
        }
        size_t nread = 0;
        res = FLIGrabFrame(dev, buf, w * h * 2, &nread);
        if (res < 0)
        {
            printf("FLIGrabFrame failed with error code %d\n", res);
            goto freemem;
        }
        printf("Read %ld bytes\n", nread);
        FILE *f = fopen("image.raw", "wb");
        if (!f)
        {
            printf("Failed to open file\n");
            goto freemem;
        }
        size_t written = fwrite(buf, 1, nread, f);
        if (written != nread)
        {
            printf("Failed to write to file\n");
            fclose(f);
            goto freemem;
        }
        fclose(f);
freemem:
        free(buf);
close:
        FLIClose(dev);
    }
cleanup:
    if (list)
    {
        FLIFreeList(list);
    }
}