use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn bevy_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    assert_eq!(
        input.sig.ident, "main",
        "`bevy_main` can only be used on a function called 'main'."
    );

    TokenStream::from(quote! {
        #[no_mangle]
        #[cfg(target_os = "android")]
        fn android_main(android_app: bevy::window::android_activity::AndroidApp) {
            let _ = bevy::window::ANDROID_APP.set(android_app);
            main();
        }

        #[no_mangle]
        #[cfg(target_os = "ios")]
        extern "C" fn main_rs() {
            main();
        }

        #[allow(unused)]
        #input
    })
}
