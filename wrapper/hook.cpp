
#include <memory>
#include <limits>

extern "C" {
#include <libsyscall_intercept_hook_point.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <sys/statfs.h>
#include <syscall.h>
#include <stdio.h>
#include <string.h>
}
#include "hook.hpp"

#ifndef RUST_HPP
    #include "rust.hpp"
    #define RUST_HPP
#endif

int relativize_fd_path_wrapper(int dirfd, const char* cpath, std::string& resolved, bool follow_links = false){
    char* c_resolved = new char[255];
    int ret = relativize_fd_path(dirfd, cpath, c_resolved, follow_links);
    resolved = c_resolved;
    return ret;
}
bool relativize_path_wrapper(const char* path, std::string& rel_path, bool follow_links = false){
    char* c_rel_path = new char[255];
    int ret = relativize_path(path, c_rel_path, follow_links);
    rel_path = c_rel_path;
    return ret;
}
std::string get_path_wrapper(int fd){
    char* path = new char[255];
    fd_get_path(fd, path);
    return path;
}
std::string get_dir_path_wrapper(int fd){
    char* path = new char[255];
    fd_get_dir_path(fd, path);
    return path;
}
std::string get_mountdir_wrapper(){
    char* path = new char[255];
    get_mountdir(path);
    return path;
}
std::string get_ctx_cwd_wrapper(){
    char* cwd = new char[255];
    get_ctx_cwd(cwd);
    return cwd;
}
std::string get_sys_cwd_wrapper(){
    char* cwd = new char[255];
    get_sys_cwd(cwd);
    return cwd;
}

enum RelativizeStatus { internal = 0, external = 1, fd_unknown = 2, fd_not_a_dir = 3 };
namespace {

// TODO replace all internal gkfs errno variable usage with LEAF
inline int
with_errno(int ret) {
    return (ret < 0) ? -errno : ret;
}

} // namespace

bool
has_trailing_slash(const std::string& path) {
    return (!path.empty()) && (path.back() == '/');
}

template <class... Args>
inline long
syscall_no_intercept_wrapper(long syscall_number, Args... args) {
    long result;
    int error;
    result = syscall_no_intercept(syscall_number, args...);
    error = syscall_error_code(result);
    if(error != 0) {
        return -error;
    }
    return result;
}

int
hook_openat(int dirfd, const char* cpath, int flags, mode_t mode) {

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept(SYS_openat, dirfd, cpath, flags, mode);

        case RelativizeStatus::external:
            return syscall_no_intercept(SYS_openat, dirfd, resolved.c_str(),
                                        flags, mode);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return with_errno(sfs_open(resolved.c_str(), mode, flags));

        default:
            return -EINVAL;
    }
}

int
hook_close(int fd) {

    if(fd_exist(fd)) {
        // No call to the daemon is required
       fd_remove(fd);
        return 0;
    }

    if(fd_is_internal(fd)) {
        // the client application (for some reason) is trying to close an
        // internal fd: ignore it
        return 0;
    }

    return syscall_no_intercept_wrapper(SYS_close, fd);
}

int
hook_stat(const char* path, struct stat* buf) {

    std::string rel_path;
    if(relativize_path_wrapper(path, rel_path, false)) {
        return with_errno(sfs_stat(rel_path.c_str(), buf, false));
    }

    return syscall_no_intercept_wrapper(SYS_stat, rel_path.c_str(), buf);
}

int
hook_lstat(const char* path, struct stat* buf) {
    std::string rel_path;
    if(relativize_path_wrapper(path, rel_path)) {
        return with_errno(sfs_stat(rel_path.c_str(), buf, false));
    }
    return syscall_no_intercept_wrapper(SYS_lstat, rel_path.c_str(), buf);
}

int
hook_fstat(unsigned int fd, struct stat* buf) {

    if(fd_exist(fd)) {
        auto path = get_path_wrapper(fd);
        return with_errno(sfs_stat(path.c_str(), buf, false));
    }
    return syscall_no_intercept_wrapper(SYS_fstat, fd, buf);
}

int
hook_fstatat(int dirfd, const char* cpath, struct stat* buf, int flags) {

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_newfstatat, dirfd, cpath,
                                                buf, flags);

        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_newfstatat, dirfd,
                                                resolved.c_str(), buf, flags);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return with_errno(sfs_stat(resolved.c_str(), buf, false));

        default:
            return -EINVAL;
    }
}

int
hook_read(unsigned int fd, void* buf, size_t count) {
    

    if(fd_exist(fd)) {
        return with_errno(sfs_read(fd, (char *)buf, count));
    }
    return syscall_no_intercept_wrapper(SYS_read, fd, buf, count);
}

