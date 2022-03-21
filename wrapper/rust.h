#include <unistd.h>
#include <sys/stat.h>
#include <sys/statfs.h>
#include <sys/statvfs.h>
#include <dirent.h>
#include <stdbool.h>

extern int sfs_open(const char *, unsigned int, int);
extern int sfs_create(const char *, unsigned int);
extern int sfs_remove(const char *);
extern int sfs_access(const char *, int, bool);
extern int sfs_stat(const char *, struct stat*, bool);
extern int sfs_statfs(struct statfs*, bool);
extern int sfs_statvfs(struct statvfs*, bool);
extern int sfs_lseek(int, long, int);
extern int sfs_truncate(char*, long, long);
extern int sfs_dup(int);
extern int sfs_dup2(int, int);
extern int sfs_pwrite(int, const char*, long, long);
extern int sfs_write(int, const char*, long);
extern int sfs_pread(int, char*, long, long);
extern int sfs_read(int, char*, long);
extern int sfs_rmdir(const char*);
extern int sfs_opendir(const char*);
extern int sfs_getdents(int, struct dirent*, long);
extern int sfs_getdents64(int, struct dirent64*, long);
extern void init_environment();