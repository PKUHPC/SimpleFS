extern "C"{
    #include <libsyscall_intercept_hook_point.h>
    #include <syscall.h>
    #include <errno.h>
    #include <stdio.h>
    #include <sys/types.h>
    #include <sys/socket.h>
}
#include <optional>
#include "hook.hpp"

static int
hook(long syscall_number,
			long arg0, long arg1,
			long arg2, long arg3,
			long arg4, long arg5,
			long *result)
{
        if(!intercept_enabled()){
                return 1;
        }
	switch(syscall_number) {

        case SYS_execve:
            *result = syscall_no_intercept(
                    syscall_number, reinterpret_cast<const char*>(arg0),
                    reinterpret_cast<const char* const*>(arg1),
                    reinterpret_cast<const char* const*>(arg2));
            break;

#ifdef SYS_execveat
        case SYS_execveat:
            *result = syscall_no_intercept(
                    syscall_number, arg0, reinterpret_cast<const char*>(arg1),
                    reinterpret_cast<const char* const*>(arg2),
                    reinterpret_cast<const char* const*>(arg3), arg4);
            break;
#endif

        case SYS_open:
            *result = hook_openat(
                    AT_FDCWD, reinterpret_cast<char*>(arg0),
                    static_cast<int>(arg1), static_cast<mode_t>(arg2));
            break;

        case SYS_creat:
            *result = hook_openat(
                    AT_FDCWD, reinterpret_cast<const char*>(arg0),
                    O_WRONLY | O_CREAT | O_TRUNC, static_cast<mode_t>(arg1));
            break;
        case SYS_openat:
            *result = hook_openat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    static_cast<int>(arg2), static_cast<mode_t>(arg3));
            break;
        case SYS_close:
            *result = hook_close(static_cast<int>(arg0));
            break;
        case SYS_stat:
            *result =
                    hook_stat(reinterpret_cast<char*>(arg0),
                                          reinterpret_cast<struct stat*>(arg1));
            break;
#ifdef STATX_TYPE
        case SYS_statx:
            *result = hook_statx(
                    static_cast<int>(arg0), reinterpret_cast<char*>(arg1),
                    static_cast<int>(arg2), static_cast<unsigned int>(arg3),
                    reinterpret_cast<struct statx*>(arg4));
            break;
#endif
        case SYS_lstat:
            *result = hook_lstat(
                    reinterpret_cast<char*>(arg0),
                    reinterpret_cast<struct stat*>(arg1));
            break;
        case SYS_fstat:
            *result = hook_fstat(
                    static_cast<int>(arg0),
                    reinterpret_cast<struct stat*>(arg1));
            break;
        case SYS_newfstatat:
            *result = hook_fstatat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    reinterpret_cast<struct stat*>(arg2),
                    static_cast<int>(arg3));
            break;
        case SYS_read:
            *result = hook_read(static_cast<unsigned int>(arg0),
                                            reinterpret_cast<void*>(arg1),
                                            static_cast<size_t>(arg2));
            break;

        case SYS_pread64:
            *result = hook_pread(static_cast<unsigned int>(arg0),
                                             reinterpret_cast<char*>(arg1),
                                             static_cast<size_t>(arg2),
                                             static_cast<loff_t>(arg3));
            break;
/*
        case SYS_readv:
            *result = hook_readv(
                    static_cast<unsigned long>(arg0),
                    reinterpret_cast<const struct iovec*>(arg1),
                    static_cast<unsigned long>(arg2));
            break;

        case SYS_preadv:
            *result = hook_preadv(
                    static_cast<unsigned long>(arg0),
                    reinterpret_cast<const struct iovec*>(arg1),
                    static_cast<unsigned long>(arg2),
                    static_cast<unsigned long>(arg3),
                    static_cast<unsigned long>(arg4));
            break;
*/
        case SYS_pwrite64:
            *result = hook_pwrite(
                    static_cast<unsigned int>(arg0),
                    reinterpret_cast<const char*>(arg1),
                    static_cast<size_t>(arg2), static_cast<loff_t>(arg3));
            break;
        case SYS_write:
            *result =
                    hook_write(static_cast<unsigned int>(arg0),
                                           reinterpret_cast<const char*>(arg1),
                                           static_cast<size_t>(arg2));
            break;
/*
        case SYS_writev:
            *result = hook_writev(
                    static_cast<unsigned long>(arg0),
                    reinterpret_cast<const struct iovec*>(arg1),
                    static_cast<unsigned long>(arg2));
            break;

        case SYS_pwritev:
            *result = hook_pwritev(
                    static_cast<unsigned long>(arg0),
                    reinterpret_cast<const struct iovec*>(arg1),
                    static_cast<unsigned long>(arg2),
                    static_cast<unsigned long>(arg3),
                    static_cast<unsigned long>(arg4));
            break;
*/
        case SYS_unlink:
            *result = hook_unlinkat(
                    AT_FDCWD, reinterpret_cast<const char*>(arg0), 0);
            break;

        case SYS_unlinkat:
            *result = hook_unlinkat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    static_cast<int>(arg2));
            break;

        case SYS_rmdir:
            *result = hook_unlinkat(
                    AT_FDCWD, reinterpret_cast<const char*>(arg0),
                    AT_REMOVEDIR);
            break;

        case SYS_symlink:
            *result = hook_symlinkat(
                    reinterpret_cast<const char*>(arg0), AT_FDCWD,
                    reinterpret_cast<const char*>(arg1));
            break;

        case SYS_symlinkat:
            *result = hook_symlinkat(
                    reinterpret_cast<const char*>(arg0), static_cast<int>(arg1),
                    reinterpret_cast<const char*>(arg2));
            break;

        case SYS_access:
            *result =
                    hook_access(reinterpret_cast<const char*>(arg0),
                                            static_cast<int>(arg1));
            break;

        case SYS_faccessat:
            *result = hook_faccessat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    static_cast<int>(arg2));
            break;
