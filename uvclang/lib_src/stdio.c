#include <stdio.h>

static FILE __uvclang_files[FOPEN_MAX];

// Open `path` per the C mode string. The first character selects the primary
// mode (r/w/a); a '+' anywhere adds the opposite access; 'b' is ignored (all
// UVM I/O is binary). Returns a stream, or NULL on a bad mode, a rejected/absent
// path, or an exhausted stream pool.
FILE *fopen(const char *path, const char *mode)
{
    uint64_t flags;
    int append = 0;
    switch (mode[0]) {
        case 'r': flags = OPEN_READ; break;
        case 'w': flags = OPEN_WRITE | OPEN_CREATE | OPEN_TRUNC; break;
        case 'a': flags = OPEN_WRITE | OPEN_CREATE; append = 1; break;
        default:  return NULL;
    }
    for (int i = 1; mode[i] != '\0'; i++)
        if (mode[i] == '+')
            flags |= OPEN_READ | OPEN_WRITE;

    uint64_t handle = file_open(path, flags);
    if (handle == 0)
        return NULL;

    // Claim a free slot (handle == 0). Fail closed if the pool is full.
    FILE *fp = NULL;
    for (int i = 0; i < FOPEN_MAX; i++) {
        if (__uvclang_files[i].__handle == 0) {
            fp = &__uvclang_files[i];
            break;
        }
    }
    if (fp == NULL) {
        file_close(handle);
        return NULL;
    }

    fp->__handle = handle;
    fp->__eof = 0;
    fp->__error = 0;

    // Append mode starts positioned at end of file.
    if (append)
        file_seek(handle, file_size(handle));

    return fp;
}

// Close `stream` and return its pool slot. Returns 0 on success, EOF if the
// stream is NULL.
int fclose(FILE *stream)
{
    if (stream == NULL)
        return EOF;
    file_close(stream->__handle);
    stream->__handle = 0;   // release the slot back to the pool
    return 0;
}