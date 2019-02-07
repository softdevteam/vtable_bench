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

* The `multiref` benchmarks allocate a single box and multiply alias it;
  non-`multiref` benchmarks allocate multiple boxes without aliasing.

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

Here are example runs with a gradually increasing `vec size` (and a decreasing
number of `iters` to keep the runs to a manageable time and make comparisons
easier).

```
$ cargo run --release --bin vtable_bench 30 10000 100000
bench_innervtable_multiref_no_read: 1.627 +/- 0.0002
bench_innervtable_multiref_with_read: 1.631 +/- 0.0100
bench_innervtable_no_read: 1.483 +/- 0.0040
bench_innervtable_with_read: 1.536 +/- 0.0043
bench_fat_multiref_no_read: 1.629 +/- 0.0049
bench_fat_multiref_with_read: 1.627 +/- 0.0007
bench_fat_no_read: 1.628 +/- 0.0015
bench_fat_with_read: 1.669 +/- 0.0051
$ cargo run --release --bin vtable_bench 30 1000 1000000
bench_innervtable_multiref_no_read: 1.649 +/- 0.0198
bench_innervtable_multiref_with_read: 1.644 +/- 0.0104
bench_innervtable_no_read: 2.063 +/- 0.0086
bench_innervtable_with_read: 2.098 +/- 0.0123
bench_fat_multiref_no_read: 1.704 +/- 0.0053
bench_fat_multiref_with_read: 1.700 +/- 0.0007
bench_fat_no_read: 1.707 +/- 0.0131
bench_fat_with_read: 2.188 +/- 0.0125
$ cargo run --release --bin vtable_bench 30 100 10000000
bench_innervtable_multiref_no_read: 1.666 +/- 0.0082
bench_innervtable_multiref_with_read: 1.666 +/- 0.0121
bench_innervtable_no_read: 2.077 +/- 0.0205
bench_innervtable_with_read: 2.100 +/- 0.0126
bench_fat_multiref_no_read: 1.699 +/- 0.0014
bench_fat_multiref_with_read: 1.702 +/- 0.0061
bench_fat_no_read: 1.698 +/- 0.0059
bench_fat_with_read: 2.184 +/- 0.0059
$ cargo run --release --bin vtable_bench 30 10 100000000
bench_innervtable_multiref_no_read: 1.663 +/- 0.0082
bench_innervtable_multiref_with_read: 1.672 +/- 0.0230
bench_innervtable_no_read: 2.076 +/- 0.0141
bench_innervtable_with_read: 2.112 +/- 0.0160
bench_fat_multiref_no_read: 1.709 +/- 0.0115
bench_fat_multiref_with_read: 1.701 +/- 0.0025
bench_fat_no_read: 1.702 +/- 0.0012
bench_fat_with_read: 2.196 +/- 0.0065
```
