# Solutions for chapter 2

1. C: first summands is number of threads in previous blocks, second summand is position in current block
2. C: Same as above, but now with stride 2
3. D: Previous blocks processed twice as many elements as in 1, in this block threads first process elements from the beginning
4. C: 8192 is next multiple of 1024
5. D: need size of memory in bytes
6. D: Want to pass `A_d` by reference, which will be a pointer to a pointer
7. C: destination, then source, then byte count
8. C
9.
    - 128 (second generic argument)
    - `N.next_multiple_of(128)`
    - `ceil(N / 128.0)`
    - all
    - `N`

10. Declare one function with `__host__ __device__`, it will be available on both
