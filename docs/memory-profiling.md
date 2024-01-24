# Memory Profiling

Valgrind is a tool for profiling memory usage. It can be used to find memory leaks,
memory errors, and other memory-related issues. From the various tools we'll have a
look at memcheck, which gives a good overview of the memory usage of a program and
possible memory leaks.

## Installation

Valgrind is available in the official repositories of most Linux distributions. For
example, on Ubuntu, you can install it with:

```bash
sudo apt install valgrind
```

In order to use the latest version of Valgrind, you can download it from here:
[Valgrind current](https://valgrind.org/downloads/current.html)

In case you want to build from source you can follow this [documentation](https://valgrind.org/docs/manual/manual-core.html#manual-core.install)

> **NOTE** The version 3.19 contains some fixes for the memory profiling of Rust

## Scenario

In order to have a consistent feedback loop for Intent Brokering and valgrind, we'll use the
load test benchmarking available in Intent Brokering.

## Running memcheck

Memcheck is the default valgrind tool for memory profiling. The usage for Intent Brokering
together with the e2e tests is as follows:

In a dedicated terminal, run valgrind with Intent Brokering

```bash
valgrind ./target/debug/intent_brokering
```

In another terminal, run the e2e tests

```bash
cargo run -p kv-app &
cargo test --test "*e2e"
```

After the tests have finished, you can stop the valgrind process with `Ctrl+C`.

The output of valgrind is quite verbose. The most important part is the summary at
the end of the output. It looks like this:

```bash
==7386== HEAP SUMMARY:
==7386==     in use at exit: 268,010 bytes in 2,095 blocks
==7386==   total heap usage: 4,523 allocs, 2,428 frees, 630,827 bytes allocated
==7386==
==7386== LEAK SUMMARY:
==7386==    definitely lost: 0 bytes in 0 blocks
==7386==    indirectly lost: 0 bytes in 0 blocks
==7386==      possibly lost: 173,607 bytes in 1,871 blocks
==7386==    still reachable: 94,403 bytes in 224 blocks
==7386==         suppressed: 0 bytes in 0 blocks
==7386== Rerun with --leak-check=full to see details of leaked memory
==7386==
==7386== For lists of detected and suppressed errors, rerun with: -s
==7386== ERROR SUMMARY: 0 errors from 0 contexts (suppressed: 0 from 0)
```

## Running massif

The other tool to use is **massif**. It writes its output to a file, which can be
visualized with the `ms_print` tool. The usage for Intent Brokering together with the e2e
tests is as follows:

In a dedicated terminal, run massif with Intent Brokering

```bash
valgrind --tool=massif ./target/debug/intent_brokering
```

In another terminal, run the e2e tests

```bash
cargo run -p kv-app &
cargo test --test "*e2e"
```

After the tests have finished, you can stop the valgrind process with `Ctrl+C`.

The output of massif is a file called `massif.out.<pid>`. You can visualize it with
the `ms_print` tool:

```bash
ms_print massif.out.7386
```

The output of `ms_print` looks like this (just a part of it):

```bash
--------------------------------------------------------------------------------
Command:            ./target/debug/intent_brokering
Massif arguments:   (none)
ms_print arguments: massif.out.7704
--------------------------------------------------------------------------------


    KB
505.4^                                                                 #
     |                                                           ::@@::#:
     |                                                          :: @@: #::
     |                                                        :::: @@: #::
     |                                                        : :: @@: #:::
     |                                                    ::::: :: @@: #:::
     |                                                    ::: : :: @@: #::::
     |                                                   :::: : :: @@: #::::
     |                                                ::::::: : :: @@: #::::
     |                                            ::::::::::: : :: @@: #:::::@
     |                                         :::::: ::::::: : :: @@: #:::::@
     |                                      ::::::::: ::::::: : :: @@: #:::::@
     |                               :::::::::::::::: ::::::: : :: @@: #:::::@
     |                          :::::::::: :::::::::: ::::::: : :: @@: #:::::@
     |                      :::::: ::::::: :::::::::: ::::::: : :: @@: #:::::@
     |                    ::: :::: ::::::: :::::::::: ::::::: : :: @@: #:::::@
     |                 :::::: :::: ::::::: :::::::::: ::::::: : :: @@: #:::::@
     |           ::::::: :::: :::: ::::::: :::::::::: ::::::: : :: @@: #:::::@
     |           :    :: :::: :::: ::::::: :::::::::: ::::::: : :: @@: #:::::@
     |        ::::    :: :::: :::: ::::::: :::::::::: ::::::: : :: @@: #:::::@
   0 +----------------------------------------------------------------------->Mi
     0                                                                   18.91

Number of snapshots: 64
 Detailed snapshots: [2, 48, 50, 53 (peak), 63]

--------------------------------------------------------------------------------
  n        time(i)         total(B)   useful-heap(B) extra-heap(B)    stacks(B)
--------------------------------------------------------------------------------
  0              0                0                0             0            0
  1        970,208              488              472            16            0
  2      1,360,774            8,520            8,435            85            0
99.00% (8,435B) (heap allocation functions) malloc/new/new[], --alloc-fns, etc.
->97.82% (8,334B) 0xBF01DB: alloc::alloc::alloc (alloc.rs:89)
| ->97.82% (8,334B) 0xBF0266: alloc::alloc::Global::alloc_impl (alloc.rs:171)
|   ->97.82% (8,334B) 0xBF0F79: <alloc::alloc::Global as core::alloc::Allocator>::allocate (alloc.rs:231)
|     ->96.16% (8,193B) 0xBE1550: alloc::raw_vec::RawVec<T,A>::allocate_in (raw_vec.rs:185)
|     | ->96.16% (8,193B) 0xBE1B5C: alloc::raw_vec::RawVec<T,A>::with_capacity_in (raw_vec.rs:131)
|     |   ->96.15% (8,192B) 0xBE132E: alloc::raw_vec::RawVec<T>::with_capacity (raw_vec.rs:93)
|     |   | ->96.15% (8,192B) 0xBE89B3: alloc::boxed::Box<[T]>::new_uninit_slice (boxed.rs:657)
|     |   |   ->96.15% (8,192B) 0xBE06BA: std::io::buffered::bufreader::BufReader<R>::with_capacity (bufreader.rs:96)
|     |   |     ->96.15% (8,192B) 0xBE076C: std::io::buffered::bufreader::BufReader<R>::new (bufreader.rs:75)
|     |   |       ->96.15% (8,192B) 0xBEAE20: num_cpus::linux::MountInfo::load_cpu (linux.rs:223)
|     |   |       | ->96.15% (8,192B) 0xBEA321: num_cpus::linux::load_cgroups (linux.rs:147)
|     |   |       |   ->96.15% (8,192B) 0xBEA18D: num_cpus::linux::init_cgroups (linux.rs:129)
|     |   |       |     ->96.15% (8,192B) 0xBEF068: core::ops::function::FnOnce::call_once (function.rs:248)
|     |   |       |       ->96.15% (8,192B) 0xBEC1EA: std::sync::once::Once::call_once::{{closure}} (once.rs:276)
|     |   |       |         ->96.15% (8,192B) 0x230FF9: std::sync::once::Once::call_inner (once.rs:434)
|     |   |       |           ->96.15% (8,192B) 0xBEC178: std::sync::once::Once::call_once (once.rs:276)
|     |   |       |             ->96.15% (8,192B) 0xBEA0E7: num_cpus::linux::cgroups_num_cpus (linux.rs:114)
```

## Profiling with bytehound

[Bytehound](https://github.com/koute/bytehound) is a tool that can be used to
profile the memory usage of a program. It works by instrumenting the program
with a custom allocator that tracks the number of bytes allocated and freed.

One of the benefits of bytehound is that it comes out of the box with a visualisation
tool that can be used to dig into the collecting data. Further it also supports
a scripting language to filter and aggregate the data.

This is an alternative that is just mentioned for completeness.
