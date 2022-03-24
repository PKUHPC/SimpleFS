#ifndef TYPE_DEF
  #define TYPE_DEF
  extern "C" {
  #include "mytype.h"
  }
#endif


#ifndef RUST_HPP
    #include "rust.hpp"
    #define RUST_HPP
#endif

extern "C" {
#include <sys/types.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <dirent.h>
#include <linux/kernel.h>
#include <sys/statfs.h>
#include <sys/statvfs.h>
}

struct statfs;

int
hook_openat(int dirfd, const char* cpath, int flags, mode_t mode);

int
hook_close(int fd);

int
hook_stat(const char* path, struct stat* buf);

#ifdef STATX_TYPE
int
hook_statx(int dirfd, const char* path, int flags, unsigned int mask,
           struct statx* buf);
#endif

int
hook_lstat(const char* path, struct stat* buf);

int
hook_fstat(unsigned int fd, struct stat* buf);

int
hook_fstatat(int dirfd, const char* cpath, struct stat* buf, int flags);

int
hook_read(unsigned int fd, void* buf, size_t count);

int
hook_pread(unsigned int fd, char* buf, size_t count, loff_t pos);

int
hook_readv(unsigned long fd, const struct iovec* iov, unsigned long iovcnt);

int
hook_preadv(unsigned long fd, const struct iovec* iov, unsigned long iovcnt,
            unsigned long pos_l, unsigned long pos_h);

int
hook_write(unsigned int fd, const char* buf, size_t count);

int
hook_pwrite(unsigned int fd, const char* buf, size_t count, loff_t pos);

int
hook_writev(unsigned long fd, const struct iovec* iov, unsigned long iovcnt);

int
hook_pwritev(unsigned long fd, const struct iovec* iov, unsigned long iovcnt,
             unsigned long pos_l, unsigned long pos_h);

int
hook_unlinkat(int dirfd, const char* cpath, int flags);

int
hook_symlinkat(const char* oldname, int newdfd, const char* newname);

int
hook_access(const char* path, int mask);

int
hook_faccessat(int dirfd, const char* cpath, int mode);

#ifdef SYS_faccessat2
int
hook_faccessat2(int dirfd, const char* cpath, int mode, int flags);
#endif

off_t
hook_lseek(unsigned int fd, off_t offset, unsigned int whence);

int
hook_truncate(const char* path, long length);

int
hook_ftruncate(unsigned int fd, unsigned long length);

int
hook_dup(unsigned int fd);

int
hook_dup2(unsigned int oldfd, unsigned int newfd);

int
hook_dup3(unsigned int oldfd, unsigned int newfd, int flags);

int
hook_getdents(unsigned int fd, struct linux_dirent* dirp, unsigned int count);

int
hook_getdents64(unsigned int fd, struct linux_dirent64* dirp,
                unsigned int count);

int
hook_mkdirat(int dirfd, const char* cpath, mode_t mode);

int
hook_fchmodat(int dirfd, const char* path, mode_t mode);

int
hook_fchmod(unsigned int dirfd, mode_t mode);

int
hook_chdir(const char* path);

int
hook_fchdir(unsigned int fd);

int
hook_getcwd(char* buf, unsigned long size);

int
hook_readlinkat(int dirfd, const char* cpath, char* buf, int bufsiz);

int
hook_fcntl(unsigned int fd, unsigned int cmd, unsigned long arg);

int
hook_renameat(int olddfd, const char* oldname, int newdfd, const char* newname,
              unsigned int flags);

int
hook_statfs(const char* path, struct statfs* buf);

int
hook_fstatfs(unsigned int fd, struct statfs* buf);

int
hook_fsync(unsigned int fd);

int
hook_getxattr(const char* path, const char* name, void* value, size_t size);

