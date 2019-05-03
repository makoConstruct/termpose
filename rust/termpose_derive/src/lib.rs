
#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate termpose;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as PM2TS};
use quote::quote;
use syn::{Ident, Data::{Struct, Enum, Union}, Fields::{Named, Unnamed, Unit}};



#[proc_macro_derive(Termable)]
pub fn termable_derive(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let name = &ast.ident;
	
	let ret: TokenStream = match ast.data {
		Struct(ref s)=> {
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
					n.unnamed.iter().enumerate().map(|(i, _m)|{
						let id = syn::Index::from(i);
						quote!{ self.#id.termify() }
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
						termpose::list!(
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
		},
		Union(_)=> {
			panic!("derive(Termable) cannot support untagged unions. It would have no way of knowing which variant to read from")
		}
	};
	
	ret
}



#[proc_macro_derive(Determable)]
pub fn determable_derive(input: TokenStream) -> TokenStream {
	let ast: syn::DeriveInput = syn::parse(input).unwrap();

	let name = &ast.ident;
	
	let ret = TokenStream::from( match ast.data {
		Struct(ref s)=> {
			
			let with_body = |method_body|-> PM2TS {
				//it seeks over the contents of the term list in such way where if the items are in order it will find each one immediately
				quote! {
					impl termpose::Determable for #name {
						fn determify(v:&termpose::Term)-> Result<Self, termpose::DetermifyError> {
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
					
						let var_parsing:Vec<PM2TS> = n.named.iter().map(|m:&syn::Field|{
							let var_ident = &m.ident.as_ref().unwrap();
							let var_type = &m.ty;
							quote!{
								#var_ident: #var_type::determify(scanning.seek(stringify!(#var_ident))?)?
							}
						}).collect();
						
						let number_of_fields = var_parsing.len();
						
						with_body(quote!{
							let mut scanning = termpose::FieldScanning::new(v);
							if scanning.li.len() != #number_of_fields {
								return Err(termpose::DetermifyError::new(v, format!("{} expected the term to have {} elements, but it has {}", stringify!(#name), #number_of_fields, scanning.li.len())));
							}
							
							// let mut check_for = |key:&str|-> Result<&termpose::Term, termpose::DetermifyError> {
							// 	for i in 0..li.len() {
							// 		let c = &li[current_eye];
							// 		if c.initial_str() == key {
							// 			return
							// 				if let Some(s) = c.tail().next() {
							// 					Ok(s)
							// 				}else{
							// 					Err(termpose::DetermifyError::new(c, format!("expected a subterm, but the term has no tail")))
							// 				}
							// 		}
							// 		current_eye += 1;
							// 		if current_eye >= li.len() { current_eye = 0; }
							// 	}
							// 	Err(termpose::DetermifyError::new(v, format!("could not find key \"{}\"", key)))
							// };
							
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
						quote!{#ty::determify(&li[#i])?}
					});
					
					with_body(quote!{
						let li = v.tail().as_slice();
						if li.len() == #number_of_fields {
							Ok(Self(#(#each_field),*))
						}else{
							Err(termpose::DetermifyError::new(v, format!("{} expected the term to have {} fields, but it has {}", stringify!(#name), #number_of_fields, li.len())))
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
							quote!{ #id: #ty::determify(scanning.seek(stringify!(#id))?)? }
						}).collect();
						
						let number_of_fields = each_feild.len();
						
						quote!{
							let mut scanning = termpose::FieldScanning::new(v);
							if scanning.li.len() != #number_of_fields {
								return Err(termpose::DetermifyError::new(v, format!("variant {} expected {} elements, found {}", stringify!(#variant_name), #number_of_fields, scanning.li.len())));
							}
							Ok(#name::#variant_name {
								#(#each_feild),*
							})
						}
					},
					Unnamed(ref n)=> {
						
						let each_feild = n.unnamed.iter().enumerate().map(|(i,m)|{
							let ty = &m.ty;
							quote!{ #ty::determify(&li[#i])? }
						});
						
						let number_of_fields = each_feild.len();
						
						quote!{
							let li = v.tail().as_slice();
							
							if li.len() != #number_of_fields {
								return Err(termpose::DetermifyError::new(v, format!("variant {} expected {} elements, found {}", stringify!(#variant_name), #number_of_fields, li.len())));
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
				impl Determable for #name {
					fn determify(v:&termpose::Term)-> Result<Self, termpose::DetermifyError> {
						match v.initial_str() {
							#(#variant_cases,)*
							erc => Err(termpose::DetermifyError::new(v, format!("expected a {}, but no variant of {} is called {}", stringify!(#name), stringify!(#name), erc))),
						}
					}
				}
			}
		},
		
		
		Union(_)=> {
			panic!("derive(Determable) doesn't yet support unions")
		}
	});
	
	
	// println!("{}", ret);
	
	ret
}


