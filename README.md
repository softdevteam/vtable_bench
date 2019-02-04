# vtable_bench

This is a series of benchmarks designed to give some understanding of how
performance in Rust programs depending on whether the vtable pointer is stored
alongside the normal pointer (a "fat pointer") or next to the object itself.

Run as follows:

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

* The `normal` benchmarks use fat pointers and the `alongside` benchmarks store
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