int
hook_pread(unsigned int fd, char* buf, size_t count, loff_t pos) {

    if(fd_exist(fd)) {
        return with_errno(sfs_pread(fd, buf, count, pos));
    }
    /* Since kernel 2.6: pread() became pread64(), and pwrite() became
     * pwrite64(). */
    return syscall_no_intercept_wrapper(SYS_pread64, fd, buf, count, pos);
}
/*
int
hook_readv(unsigned long fd, const struct iovec* iov, unsigned long iovcnt) {

    if(fd_exist(fd)) {
        return with_errno(sfs_readv(fd, iov, iovcnt));
    }
    return syscall_no_intercept_wrapper(SYS_readv, fd, iov, iovcnt);
}

int
hook_preadv(unsigned long fd, const struct iovec* iov, unsigned long iovcnt,
            unsigned long pos_l, unsigned long pos_h) {
                

    if(fd_exist(fd)) {
        return with_errno(sfs_preadv(fd, iov, iovcnt, pos_l));
    }
    return syscall_no_intercept_wrapper(SYS_preadv, fd, iov, iovcnt, pos_l);
}
*/
int
hook_write(unsigned int fd, const char* buf, size_t count) {
    

    if(fd_exist(fd)) {
        return with_errno(sfs_write(fd, buf, count));
    }
    return syscall_no_intercept_wrapper(SYS_write, fd, buf, count);
}

int
hook_pwrite(unsigned int fd, const char* buf, size_t count, loff_t pos) {
    

    if(fd_exist(fd)) {
        return with_errno(sfs_pwrite(fd, buf, count, pos));
    }
    /* Since kernel 2.6: pread() became pread64(), and pwrite() became
     * pwrite64(). */
    return syscall_no_intercept_wrapper(SYS_pwrite64, fd, buf, count, pos);
}
/*
int
hook_writev(unsigned long fd, const struct iovec* iov, unsigned long iovcnt) {

    if(fd_exist(fd)) {
        return with_errno(sfs_writev(fd, iov, iovcnt));
    }
    return syscall_no_intercept_wrapper(SYS_writev, fd, iov, iovcnt);
}

int
hook_pwritev(unsigned long fd, const struct iovec* iov, unsigned long iovcnt,
             unsigned long pos_l, unsigned long pos_h) {

    if(fd_exist(fd)) {
        return with_errno(sfs_pwritev(fd, iov, iovcnt, pos_l));
    }
    return syscall_no_intercept_wrapper(SYS_pwritev, fd, iov, iovcnt, pos_l);
}
*/
int
hook_unlinkat(int dirfd, const char* cpath, int flags) {

    if((flags & ~AT_REMOVEDIR) != 0) {
        return -EINVAL;
    }

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved, false);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_unlinkat, dirfd, cpath,
                                                flags);

        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_unlinkat, dirfd,
                                                resolved.c_str(), flags);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            if(flags & AT_REMOVEDIR) {
                return with_errno(sfs_rmdir(resolved.c_str()));
            } else {
                return with_errno(sfs_remove(resolved.c_str()));
            }

        default:
            return -EINVAL;
    }
}

int
hook_symlinkat(const char* oldname, int newdfd, const char* newname) {

    std::string oldname_resolved;
    if(relativize_path_wrapper(oldname, oldname_resolved)) {
        return -ENOTSUP;
    }

    std::string newname_resolved;
    auto rstatus =
            relativize_fd_path_wrapper(newdfd, newname, newname_resolved, false);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_symlinkat, oldname, newdfd,
                                                newname);

        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_symlinkat, oldname, newdfd,
                                                newname_resolved.c_str());

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return -ENOTSUP;

        default:
            return -EINVAL;
    }
}


int
hook_access(const char* path, int mask) {

    std::string rel_path;
    if(relativize_path_wrapper(path, rel_path)) {
        auto ret = sfs_access(rel_path.c_str(), mask, false);
        if(ret < 0) {
            return -errno;
        }
        return ret;
    }
    return syscall_no_intercept_wrapper(SYS_access, rel_path.c_str(), mask);
}

int
hook_faccessat(int dirfd, const char* cpath, int mode) {

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_faccessat, dirfd, cpath,
                                                mode);

        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_faccessat, dirfd,
                                                resolved.c_str(), mode);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return with_errno(sfs_access(resolved.c_str(), mode, false));

        default:
            return -EINVAL;
    }
}

