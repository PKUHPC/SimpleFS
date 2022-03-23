
typedef long int int64_t;
typedef unsigned long int uint64_t;

struct linux_dirent {
    unsigned long d_ino;
    unsigned long d_off;
    unsigned short d_reclen;
    char d_name[1];
};
/*
 * linux_dirent64 is used in getdents64() and defined in the linux kernel:
 * include/linux/dirent.h. However, it is not part of the kernel-headers and
 * cannot be imported.
 */
struct linux_dirent64 {
    uint64_t d_ino;
    int64_t d_off;
    unsigned short d_reclen;
    unsigned char d_type;
    char d_name[1]; // originally `char d_name[0]` in kernel, but ISO C++
                    // forbids zero-size array 'd_name'
};