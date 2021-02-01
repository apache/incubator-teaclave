
## Compile for Non-SGX
```
mkdir build
cd build
cmake .. -DNON_SGX_PROTECTED_FS=ON
make
```

## Compile for iOS

```bash
# select xcode build env
sudo xcode-select -switch /Applications/Xcode.app/Contents/Developer
```

```bash
mkdir build
cd build
cmake .. -G Xcode -DCMAKE_TOOLCHAIN_FILE=../ios.toolchain.cmake -DPLATFORM=OS64 -DNON_SGX_PROTECTED_FS=ON
cmake --build . --config Release
```
