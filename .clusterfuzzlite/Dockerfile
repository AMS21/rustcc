FROM gcr.io/oss-fuzz-base/base-builder-rust:v1
RUN apt update -y && \
    apt upgrade -y && \
    apt install -y --no-install-recommends clang cmake git liblzma-dev libpthread-stubs0-dev libz-dev ninja-build zip
RUN git clone https://github.com/llvm/llvm-project.git /llvm-project --branch llvmorg-21.1.2 --depth 1 --single-branch --no-tags
RUN cmake -G "Ninja" \
        -S /llvm-project/llvm \
        -B /llvm-project/build \
        -DBUILD_SHARED_LIBS=OFF \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_CXX_COMPILER=clang++ \
        -DCMAKE_CXX_FLAGS="-dwarf4" \
        -DCMAKE_C_COMPILER=clang \
        -DCMAKE_INSTALL_PREFIX=/usr/local \
        -DCMAKE_SKIP_RPATH=ON \
        -DLLVM_BUILD_STATIC=ON \
        -DLLVM_BUILD_TOOLS=ON \
        -DLLVM_BUILD_UTILS=OFF \
        -DLLVM_ENABLE_ASSERTIONS=ON \
        -DLLVM_ENABLE_BINDINGS=OFF \
        -DLLVM_ENABLE_LIBCXX=ON \
        -DLLVM_ENABLE_PIC=OFF \
        -DLLVM_ENABLE_PROJECTS="" \
        -DLLVM_ENABLE_RUNTIMES="" \
        -DLLVM_ENABLE_TELEMETRY=OFF \
        -DLLVM_ENABLE_TERMINFO=OFF \
        -DLLVM_ENABLE_UNWIND_TABLES=OFF \
        -DLLVM_ENABLE_WARNINGS=OFF \
        -DLLVM_ENABLE_ZLIB=ON \
        -DLLVM_INCLUDE_BENCHMARKS=OFF \
        -DLLVM_INCLUDE_DOCS=OFF \
        -DLLVM_INCLUDE_EXAMPLES=OFF \
        -DLLVM_INCLUDE_RUNTIMES=OFF \
        -DLLVM_INCLUDE_TESTS=OFF \
        -DLLVM_INCLUDE_TOOLS=ON \
        -DLLVM_INCLUDE_UTILS=OFF \
        -DLLVM_OPTIMIZED_TABLEGEN=ON \
        -DLLVM_TARGETS_TO_BUILD="X86" \
        -DLLVM_USE_LINKER=lld \
        -DLLVM_USE_STATIC_ZSTD=ON
RUN cmake --build /llvm-project/build --target install-llvm-config-stripped --target install-llvm-libraries-stripped -j$(nproc)
RUN rm -rf /llvm-project
COPY . $SRC/rustcc
WORKDIR rustcc
COPY .clusterfuzzlite/build.sh $SRC/
