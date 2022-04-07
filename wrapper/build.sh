cc intercept.cpp hook.cpp -lsyscall_intercept libsfs_client.a  -lpthread -lm -lstdc++ -fpic -shared -o libsfs_client.so
export LD_LIBRARY_PATH=/home/dev/Desktop/SimpleFS/wrapper/. 
export LD_PRELOAD=/home/dev/Desktop/SimpleFS/wrapper/libsfs_client.so