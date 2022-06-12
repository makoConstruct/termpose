
#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate wood;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as PM2TS};
use quote::quote;
use syn::{Ident, Data::{Struct, Enum, Union}, Fields::{Named, Unnamed, Unit}};


// was going to make an impl that only does part of the translation, to let the user prepend tags. But I decided I'd prefer to just write translations by hand. Opening this up to the point where it's helping to automate, but also the user has full control, seems daunting. I might hate magic.
// #[proc_macro_derive(GeneratedWoodable)]
// pub fn generatedwoodable_derive(input: TokenStream) -> TokenStream {
// 	generated_Woodable(input, quote!{GeneratedWoodable});
// }

#[proc_macro_derive(Woodable)]
pub fn woodable_derive(input: TokenStream) -> TokenStream {
	generated_woodable(input, quote!{Woodable})
}
fn generated_woodable(input: TokenStream, being_impld: PM2TS) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let name = &ast.ident;
	
	let ret: TokenStream = match ast.data {
		Struct(ref s)=> {
			let woodify_branch_contents:Vec<PM2TS> = match s.fields {
				Named(ref n)=> {
					n.named.iter().map(|m:&syn::Field|{
						let mid = &m.ident.as_ref().unwrap();
						quote!{
							wood::branch!(
								stringify!(#mid),
								self.#mid.woodify(),
							)
						}
					}).collect()
				},
				Unnamed(ref n)=> {
					n.unnamed.iter().enumerate().map(|(i, _m)|{
						let id = syn::Index::from(i);
						quote!{ self.#id.woodify() }
					}).collect()
				},
				Unit=> {
					vec!(quote!{})
				}
			};
			
			let tit = woodify_branch_contents.into_iter();
			
			(quote! {
				impl wood::#being_impld for #name {
					fn woodify(&self)-> wood::Wood {
						wood::branch!(
							stringify!(#name),
							#(#tit),*
						)
					}
				}
			}).into()
		},
		
		
		Enum(ref e)=> {
			let variant_cases:Vec<PM2TS> = e.variants.iter().map(|m:&syn::Variant|{
				let variant_name = &m.ident;
				
				//left is just the name, right is the 
				let has_fields = |bindings: PM2TS, woodifications:PM2TS|{
					quote!{
						#name::#variant_name #bindings => {
							wood::branch!(
								stringify!(#variant_name),
								#woodifications
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
						let woodvec:Vec<PM2TS> = fs.named.iter().map(|m:&syn::Field|{
							let fid = m.ident.as_ref().unwrap();
							quote!{
								wood::branch!(
									stringify!(#fid),
									#fid.woodify(),
								)
							}
						}).collect();
						
						has_fields(
							quote!{ { #(#bindvec),* } },
							quote!{ #(#woodvec),* }
						)
					},
					Unnamed(ref fs)=>{
						let names:Vec<Ident> = (0..fs.unnamed.len()).map(|i|{
							Ident::new(format!("v{}", i).as_str(), Span::call_site())
						}).collect();
						let nam = names.iter().map(|id| quote!{ ref #id });
						let woodvec = names.iter().map(|s| quote!{ #s.woodify() });
						has_fields(
							quote!{ ( #(#nam),* ) },
							quote!{ #(#woodvec),* },
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
				impl wood::#being_impld for #name {
					fn woodify(&self)-> wood::Wood {
					 	match *self {
							#(#variant_cases),*
						}
					}
				}
			}).into()
		},
		Union(_)=> {
			panic!("derive(Woodable) cannot support untagged unions. It would have no way of knowing which variant to read from")
		}
	};
	
	ret
}



// #[proc_macro_attribute]
// pub fn wood_derive(attr: TokenStream, item: TokenStream) -> TokenStream {
// 	(quote!{}).into()
// }


#[proc_macro_derive(Dewoodable, attributes(wood))]
pub fn dewoodable_derive(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let name = &ast.ident;
	
	let ret = TokenStream::from( match ast.data {
		Struct(ref s)=> {
			
			let with_body = |method_body|-> PM2TS {
				//it seeks over the contents of the wood branch in such way where if the items are in order it will find each one immediately
				quote! {
					impl wood::Dewoodable for #name {
						fn dewoodify(v:&wood::Wood)-> Result<Self, Box<wood::WoodError>> {
							#method_body
						}
					}
				}
			};
			
			match s.fields {
				Named(ref n)=> {
					
					if n.named.len() == 0 {
						with_body(quote! { Self{} })
					}else{
						
						let mut var_parsing:Vec<PM2TS> = Vec::new();
						for m in n.named.iter() {
							// if m.attrs.iter().any(|a| a.path.segments.zip(["wood", "dont"].iter()).all(|i, s| i.ident == s)) {continue;}
							let var_ident = &m.ident.as_ref().unwrap();
							let var_type = &m.ty;
							var_parsing.push(quote!{
								#var_ident: {type T = #var_type; T::dewoodify(scanning.find(stringify!(#var_ident))?)?}
							});
						}
						
						let number_of_fields = var_parsing.len();
						
						with_body(quote!{
							let mut scanning = wood::wooder::FieldScanning::new(v);
							if scanning.li.len() != #number_of_fields {
								return Err(Box::new(wood::WoodError::new(v, format!("{} expected the wood to have {} elements, but it has {}", stringify!(#name), #number_of_fields, scanning.li.len()))));
							}
							
							Ok(Self{
								#(#var_parsing),*
							})
						})
					}
				},
				
				Unnamed(ref n)=> {
					
					let number_of_fields = n.unnamed.len();
					
					let each_field = n.unnamed.iter().enumerate().map(|(i, m)|{
						let ty = &m.ty;
						quote!{#ty::dewoodify(&li[#i])?}
					});
					
					with_body(quote!{
						let li = v.tail().as_slice();
						if li.len() == #number_of_fields {
							Ok(Self(#(#each_field),*))
						}else{
							Err(Box::new(wood::WoodError::new(v, format!("{} expected the wood to have {} fields, but it has {}", stringify!(#name), #number_of_fields, li.len()))))
						}
					})
				},
				Unit=> {
					with_body(quote!{
						Ok(#name)
					})
				}
			}
		},
		
		
		Enum(ref e)=> {
			
			let variant_cases:Vec<PM2TS> = e.variants.iter().map(|m:&syn::Variant|{
				let variant_name = &m.ident;
				
				let for_fields:PM2TS = match m.fields {
					Named(ref n)=> {
						let each_feild:Vec<PM2TS> = n.named.iter().map(|f:&syn::Field|{
							let id = &f.ident;
							let ty = &f.ty;
							quote!{ #id: #ty::dewoodify(scanning.find(stringify!(#id))?)? }
						}).collect();
						
						let number_of_fields = each_feild.len();
						
						quote!{
							let mut scanning = wooder::FieldScanning::new(v);
							if scanning.li.len() != #number_of_fields {
								return Err(Box::new(wood::WoodError::new(v, format!("variant {} expected {} elements, found {}", stringify!(#variant_name), #number_of_fields, scanning.li.len()))));
							}
							Ok(#name::#variant_name {
								#(#each_feild),*
							})
						}
					},
					Unnamed(ref n)=> {
						
						let each_feild = n.unnamed.iter().enumerate().map(|(i,m)|{
							let ty = &m.ty;
							quote!{ #ty::dewoodify(&li[#i])? }
						});
						
						let number_of_fields = each_feild.len();
						
						quote!{
							let li = v.tail().as_slice();
							
							if li.len() != #number_of_fields {
								return Err(Box::new(wood::WoodError::new(v, format!("variant {} expected {} elements, found {}", stringify!(#variant_name), #number_of_fields, li.len()))));
							}
							
							Ok(#name::#variant_name(
								#(#each_feild),*
							))
						}
					},
					Unit=> {
						quote!{ Ok(#name::#variant_name) }
					}
				};
				
				quote!{
					stringify!(#variant_name)=> {
						#for_fields
					}
				}
			}).collect();
			
			quote!{
				impl Dewoodable for #name {
					fn dewoodify(v:&wood::Wood)-> Result<Self, Box<wood::WoodError>> {
						match v.initial_str() {
							#(#variant_cases,)*
							erc => Err(Box::new(wood::WoodError::new(v, format!("expected a {}, but no variant of {} is called {}", stringify!(#name), stringify!(#name), erc)))),
						}
					}
				}
			}
		},
		
		
		Union(_)=> {
			panic!("derive(Dewoodable) doesn't yet support unions")
		}
	});
	
	ret
}


