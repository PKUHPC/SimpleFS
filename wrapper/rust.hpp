#include <unistd.h>
#include <sys/stat.h>
#include <sys/statfs.h>
#include <sys/statvfs.h>
#include <dirent.h>
#include <stdbool.h>
#include <fcntl.h>
#include <sys/types.h>
#include <string>

#ifndef TYPE_DEF
  #define TYPE_DEF
  extern "C" {
  #include "mytype.h"
  }
#endif

extern "C" int sfs_open(const char *, unsigned int, int);
extern "C" int sfs_create(const char *, unsigned int);
extern "C" int sfs_remove(const char *);
extern "C" int sfs_access(const char *, int, bool);
extern "C" int sfs_stat(const char *, struct stat*, bool);
extern "C" int sfs_statfs(struct statfs*);
extern "C" int sfs_statvfs(struct statvfs*);
extern "C" int sfs_lseek(int, long, int);
extern "C" int sfs_truncate(const char*, long);
extern "C" int sfs_dup(int);
extern "C" int sfs_dup2(int, int);
extern "C" int sfs_pwrite(int, const char*, long, long);
extern "C" int sfs_write(int, const char*, long);
extern "C" int sfs_pread(int, char*, long, long);
extern "C" int sfs_read(int, char*, long);
extern "C" int sfs_rmdir(const char*);
extern "C" int sfs_opendir(const char*);
extern "C" int sfs_getdents(int, struct linux_dirent*, long);
extern "C" int sfs_getdents64(int, struct linux_dirent64*, long);

extern "C" int relativize_fd_path(int dirfd, const char* cpath, char* resolved, bool follow_links);
extern "C" bool relativize_path(const char* path, char* rel_path, bool follow_links);
extern "C" bool fd_exist(int fd);
extern "C" void fd_remove(int fd);
extern "C" bool fd_is_internal(int fd);
extern "C" void fd_get_path(int fd, char* path);
extern "C" void fd_get_dir_path(int fd, char* path);
extern "C" void set_flag(int fd, int flag, bool val);
extern "C" bool get_flag(int fd, int flag);
extern "C" void get_mountdir(char* path);
extern "C" void get_ctx_cwd(char* cwd);
extern "C" void set_ctx_cwd(const char* cwd);
extern "C" void set_cwd(const char* cwd, bool internal);
extern "C" void unset_env_cwd();
extern "C" void get_sys_cwd(char* cwd);
extern "C" int get_md_mode(const char* path);
extern "C" bool intercept_enabled();


/*
extern "C" int hook_openat(int dirfd, const char* cpath, int flags, mode_t mode);
extern "C" int hook_close(int fd);
extern "C" int hook_stat(const char* path, struct stat* buf);
extern "C" int hook_lstat(const char* path, struct stat* buf);
extern "C" int hook_fstat(unsigned int fd, struct stat* buf);
extern "C" int hook_fstatat(int dirfd, const char* cpath, struct stat* buf, int flags);
extern "C" int hook_read(unsigned int fd, void* buf, size_t count);
extern "C" int hook_pread(unsigned int fd, char* buf, size_t count, loff_t pos);
extern "C" int hook_readv(unsigned long fd, const struct iovec* iov, unsigned long iovcnt);
extern "C" int hook_preadv(unsigned long fd, const struct iovec* iov, unsigned long iovcnt,
            unsigned long pos_l, unsigned long pos_h);
extern "C" int hook_write(unsigned int fd, const char* buf, size_t count);
extern "C" int hook_pwrite(unsigned int fd, const char* buf, size_t count, loff_t pos);
extern "C" int hook_writev(unsigned long fd, const struct iovec* iov, unsigned long iovcnt);
extern "C" int hook_pwritev(unsigned long fd, const struct iovec* iov, unsigned long iovcnt,
             unsigned long pos_l, unsigned long pos_h);
extern "C" int hook_unlinkat(int dirfd, const char* cpath, int flags);
extern "C" int hook_symlinkat(const char* oldname, int newdfd, const char* newname);
extern "C" int hook_access(const char* path, int mask);
extern "C" int hook_faccessat(int dirfd, const char* cpath, int mode);
extern "C" off_t hook_lseek(unsigned int fd, off_t offset, unsigned int whence);
extern "C" int hook_truncate(const char* path, long length);
extern "C" int hook_ftruncate(unsigned int fd, unsigned long length);
extern "C" int hook_dup(unsigned int fd);
extern "C" int hook_dup2(unsigned int oldfd, unsigned int newfd);
extern "C" int hook_dup3(unsigned int oldfd, unsigned int newfd, int flags);
extern "C" int hook_getdents(unsigned int fd, struct linux_dirent* dirp, unsigned int count);
extern "C" int hook_getdents64(unsigned int fd, struct linux_dirent64* dirp, unsigned int count);
extern "C" int hook_mkdirat(int dirfd, const char* cpath, mode_t mode);
extern "C" int hook_fchmodat(int dirfd, const char* cpath, mode_t mode);
extern "C" int hook_fchmod(unsigned int fd, mode_t mode);
extern "C" int hook_chdir(const char* path);
extern "C" int hook_fchdir(unsigned int fd);
extern "C" int hook_getcwd(char* buf, unsigned long size);
extern "C" int hook_readlinkat(int dirfd, const char* cpath, char* buf, int bufsiz);
extern "C" int hook_fcntl(unsigned int fd, unsigned int cmd, unsigned long arg);
extern "C" int hook_renameat(int olddfd, const char* oldname, int newdfd, const char* newname, unsigned int flags);
extern "C" int hook_statfs(const char* path, struct statfs* buf);
extern "C" int hook_fstatfs(unsigned int fd, struct statfs* buf);
extern "C" int hook_fsync(unsigned int fd);
extern "C" int hook_getxattr(const char* path, const char* name, void* value, size_t size);
*/