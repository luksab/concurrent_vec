# concurrent_vec

This is my first (second) adventure into fast concurrency, specifically in rust.

## "Benchmarks"
You need to choose the buffer size for your application. 
Here is a plot to show different numbers of cores and buffer sizes from 1 to 16k compared to one std::vec per thread.

![plots](buf_size.svg)