use proc_macro::TokenStream;
use std::path::PathBuf;

use glob::glob;
use quote::quote;

#[proc_macro]
pub fn list_blog_files(_: TokenStream) -> TokenStream {
    // // Generate list of literal string paths
    // //PathBuf::from("../blog").into_os_string().into_string().unwrap();
    // {
    //     // dbg!(PathBuf::from("blog").canonicalize());
    //     // dbg!(PathBuf::from("../klamer_dev/blog").canonicalize().unwrap().into_os_string().into_string().unwrap() + "/*.html");
    //     dbg!(env::current_dir().unwrap());
    //     dbg!(glob(&(PathBuf::from("./klamer_dev/blog").canonicalize().unwrap().into_os_string().into_string().unwrap() + "/*.html")).unwrap().collect::<Vec<_>>());
    // }
    let files = glob(&(PathBuf::from("./klamer_dev/blog").canonicalize().unwrap().into_os_string().into_string().unwrap() + "/*.html")).unwrap();
    let file_literals = files.map(|file| {
        let file = file.unwrap().canonicalize().unwrap().into_os_string().into_string().unwrap();
        quote!(#file)
    });

    // Run handlers if required
    quote!([#((#file_literals, include_str!(#file_literals))),*]).into()
}