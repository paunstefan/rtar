# rtar
Archiver/extractor for tar files written in Rust

## How to use

rtar at the moment supports only 3 options, extracting, archiving and viewing.

### View
```
rtar -v [tar file]
```

### Extract
```
rtar -x [tar file]
```

### Archive
```
rtar -c [new tar file] [files to archive]
```


It is an unfinished program, it doesn't support all file types supported by
the tar format (such as symlinks), only simple files and directories. I made
it to familiarize myself better with Rust.