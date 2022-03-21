fn main(){
    let library_name = "syscall_intercept";
    println!("cargo:rustc-link-lib=static={}", library_name);
    println!("cargo:rustc-link-search=native={}", "src");
}