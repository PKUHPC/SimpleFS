cc intercept.cpp hook.cpp -lsyscall_intercept libsfs_client.a  -lpthread -lm -lstdc++ -fpic -shared -o libsfs_client.so
LD_LIBRARY_PATH=/home/dev/Desktop/SimpleFS/wrapper/. LD_PRELOAD=/home/dev/Desktop/SimpleFS/wrapper/libsfs_client.so ls
LD_LIBRARY_PATH=. LD_PRELOAD=libsfs_client.so ls