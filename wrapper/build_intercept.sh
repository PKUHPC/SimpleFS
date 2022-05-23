rm -r build
mkdir build
cd build

cmake ../syscall_intercept -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_BUILD_TYPE=Release -DCMAKE_C_COMPILER=clang
make

sudo make install

cp libsyscall_intercept.a ..
cp libsyscall_intercept.so ..
cp libsyscall_intercept.so.0 ..
cp libsyscall_intercept.so.0.1.0 ..