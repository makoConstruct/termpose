
extern crate proc_macro;
extern crate proc_macro2;
extern crate termpose;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as PM2TS};
use quote::quote;
use syn::{Ident, Fields::{Named, Unnamed, Unit}};

#[proc_macro_derive(Termable)]
pub fn termable_derive(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let name = &ast.ident;
	
	let ret: TokenStream = match ast.data {
		syn::Data::Struct(ref s)=> {
			let termify_list_contents:Vec<PM2TS> = match s.fields {
				Named(ref n)=> {
					n.named.iter().map(|m:&syn::Field|{
						let mid = &m.ident.as_ref().unwrap();
						quote!{
							termpose::list!(
								stringify!(#mid),
								self.#mid.termify(),
							)
						}
					}).collect()
				},
				Unnamed(ref n)=> {
					n.unnamed.iter().enumerate().map(|(_m, i)|{
						quote!{ self.#i.termify(), }
					}).collect()
				},
				Unit=> {
					vec!(quote!{})
				}
			};
			
			let tit = termify_list_contents.into_iter();
			
			(quote! {
				impl termpose::Termable for #name {
					fn termify(&self)-> termpose::Term {
						// termpose::list!("yes")
						termpose::list!(
							stringify!(#name),
							#(#tit),*
						)
					}
				}
			}).into()
		},
		
		
		syn::Data::Enum(ref e)=> {
			let variant_cases:Vec<PM2TS> = e.variants.iter().map(|m:&syn::Variant|{
				let variant_name = &m.ident;
				
				//left is just the name, right is the 
				let has_fields = |bindings: PM2TS, termifications:PM2TS|{
					quote!{
						#name::#variant_name #bindings => {
							termpose::list!(
								stringify!(#variant_name),
								#termifications
							)
						}
					}
				};
				
				
				match m.fields {
					Named(ref fs)=>{
						let bindvec:Vec<PM2TS> = fs.named.iter().map(|m:&syn::Field|{
							let id = &m.ident;
							quote!{ ref #id }
						}).collect();
						let termvec:Vec<PM2TS> = fs.named.iter().map(|m:&syn::Field|{
							let fid = m.ident.as_ref().unwrap();
							quote!{
								termpose::list!(
									stringify!(#fid),
									#fid.termify(),
								)
							}
						}).collect();
						
						has_fields(
							quote!{ { #(#bindvec),* } },
							quote!{ #(#termvec),* }
						)
					},
					Unnamed(ref fs)=>{
						let names:Vec<Ident> = (0..fs.unnamed.len()).map(|i|{
							Ident::new(format!("v{}", i).as_str(), Span::call_site())
						}).collect();
						let nam = names.iter().map(|id| quote!{ ref #id });
						let termvec = names.iter().map(|s| quote!{ #s.termify() });
						has_fields(
							quote!{ ( #(#nam),* ) },
							quote!{ #(#termvec),* },
						)
					},
					Unit =>{
						quote! {
							#name::#variant_name => stringify!(#variant_name).into()
						}
					}
				}
			}).collect();
			
			(quote! {
				impl termpose::Termable for #name {
					fn termify(&self)-> termpose::Term {
					 	match *self {
							#(#variant_cases),*
						}
					}
				}
			}).into()
		}
		_=> {
			panic!("termable_derive only supports structs right now");
		}
	};
	
	ret
}


// pub trait Termable {
// 	fn termify(&self)-> Term;
// }
// pub trait Untermable {
// 	fn untermify(&Term)-> Result<Self, UntermifyError> where Self:Sized;
// }


// #[cfg(test)]
// mod tests {
// 	#[test]
// 	fn it_works() {
// 		assert_eq!(2 + 2, 4);
// 	}
// }
