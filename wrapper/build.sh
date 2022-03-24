cc intercept.cpp hook.cpp -lsyscall_intercept libsfs_client.a  -lpthread -lm -lstdc++ -fpic -shared -o libsfs_client.so
LD_LIBRARY_PATH=. LD_PRELOAD=libsfs_client.so ls