#ifdef SYS_faccessat2
        case SYS_faccessat2:
            *result = hook_faccessat2(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    static_cast<int>(arg2), static_cast<int>(arg3));
            break;
#endif
        case SYS_lseek:
            *result = hook_lseek(static_cast<unsigned int>(arg0),
                                             static_cast<off_t>(arg1),
                                             static_cast<unsigned int>(arg2));
            break;

        case SYS_truncate:
            *result = hook_truncate(
                    reinterpret_cast<const char*>(arg0),
                    static_cast<long>(arg1));
            break;

        case SYS_ftruncate:
            *result = hook_ftruncate(
                    static_cast<unsigned int>(arg0),
                    static_cast<unsigned long>(arg1));
            break;

        case SYS_dup:
            *result = hook_dup(static_cast<unsigned int>(arg0));
            break;

        case SYS_dup2:
            *result = hook_dup2(static_cast<unsigned int>(arg0),
                                            static_cast<unsigned int>(arg1));
            break;

        case SYS_dup3:
            *result = hook_dup3(static_cast<unsigned int>(arg0),
                                            static_cast<unsigned int>(arg1),
                                            static_cast<int>(arg2));
            break;

        case SYS_getdents:
            *result = hook_getdents(
                    static_cast<unsigned int>(arg0),
                    reinterpret_cast<struct linux_dirent*>(arg1),
                    static_cast<unsigned int>(arg2));
            break;

        case SYS_getdents64:
            *result = hook_getdents64(
                    static_cast<unsigned int>(arg0),
                    reinterpret_cast<struct linux_dirent64*>(arg1),
                    static_cast<unsigned int>(arg2));
            break;

        case SYS_mkdirat:
            *result = hook_mkdirat(
                    static_cast<unsigned int>(arg0),
                    reinterpret_cast<const char*>(arg1),
                    static_cast<mode_t>(arg2));
            break;

        case SYS_mkdir:
            *result = hook_mkdirat(
                    AT_FDCWD, reinterpret_cast<const char*>(arg0),
                    static_cast<mode_t>(arg1));
            break;

        case SYS_chmod:
            *result = hook_fchmodat(AT_FDCWD,
                                                reinterpret_cast<char*>(arg0),
                                                static_cast<mode_t>(arg1));
            break;

        case SYS_fchmod:
            *result = hook_fchmod(static_cast<unsigned int>(arg0),
                                              static_cast<mode_t>(arg1));
            break;

        case SYS_fchmodat:
            *result = hook_fchmodat(static_cast<unsigned int>(arg0),
                                                reinterpret_cast<char*>(arg1),
                                                static_cast<mode_t>(arg2));
            break;

        case SYS_chdir:
            *result =
                    hook_chdir(reinterpret_cast<const char*>(arg0));
            break;

        case SYS_fchdir:
            *result = hook_fchdir(static_cast<unsigned int>(arg0));
            break;

        case SYS_getcwd:
            *result = hook_getcwd(reinterpret_cast<char*>(arg0),
                                              static_cast<unsigned long>(arg1));
            break;
        case SYS_readlink:
            *result = hook_readlinkat(
                    AT_FDCWD, reinterpret_cast<const char*>(arg0),
                    reinterpret_cast<char*>(arg1), static_cast<int>(arg2));
            break;
        case SYS_readlinkat:
            *result = hook_readlinkat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    reinterpret_cast<char*>(arg2), static_cast<int>(arg3));
            break;

        case SYS_fcntl:
            *result = hook_fcntl(static_cast<unsigned int>(arg0),
                                             static_cast<unsigned int>(arg1),
                                             static_cast<unsigned long>(arg2));
            break;

        case SYS_rename:
            *result = hook_renameat(
                    AT_FDCWD, reinterpret_cast<const char*>(arg0), AT_FDCWD,
                    reinterpret_cast<const char*>(arg1), 0);
            break;

        case SYS_renameat:
            *result = hook_renameat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    static_cast<int>(arg2), reinterpret_cast<const char*>(arg3),
                    0);
            break;

        case SYS_renameat2:
            *result = hook_renameat(
                    static_cast<int>(arg0), reinterpret_cast<const char*>(arg1),
                    static_cast<int>(arg2), reinterpret_cast<const char*>(arg3),
                    static_cast<unsigned int>(arg4));
            break;

        case SYS_fstatfs:
            *result = hook_fstatfs(
                    static_cast<unsigned int>(arg0),
                    reinterpret_cast<struct statfs*>(arg1));
            break;

        case SYS_statfs:
            *result = hook_statfs(
                    reinterpret_cast<const char*>(arg0),
                    reinterpret_cast<struct statfs*>(arg1));
            break;

        case SYS_fsync:
            *result = hook_fsync(static_cast<unsigned int>(arg0));
            break;

        case SYS_getxattr:
            *result = hook_getxattr(
                    reinterpret_cast<const char*>(arg0),
                    reinterpret_cast<const char*>(arg1),
                    reinterpret_cast<void*>(arg2), static_cast<size_t>(arg4));
            break;

        default:
            return 1;
    }
    return 0;
}

static __attribute__((constructor)) void
init(void)
{
	// Set up the callback function
	intercept_hook_point = hook;
}