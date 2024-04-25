- 4/25: Framerate was real stuttery. I was wondering if this was something I had
    done wrong with rounding and f32's and misordered divides/multiplies but it
    was a problem with the framerate itself! It turns out I had been working on
    and pushing the project with a log level of debug, so we were wasting a ton of
    cycles on printing crap to the console.
- 4/24: Trying to get a timestamp so that I can track the elapsed time and in the
    browser was getting errors that Rust's time builtins aren't supported in WASM.
    This seems to be because wasm-unknown-unknown makes no assumptions about the
    existence of an OS, and Rust's time builtin relies on an OS. There are some
    ways around this including a crate called `chronos` which allows for binding
    to the JS time implementations, but going to explore the timestamp write
    options of the render pass.
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
