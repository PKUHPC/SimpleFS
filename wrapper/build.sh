./clean.sh

cd ../sfs_client
sh build.sh
cd ../wrapper

./build_intercept.sh
cc intercept.cpp hook.cpp -lsyscall_intercept libsfs_client.a  -lpthread -lm -lstdc++ -lrdmacm -libverbs -fpic -shared -o libsfs_client.so
#export LD_LIBRARY_PATH=/home/dev/Desktop/SimpleFS/wrapper/. 
#export LD_PRELOAD=/home/dev/Desktop/SimpleFS/wrapper/libsfs_client.so LD_LIBRARY_PATH=/home/dev/Desktop/SimpleFS/wrapper/. 
#LD_LIBRARY_PATH=. LD_PRELOAD=libsfs_client.so ls
#LD_LIBRARY_PATH=. LD_PRELOAD=libsfs_client.so ./a.out