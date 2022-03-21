#include <fcntl.h>
#include <stdio.h>
#include "rust.h"

int main(){
    init_environment();
    sfs_create("/sfs", __S_IFDIR);
    int fd = sfs_open("/sfs/file1", __S_IFREG, O_CREAT | O_RDWR);
    printf("%d\n", fd);
}