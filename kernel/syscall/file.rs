use arch::context::ContextFile;

use core::slice;

use fs::{ResourceSeek, Url};

use schemes::pipe::{PipeRead, PipeWrite};

use system::c_string_to_str;

use syscall::{Stat, SEEK_CUR, SEEK_END, SEEK_SET};

use system::error::{Error, Result, EBADF, EFAULT, EINVAL};

/** <!-- @MANSTART{sys_chdir} -->
NAME
    sys_chdir - change working directory

SYNOPSIS
    sys_chdir(path: *const u8) -> Result<usize>;

DESCRIPTION
    sys_chdir changes the current working directory of the calling process to the directory
    specified in path

RETURN VALUE
    On success, Ok(0) is returned. On error, Err(err) is returned where err is one of the following
    errors

ERRORS
    EACCESS TODO
        Access permissions denied to one of the path components

    EFAULT TODO
        path points outside the accessible address space of the process

    EIO TODO
        An I/O error occured

    ENOENT TODO
        path references a directory that does not exist

    ENOMEM TODO
        Insufficient kernel memory was available

    ENOTDIR TODO
        A component of path is not a directory

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_chdir(path: *const u8) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    unsafe {
        *current.cwd.get() = current.canonicalize(c_string_to_str(path));
    }
    Ok(0)
}

/** <!-- @MANSTART{sys_close} -->
NAME
    sys_close - close a file descriptor

SYNOPSIS
    sys_close(fd: usize) -> Result<usize>;

DESCRIPTION
    sys_close closes a file descriptor, so that it no longer refers to any file and may be reused.

RETURN VALUE
    On success, Ok(0) is returned. On error, Err(err) is returned where err is one of the following
    errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    EIO TODO
        An I/O error occured

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_close(fd: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());

    //debugln!("{}: {}: close {}", current.pid, current.name, fd);

    for i in 0..unsafe { (*current.files.get()).len() } {
        let mut remove = false;
        if let Some(file) = unsafe { (*current.files.get()).get(i) } {
            if file.fd == fd {
                remove = true;
            }
        }

        if remove {
            if i < unsafe { (*current.files.get()).len() } {
                drop(unsafe { (*current.files.get()).remove(i) });

                return Ok(0);
            }
        }
    }

    Err(Error::new(EBADF))
}

/** <!-- @MANSTART{sys_dup} -->
NAME
    sys_dup - duplicate a file descriptor

SYNOPSIS
    sys_dup(fd: usize) -> Result<usize>;

DESCRIPTION
    sys_dup creates a copy of fd, using the lowest unused descriptor for the new descriptor

RETURN VALUE
    On success, Ok(new_fd) is returned, where new_fd is the new file descriptor. On error, Err(err)
    is returned where err is one of the following errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_dup(fd: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let resource = try!(current.get_file(fd));
    let new_resource = try!(resource.dup());
    let new_fd = current.next_fd();

    //debugln!("{}: {}: dup {} as {}", current.pid, current.name, fd, new_fd);

    unsafe {
        (*current.files.get()).push(ContextFile {
            fd: new_fd,
            resource: new_resource,
        });
    }
    Ok(new_fd)
}

pub fn do_sys_fpath(fd: usize, buf: *mut u8, count: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let resource = try!(current.get_file(fd));
    resource.path(unsafe { slice::from_raw_parts_mut(buf, count) })
}

pub fn do_sys_fstat(fd: usize, stat: *mut Stat) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let resource = try!(current.get_file(fd));
    if stat as usize > 0 {
        resource.stat(unsafe { &mut *stat })
    } else {
        Err(Error::new(EFAULT))
    }
}

/** <!-- @MANSTART{sys_fsync} -->
NAME
    sys_fsync - synchronize a file's in-core state with storage device

SYNOPSIS
    sys_fsync(fd: usize) -> Result<usize>;

DESCRIPTION
    sys_fsync transfers all modified in-core data of the file refered to by the file descriptor fd
    to the underlying device

RETURN VALUE
    On success, Ok(0) is returned. On error, Err(err) is returned where err is one of the following
    errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    EIO TODO
        An I/O error occured

    EINVAL TODO
        fd does not support synchronization

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_fsync(fd: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.sync().and(Ok(0))
}

/** <!-- @MANSTART{sys_ftruncate} -->
NAME
    sys_ftruncate - truncate a file to a specified length

SYNOPSIS
    sys_ftruncate(fd: usize, length: usize) -> Result<usize>;

DESCRIPTION
    sys_ftruncate causes the file referenced by fd to be truncated to a size of precisely length
    bytes

RETURN VALUE
    On success, Ok(0) is returned. On error, Err(err) is returned where err is one of the following
    errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    EIO TODO
        An I/O error occured

    EINVAL TODO
        fd does not support truncation

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_ftruncate(fd: usize, length: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.truncate(length).and(Ok(0))
}

//TODO: Link

/** <!-- @MANSTART{sys_lseek} -->
NAME
    sys_lseek - reposition read/write file offset

SYNOPSIS
    sys_lseek(fd: usize, offset: isize, whence: usize) -> Result<usize>;

DESCRIPTION
    sys_lseek repositions the offset of the file referenced by fd to the offset according to whence

    SEEK_SET: 0
        The offset is set to offset bytes

    SEEK_CUR: 1
        The offset is set to its current location plus offset bytes

    SEEK_END: 2
        The offset is set to the size of the file plus offset bytes

RETURN VALUE
    On success, Ok(new_offset) is returned, where new_offset is the resulting offset location. On
    error, Err(err) is returned where err is one of the following errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    EINVAL
        whence or the offset is not valid

    ESPIPE
        fd does not support seeking

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_lseek(fd: usize, offset: isize, whence: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    match whence {
        SEEK_SET => resource.seek(ResourceSeek::Start(offset as usize)),
        SEEK_CUR => resource.seek(ResourceSeek::Current(offset)),
        SEEK_END => resource.seek(ResourceSeek::End(offset)),
        _ => Err(Error::new(EINVAL)),
    }
}

/** <!-- @MANSTART{sys_mkdir} -->
NAME
    sys_mkdir - create a directory

SYNOPSIS
    sys_mkdir(path: *const u8, flags: usize) -> Result<usize>;

DESCRIPTION
    sys_mkdir attempts to create a directory named path

RETURN VALUE
    On success, Ok(0) is returned. On error, Err(err) is returned where err is one of the following
    errors

ERRORS
    EACCES
        This process does not have write permissions to the parent directory or search permissions
        to other components in path

    EEXIST
        path already exists

    EFAULT
        path points outside of the accessible address space of the process

    ENOENT
        A directory component in path does not exist

    EPERM
        The filesystem containing path does not support the creation of directories

    EROFS
        The filesystem containing path is read-only

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_mkdir(path: *const u8, flags: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path_string = current.canonicalize(c_string_to_str(path));
    ::env().mkdir(try!(Url::from_str(&path_string)), flags).and(Ok(0))
}

/** <!-- @MANSTART{sys_open} -->
NAME
    sys_open - open and possibly create a file

SYNOPSIS
    sys_open(path: *const u8, flags: usize) -> Result<usize>;

DESCRIPTION
    sys_open returns a file descriptor referencing path, creating path if O_CREAT is provided

    TODO: Open is very complicated, and has a lot of flags

RETURN VALUE
    On success, Ok(fd) is returned, where fd is a file descriptor referencing path. On error,
    Err(err) is returned where err is one of the following errors

ERRORS
    EACCES
        The requested access to the file is not allowed, or search permissions are denied for one
        of the components of path, or the file did not exist and write access to the parent
        directory is not allowed

    EEXIST
        path already exists

    EFAULT
        path points outside of the accessible address space of the process

    EISDIR
        path refers to a directory and O_DIRECTORY was not provided

    ENOENT
        A directory component in path does not exist

    ENOMEM
        insufficient kernel memory was available

    ENOSPC
        There was insufficient space to create path

    ENOTDIR
        path does not refer to a directory and O_DIRECTORY was passed

    EPERM
        The filesystem containing path does not support the creation of files

    EROFS
        The filesystem containing path is read-only and write access was requested

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_open(path_c: *const u8, flags: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path = current.canonicalize(c_string_to_str(path_c));
    //debugln!("{}: {}: open {}", current.pid, current.name, path);
    let url = try!(Url::from_str(&path));
    let resource = try!(::env().open(url, flags));
    let fd = current.next_fd();
    unsafe {
        (*current.files.get()).push(ContextFile {
            fd: fd,
            resource: resource,
        });
    }
    Ok(fd)
}

pub fn do_sys_pipe2(fds: *mut usize, _flags: usize) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    if fds as usize > 0 {
        let read = box PipeRead::new();
        let write = box PipeWrite::new(&read);

        unsafe {
            *fds.offset(0) = current.next_fd();
            (*current.files.get()).push(ContextFile {
                fd: *fds.offset(0),
                resource: read,
            });

            *fds.offset(1) = current.next_fd();
            (*current.files.get()).push(ContextFile {
                fd: *fds.offset(1),
                resource: write,
            });
        }

        Ok(0)
    } else {
        Err(Error::new(EFAULT))
    }
}

/** <!-- @MANSTART{sys_read} -->
NAME
    sys_read - read from a file descriptor

SYNOPSIS
    sys_read(fd: usize, buf: *mut u8, count: usize) -> Result<usize>;

DESCRIPTION
    sys_read attempts to read up to count bytes from file descriptor fd into the buffer starting at
    buf

RETURN VALUE
    On success, Ok(count) is returned, where count is the number of bytes read into buf. On error,
    Err(err) is returned where err is one of the following errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    EFAULT
        buf is outside of the accessible address space of the process

    EINVAL
        fd refers to a ifle that does not support reading

    EIO
        I/O error

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_read(fd: usize, buf: *mut u8, count: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.read(unsafe { slice::from_raw_parts_mut(buf, count) })
}

pub fn do_sys_rmdir(path: *const u8) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path_string = current.canonicalize(c_string_to_str(path));
    ::env().rmdir(try!(Url::from_str(&path_string))).and(Ok(0))
}

pub fn do_sys_stat(path: *const u8, stat: *mut Stat) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path = current.canonicalize(c_string_to_str(path));
    let url = try!(Url::from_str(&path));
    if stat as usize > 0 {
        ::env().stat(url, unsafe { &mut *stat }).and(Ok(0))
    } else {
        Err(Error::new(EFAULT))
    }
}

pub fn do_sys_unlink(path: *const u8) -> Result<usize> {
    let contexts = ::env().contexts.lock();
    let current = try!(contexts.current());
    let path_string = current.canonicalize(c_string_to_str(path));
    ::env().unlink(try!(Url::from_str(&path_string))).and(Ok(0))
}

/** <!-- @MANSTART{sys_write} -->
NAME
    sys_write - read from a file descriptor

SYNOPSIS
    sys_write(fd: usize, buf: *mut u8, count: usize) -> Result<usize>;

DESCRIPTION
    sys_write attempts to read up to count bytes from file descriptor fd into the buffer starting at
    buf

RETURN VALUE
    On success, Ok(count) is returned, where count is the number of bytes read into buf. On error,
    Err(err) is returned where err is one of the following errors

ERRORS
    EBADF
        fd is not a valid open file decriptor

    EFAULT
        buf is outside of the accessible address space of the process

    EINVAL
        fd refers to a ifle that does not support writing

    EIO
        I/O error

    ENOSPC
        The filesystem containing fd has no more space

    EPIPE
        fd is connected to a pipe or socket whose reading end is closed

    ESRCH
        Currently not running in a process context (rare, would only happen during kernel init)
<!-- @MANEND --> */
pub fn do_sys_write(fd: usize, buf: *const u8, count: usize) -> Result<usize> {
    let mut contexts = ::env().contexts.lock();
    let mut current = try!(contexts.current_mut());
    let mut resource = try!(current.get_file_mut(fd));
    resource.write(unsafe { slice::from_raw_parts(buf, count) })
}
