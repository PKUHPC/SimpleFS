pub fn get_var(name: String, default: String) -> String{
    if let Ok(var) = std::env::var(name){
        var
    }
    else{
        default
    }
}