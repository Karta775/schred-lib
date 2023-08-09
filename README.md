# schred-lib

schred-lib is a Rust crate for securely erasing files and directories, similarly to GNU shred.

## Features

The following features are available as options:

* Customisable number of random/zero passes.
* Recursively erase sub-directories & files.
* Deallocation (rm) files once erase is complete.