off_t
hook_lseek(unsigned int fd, off_t offset, unsigned int whence) {

    if(fd_exist(fd)) {
        auto off_ret = sfs_lseek(
                fd, static_cast<off64_t>(offset), whence);
        if(off_ret > std::numeric_limits<off_t>::max()) {
            return -EOVERFLOW;
        } else if(off_ret < 0) {
            return -errno;
        }
        return off_ret;
    }
    return syscall_no_intercept_wrapper(SYS_lseek, fd, offset, whence);
}

int
hook_truncate(const char* path, long length) {

    std::string rel_path;
    if(relativize_path_wrapper(path, rel_path)) {
        return with_errno(sfs_truncate(rel_path.c_str(), length));
    }
    return syscall_no_intercept_wrapper(SYS_truncate, rel_path.c_str(), length);
}

int
hook_ftruncate(unsigned int fd, unsigned long length) {

    if(fd_exist(fd)) {
        auto path = get_path_wrapper(fd);
        return with_errno(sfs_truncate(path.c_str(), length));
    }
    return syscall_no_intercept_wrapper(SYS_ftruncate, fd, length);
}

int
hook_dup(unsigned int fd) {

    if(fd_exist(fd)) {
        return with_errno(sfs_dup(fd));
    }
    return syscall_no_intercept_wrapper(SYS_dup, fd);
}

int
hook_dup2(unsigned int oldfd, unsigned int newfd) {

    if(fd_exist(oldfd)) {
        return with_errno(sfs_dup2(oldfd, newfd));
    }
    return syscall_no_intercept_wrapper(SYS_dup2, oldfd, newfd);
}

int
hook_dup3(unsigned int oldfd, unsigned int newfd, int flags) {

    if(fd_exist(oldfd)) {
        // TODO implement O_CLOEXEC flag first which is used with fcntl(2)
        // It is in glibc since kernel 2.9. So maybe not that important :)
        return -ENOTSUP;
    }
    return syscall_no_intercept_wrapper(SYS_dup3, oldfd, newfd, flags);
}

int
hook_getdents(unsigned int fd, struct linux_dirent* dirp, unsigned int count) {
    if(fd_exist(fd)) {
        return with_errno(sfs_getdents(fd, dirp, count));
    }
    return syscall_no_intercept_wrapper(SYS_getdents, fd, dirp, count);
}


int
hook_getdents64(unsigned int fd, struct linux_dirent64* dirp,
                unsigned int count) {
    if(fd_exist(fd)) {
        return with_errno(sfs_getdents64(fd, dirp, count));
    }
    return syscall_no_intercept_wrapper(SYS_getdents64, fd, dirp, count);
}


int
hook_mkdirat(int dirfd, const char* cpath, mode_t mode) {

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved);
    switch(rstatus) {
        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_mkdirat, dirfd,
                                                resolved.c_str(), mode);

        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_mkdirat, dirfd, cpath,
                                                mode);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return with_errno(
                    sfs_create(resolved.c_str(), mode | S_IFDIR));

        default:
            return -EINVAL;
    }
}

int
hook_fchmodat(int dirfd, const char* cpath, mode_t mode) {

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_fchmodat, dirfd, cpath,
                                                mode);

        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_fchmodat, dirfd,
                                                resolved.c_str(), mode);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return -ENOTSUP;

        default:
            return -EINVAL;
    }
}

int
hook_fchmod(unsigned int fd, mode_t mode) {

    if(fd_exist(fd)) {
        return -ENOTSUP;
    }
    return syscall_no_intercept_wrapper(SYS_fchmod, fd, mode);
}

int
hook_chdir(const char* path) {

    std::string rel_path;
    bool internal = relativize_path_wrapper(path, rel_path);
    if(internal) {
        if(!S_ISDIR(get_md_mode(rel_path.c_str()))) {
            return -ENOTDIR;
        }
        // TODO get complete path from relativize_path_wrapper instead of
        // removing mountdir and then adding again here
        rel_path.insert(0, get_mountdir_wrapper());
        if(has_trailing_slash(rel_path)) {
            // open_dir is '/'
            rel_path.pop_back();
        }
    }
    set_cwd(rel_path.c_str(), internal);
    return 0;
}

int
hook_fchdir(unsigned int fd) {

    if(fd_exist(fd)) {

        std::string new_path = get_mountdir_wrapper() + get_dir_path_wrapper(fd);
        if(has_trailing_slash(new_path)) {
            // open_dir is '/'
            new_path.pop_back();
        }
        set_cwd(new_path.c_str(), true);
    } else {
        long ret = syscall_no_intercept_wrapper(SYS_fchdir, fd);
        if(ret < 0) {
            return -1;
        }
        unset_env_cwd();
        set_ctx_cwd(get_sys_cwd_wrapper().c_str());
    }
    return 0;
}

