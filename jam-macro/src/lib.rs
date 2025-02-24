extern crate proc_macro;
use proc_macro::{Ident, TokenStream, TokenTree};

use std::iter::Peekable;

fn next_group(source: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Option<proc_macro::Group> {
    if let Some(TokenTree::Group(_)) = source.peek() {
        let group = match source.next().unwrap() {
            TokenTree::Group(group) => group,
            _ => unreachable!("just checked with peek()!"),
        };
        Some(group)
    } else {
        None
    }
}

// A lot of code borrowed from macroquad's main attribute
#[proc_macro_attribute]
pub fn method_system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut modified = TokenStream::new();
    let mut source = item.into_iter().peekable();

    while let Some(TokenTree::Punct(punct)) = source.peek() {
        assert_eq!(format!("{}", punct), "#");

        let _ = source.next().unwrap();

        let group = next_group(&mut source);
        let mut group = group.unwrap().stream().into_iter().peekable();
        let attribute_name = format!("{}", group.next().unwrap());

        if attribute_name != "method_system" {
            panic!("Unexpected attribute {attribute_name:?}");
        }
    }

    let is_pub = if let TokenTree::Ident(ident) = source.peek().unwrap() {
        if ident.to_string() == "pub" {
            // skip 'pub'
            let _ = source.next().unwrap();
            true
        } else {
            false
        }
    } else {
        false
    };

    if let TokenTree::Ident(ident) = source.next().unwrap() {
        assert_eq!(format!("{}", ident), "fn");

        modified.extend(std::iter::once(TokenTree::Ident(ident)));
    } else {
        panic!("[macroquad::main] is allowed only for functions");
    }

    let old_ident = if let TokenTree::Ident(ident) = source.next().unwrap() {
        modified.extend(std::iter::once(TokenTree::Ident(Ident::new(
            &format!("{ident}_impl"),
            ident.span(),
        ))));

        ident
    } else {
        panic!("[macroquad::main] expecting main function");
    };


    let args = next_group(&mut source).unwrap();
    let old_args: TokenStream = format!(
        "{args}", args=args,
    )
    .parse()
    .unwrap();
    modified.extend(std::iter::once(old_args));
    modified.extend(source);

    let mut args = args.stream().into_iter().peekable();
    if let TokenTree::Punct(punct) = args.next().unwrap() {
        assert_eq!(punct.as_char(), '&');
    } else {
        panic!("Method systems must borrow");
    }

    let is_mut = if let TokenTree::Ident(ident) = args.peek().unwrap() {
        if ident.to_string() == "mut" {
            // skip 'mut'
            let _ = args.next().unwrap();
            true
        } else {
            false
        }
    } else {
        false
    };

    if let TokenTree::Ident(ident) = args.next().unwrap() {
        assert_eq!(ident.to_string(), "self");
    } else {
        panic!("self");
    }

    let args_stream = TokenStream::from_iter(args.clone());

    if let Some(TokenTree::Punct(_)) = args.peek() {
        args.next();
    }

    let mut args_names = Vec::new();
    while let Some(TokenTree::Ident(mut ident)) = args.next() {
        if ident.to_string() == "mut" {
            if let TokenTree::Ident(x) = args.next().unwrap() {
                ident = x;
            }
        }
        args_names.push(ident.to_string());

        args.next(); // Colon
        /* The type. Works, because all arguments are T<S> */
        args.next(); args.next(); args.next(); args.next();
        args.next(); // Comma
    }

    let mut prelude: TokenStream = format!(
        "
    {pub_main} fn {old_ident}(mut this: {view}<Self>{args_stream}) {{
        Self::{old_ident}_impl(
            {deref},
            {args}
        )
    }}
    ",
        deref = if is_mut { "&mut *this" } else { "&*this" },
        view = if is_mut { "shipyard::UniqueViewMut" } else { "shipyard::UniqueView" },
        pub_main = if is_pub { "pub" } else { "" },
        args = args_names.join(", ")
    )
    .parse()
    .unwrap();
    prelude.extend(modified);

    prelude
}