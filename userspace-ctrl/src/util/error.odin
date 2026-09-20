package util

Error :: struct {
    code:    Errno,
    message: string,
}

ERR_OK             :: Errno(0)
ERR_NO_MEMORY      :: Errno(1)
ERR_INVALID_ARG    :: Errno(2)
ERR_NOT_FOUND      :: Errno(3)
ERR_PERMISSION     :: Errno(4)
ERR_BUSY           :: Errno(5)
ERR_TIMEOUT        :: Errno(6)
ERR_DEADLOCK       :: Errno(7)
ERR_BAD_HANDLE     :: Errno(8)
ERR_BAD_SYSCALL    :: Errno(9)
ERR_NOT_IMPLEMENTED:: Errno(10)

make_error :: proc(code: Errno, msg: string) -> Error {
    return Error{code = code, message = msg}
}