int
hook_getcwd(char* buf, unsigned long size) {
    if(get_ctx_cwd_wrapper().size() + 1 > size) {
        return -ERANGE;
    }

    strcpy(buf, get_ctx_cwd_wrapper().c_str());
    return (get_ctx_cwd_wrapper().size() + 1);
}

int
hook_readlinkat(int dirfd, const char* cpath, char* buf, int bufsiz) {

    std::string resolved;
    auto rstatus = relativize_fd_path_wrapper(dirfd, cpath, resolved, false);
    switch(rstatus) {
        case RelativizeStatus::fd_unknown:
            return syscall_no_intercept_wrapper(SYS_readlinkat, dirfd, cpath,
                                                buf, bufsiz);

        case RelativizeStatus::external:
            return syscall_no_intercept_wrapper(SYS_readlinkat, dirfd,
                                                resolved.c_str(), buf, bufsiz);

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return -ENOTSUP;

        default:
            return -EINVAL;
    }
}

int
hook_fcntl(unsigned int fd, unsigned int cmd, unsigned long arg) {

    if(!fd_exist(fd)) {
        return syscall_no_intercept_wrapper(SYS_fcntl, fd, cmd, arg);
    }
    int ret;
    switch(cmd) {

        case F_DUPFD:
            return with_errno(sfs_dup(fd));

        case F_DUPFD_CLOEXEC:
            ret = sfs_dup(fd);
            if(ret == -1) {
                return -errno;
            }
            set_flag(fd, 6, true);
            return ret;

        case F_GETFD:
            if(get_flag(fd, 6)) {
                return FD_CLOEXEC;
            }
            return 0;

        case F_GETFL:
            ret = 0;
            if(get_flag(fd, 3)) {
                ret |= O_RDONLY;
            }
            if(get_flag(fd, 4)) {
                ret |= O_WRONLY;
            }
            if(get_flag(fd, 5)) {
                ret |= O_RDWR;
            }
            return ret;

        case F_SETFD:
            set_flag(fd, 6, (arg & FD_CLOEXEC));
            return 0;


        default:
            return -ENOTSUP;
    }
}

int
hook_renameat(int olddfd, const char* oldname, int newdfd, const char* newname,
              unsigned int flags) {

    const char* oldpath_pass;
    std::string oldpath_resolved;
    auto oldpath_status =
            relativize_fd_path_wrapper(olddfd, oldname, oldpath_resolved);
    switch(oldpath_status) {
        case RelativizeStatus::fd_unknown:
            oldpath_pass = oldname;
            break;

        case RelativizeStatus::external:
            oldpath_pass = oldpath_resolved.c_str();
            break;

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return -ENOTSUP;

        default:
            return -EINVAL;
    }

    const char* newpath_pass;
    std::string newpath_resolved;
    auto newpath_status =
            relativize_fd_path_wrapper(newdfd, newname, newpath_resolved);
    switch(newpath_status) {
        case RelativizeStatus::fd_unknown:
            newpath_pass = newname;
            break;

        case RelativizeStatus::external:
            newpath_pass = newpath_resolved.c_str();
            break;

        case RelativizeStatus::fd_not_a_dir:
            return -ENOTDIR;

        case RelativizeStatus::internal:
            return -ENOTSUP;

        default:
            return -EINVAL;
    }

    return syscall_no_intercept_wrapper(SYS_renameat2, olddfd, oldpath_pass,
                                        newdfd, newpath_pass, flags);
}

int
hook_statfs(const char* path, struct statfs* buf) {

    std::string rel_path;
    if(relativize_path_wrapper(path, rel_path)) {
        return with_errno(sfs_statfs(buf));
    }
    return syscall_no_intercept_wrapper(SYS_statfs, rel_path.c_str(), buf);
}

int
hook_fstatfs(unsigned int fd, struct statfs* buf) {

    if(fd_exist(fd)) {
        return with_errno(sfs_statfs(buf));
    }
    return syscall_no_intercept_wrapper(SYS_fstatfs, fd, buf);
}

/* The function should broadcast a flush message (pmem_persist i.e.) if the
 * application needs the capabilities*/
int
hook_fsync(unsigned int fd) {

    if(fd_exist(fd)) {
        errno = 0;
        return 0;
    }

    return syscall_no_intercept_wrapper(SYS_fsync, fd);
}

int
hook_getxattr(const char* path, const char* name, void* value, size_t size) {

    std::string rel_path;
    if(relativize_path_wrapper(path, rel_path)) {
        return -ENOTSUP;
    }
    return syscall_no_intercept_wrapper(SYS_getxattr, path, name, value, size);
}