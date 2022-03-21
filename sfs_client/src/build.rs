fn main(){
    //let library_name = "syscall_intercept";
    //println!("cargo:rustc-link-lib=static={}", library_name);
    println!("cargo:rustc-link-search=native={}", "src");
    println!("cargo:rustc-link-arg={}", "-fpic");
    println!("cargo:rustc-link-arg={}", "-shared");
    println!("cargo:rustc-link-arg={}", "-lsyscall_intercept");
}