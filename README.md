# vtable_bench

This is a series of benchmarks designed to give some understanding of how
performance in Rust programs depending on whether the vtable pointer is stored
alongside the normal pointer (a "fat pointer") or next to the object itself
("inner vtable pointers").

You can perform a run with default values as follows:

```
$ cargo build --release && cargo run --release --bin vtable_bench
```


# Why this is worth measuring

Trait objects' vtables are passed around as fat pointers. If they're used
rarely, this seems unlikely to make much performance difference either way.
However, if we're expecting to have a reasonably large number of trait objects,
it's not obvious if fat pointers (2 machine words) are a better trade-off vs.
storing the vtable pointer in the object itself (making pointers 1 machine
word at the cost of an indirection to access the vtable).

These benchmarks are an attempt to start understanding the performance
trade-offs. They are highly simplistic, and inevitably only tell us about a
small handful of possible performance points. They might be better than
nothing, however.

Each benchmark creates one or more trait objects, puts them in a vector, and
then iterates over that vector calling a method:

* The `fat` benchmarks use fat pointers and the `innervtable` benchmarks store
  the vtable alongside the data itself (recreating fat pointers on an as-needed
  basis).

* The `multialias` benchmarks allocate a single box and multiply alias it;
  non-`multialias` benchmarks allocate multiple boxes without aliasing.

* The `no_read` benchmarks do not read from self (they return a fixed integer);
  `with_read` benchmarks do read from self. This is trying to understand whether
  bringing a trait object into cache by reading its vtable offsets the later
  read.

The runner is fairly simplistic, but runs each benchmark in its own process
(with 30 process executions and 100 in-process iterations), printing means and
99% confidence intervals (calculated from the standard deviation).


# Example results

`vtable_bench` takes (optionally) 3 arguments:

```
$ cargo run --release --bin vtable_bench -- -h
Usage: vtable_bench [-h] [<#reps> <#iters> <#vec size>]
```

From right to left (since it's easier to understand that way): the vector
being iterated over will contain `vec size` elements; a `for` loop will
iterate over that vector `iters` times; and the benchmark will be
executed from scratch `reps` times (each execution creating a fresh Unix
process).

Here are example runs (with irrelevant lines elided) with a gradually increasing
`vec size` and a decreasing number of `iters` to keep the runs to a manageable
time and make comparisons easier:

```
$ cargo build --release
$ cargo run --release --bin vtable_bench 30 10000 100000
bench_fat_multialias_no_read: 1.628 +/- 0.0019
bench_fat_multialias_with_read: 1.630 +/- 0.0030
bench_fat_no_read: 1.627 +/- 0.0003
bench_fat_with_read: 1.677 +/- 0.0045
bench_innervtable_multialias_no_read: 1.628 +/- 0.0011
bench_innervtable_multialias_with_read: 1.627 +/- 0.0002
bench_innervtable_no_read: 1.660 +/- 0.0032
bench_innervtable_with_read: 1.671 +/- 0.0023
$ cargo run --release --bin vtable_bench 30 1000 1000000
bench_fat_multialias_no_read: 1.709 +/- 0.0104
bench_fat_multialias_with_read: 1.709 +/- 0.0099
bench_fat_no_read: 1.708 +/- 0.0138
bench_fat_with_read: 2.152 +/- 0.0103
bench_innervtable_multialias_no_read: 1.641 +/- 0.0007
bench_innervtable_multialias_with_read: 1.644 +/- 0.0115
bench_innervtable_no_read: 2.111 +/- 0.0054
bench_innervtable_with_read: 2.128 +/- 0.0090
$ cargo run --release --bin vtable_bench 30 100 10000000
bench_fat_multialias_no_read: 1.700 +/- 0.0012
bench_fat_multialias_with_read: 1.707 +/- 0.0128
bench_fat_no_read: 1.694 +/- 0.0014
bench_fat_with_read: 2.182 +/- 0.0071
bench_innervtable_multialias_no_read: 1.666 +/- 0.0006
bench_innervtable_multialias_with_read: 1.681 +/- 0.0240
bench_innervtable_no_read: 2.129 +/- 0.0046
bench_innervtable_with_read: 2.152 +/- 0.0099
$ cargo run --release --bin vtable_bench 30 10 100000000
bench_fat_multialias_no_read: 1.708 +/- 0.0101
bench_fat_multialias_with_read: 1.702 +/- 0.0038
bench_fat_no_read: 1.705 +/- 0.0126
bench_fat_with_read: 2.184 +/- 0.0046
bench_innervtable_multialias_no_read: 1.664 +/- 0.0081
bench_innervtable_multialias_with_read: 1.666 +/- 0.0101
bench_innervtable_no_read: 2.146 +/- 0.0138
bench_innervtable_with_read: 2.169 +/- 0.0125
```
