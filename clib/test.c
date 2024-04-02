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
        FLIClose(dev);
    }
cleanup:
    if (list)
    {
        FLIFreeList(list);
    }
}