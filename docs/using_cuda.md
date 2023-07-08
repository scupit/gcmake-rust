# Using CUDA

[CUDA](https://developer.nvidia.com/cuda-toolkit) is NVIDIA's toolkit for
"creating high performance GPU-accelerated applications".

See the [GCMake CUDA example project](https://github.com/scupit/gcmake-test-project/tree/develop/cuda-example)
for an example of how CUDA should be set up with GCMake.

## Enabling CUDA

When building a project that supports CUDA, *make sure you have a working CUDA installation on your system*,
then set `GCMAKE_ENABLE_CUDA` to `ON` or some other truthy value at CMake configure time. For example:

``` sh
# Configure with CUDA support
cmake -S . -B build -DGCMAKE_ENABLE_CUDA=ON
# Build the project
cmake --build build -j10
```
