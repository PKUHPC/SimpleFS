
typedef long int int64_t;
typedef unsigned long int uint64_t;

struct linux_dirent {
    unsigned long  d_ino;
    unsigned long int  d_off;
    unsigned short d_reclen;
    char d_name[];
};

struct linux_dirent64 {
    uint64_t d_ino;
    int64_t d_off;
    unsigned short int d_reclen;
    unsigned char d_type;
    char d_name[256];
};