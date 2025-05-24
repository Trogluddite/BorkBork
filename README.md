# BorkBork
 a basic chat app geared towards CLI-using LakeDogs

 ## Building cross-platform
 ### Linux from MacOS
install macos-cross-toolchains -- see: [Github -- homebrew-macos-cross-toolchains](https://github.com/messense/homebrew-macos-cross-toolchains)
```
brew tap messense/macos-cross-toolchains
brew install x86_64-unknown-linux-gnu  # dep. on architecture
brew install aarch64-unknown-linux-gnu # dep. on architecture

```
update .zshrc/.bashrc etc
 ```
# You probably only need the linker exported
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

# but per docs, may need to  set these flags as well:
export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar
```

