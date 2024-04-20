- 4/19: Drawing the nested squares led to triangles and strange behavior. This was
    because the index buffer was set up to consume elements of size u16, and in
    refactoring the index buffer to generate from the list I had inadvertently
    begun using the default int type of i32, leading to some sort of read/write
    byte misalignment.
- 4/19: My IDE kept complaining about the `use winit::platform::web` import. This
    is a submodule that the `winit` package only exposes on WASM builds. This is an
    issue because on non-WASM builds, the `...::web` submodule doesn't exist and
    will cause compilation issues. This was caused by me replacing
    `cfg_if::cfg_if!` pre-processor-style macros with simple `if cfg!(...)` macros,
    without realizing how `cfg_if` worked. Since `cfg_if` works as a precompilation
    step the output binary doesn't have any need for `...::web`, but with `if cfg!`
    both branches are compiled into the binary and the `...::web` import must be
    available on all target architectures.
