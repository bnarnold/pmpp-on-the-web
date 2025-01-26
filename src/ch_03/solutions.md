# Solutions for chapter 3

1.
    - see code: `mmul_by_row`
    - see code: `mmul_by_col`
    - Transposition converts both kernels into each other, so they are exchangeable.
      The loop accessing global memory sequentially stops the SM from interleaving computations and loads/writes, so they will be slower than using one thread per output element.
      In general, shorter loops in the shader should run faster, so computing one row at a time should be faster if the output has much less rows than columns.
2. see code
3.
    - `16 * 32 = 512`
    - `N.next_multiple_of(16) * M.next_multiple_of(32) = 160 * 320 = 51_200`
    - `10 * 10 = 100`
    - `M * N = 45_000`
4. (assuming zero-based indexing)
    - `400 * 20 + 10 = 8_010`
    - `500 * 10 + 20 = 5_020`
5. `(500 * 4 + 20) * 400 + 10 = 808_010`
