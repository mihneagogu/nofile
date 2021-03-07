# nofile
Ever get tired of writing those damned CMake and Makefile to build your project? Nofile automates the process almost entirely (the only thing you need to add extra are the library flags), building a Makefile for your C project starting from your files which are your entry points (which contain main() ). Nofile is written in rust and it is a multi-threaded program, so it is suited for large projects as well

--- USAGE ---
To build project use either "cargo build" or "cargo build --release" (need rust installed for that)
After that, either do "cargo run" or "cargo run --release" followed by the entrypoints for your executables.
What this means: say you want to build 3 executables, and each of them have an entry point called exe1.c exe2.c and exe3.c

Run "cargo run --release <path to exe1.c> <path to exe2.c> <path to exe3.c>" and this will build a Makefile which will compile all 3 executables.
!!! IF YOU DO USE EXTERNAL LIBRARIES !!! You will need to manually add the library flags to the Makefile, but it shouldn't be much of a bother.

Alternatively, you can take the binary executables of nofile from target/debug or target/release and place them in the folder with your entrypoints and run it like this:
"./nofile <path-to-executable-entrypoint1> <path-to-executable-entrypoint2> ..."


The program would benefit from thread pooling. However, the concurrency in this project was added just to experiemnt with Rust's primitives and see how easy they are to use compared to the standard C/C++ pthreads, mutexes and shared_ptrs and Java's concurrency primitives. The conclusion is that Rust's system is easier to use and saves you a lot from pitfalls.
