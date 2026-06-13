use proc_macro::TokenStream;
use proc_macro2::{Ident, Span}; 
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn convert_to_snake_case(s: String) -> String {
    let mut result = String::new();
    let mut previous_char_was_upper = false;

    for char in s.chars() {
        if char.is_uppercase() {
            if !previous_char_was_upper && result.len() > 0 {
                result.push('_');
            }
            result.push(char.to_ascii_lowercase());
            previous_char_was_upper = true;
        } else {
            result.push(char);
            previous_char_was_upper = false;
        }
    }

    result.replace(' ', "_")
}

#[proc_macro_derive(Repository)]
pub fn dal_repository_derive(input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as DeriveInput);

    let struct_name = &input_struct.ident;
    let struct_name_str = struct_name.to_string();
    let struct_name_repo = Ident::new(&format!("{}Repository", struct_name_str), Span::call_site());
    let table_name_str = format!("{}s", convert_to_snake_case(struct_name_str.clone()));
    let table_name = Ident::new(&table_name_str, Span::call_site());
    let table_module = quote! { crate::schema::#table_name  };

    let generated_impl = quote! {
            pub struct #struct_name_repo {
                pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>,
            }

            impl #struct_name_repo
            {
                pub fn new(pool: Option<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::pg::PgConnection>>>) -> Self {
                    match pool {
                        Some(pool) => Self { pool: pool },
                        None =>{
                            let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
                            let manager = diesel::r2d2::ConnectionManager::<diesel::pg::PgConnection>::new(database_url);
                            let pool = diesel::r2d2::Pool::builder()    
                                .build(manager)
                                .expect("Failed to create pool.");

                            Self { pool: pool }
                        }
                    }
                }

                pub async fn find_all(&self) -> shared::errors::ServiceResult<Vec<#struct_name>> {
                    let mut conn = self.pool.get().map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;
                    let result = #table_module::table.load::<#struct_name>(&mut conn)
                        .map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;

                    Ok(result)
                }

                pub async fn insert(&self, item: #struct_name) -> shared::errors::ServiceResult<#struct_name> {
                    let mut conn = self.pool.get().map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;
                    let data = diesel::insert_into(#table_module::table)
                        .values(item)
                        .returning(#struct_name::as_select())
                        .get_result(&mut conn)?;
                    Ok(data)
                }

                pub async fn find_by_id(&self, id: i64) -> shared::errors::ServiceResult<Option<#struct_name>> {
                    let mut conn = self.pool.get().map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;
                    let result = #table_module::table.filter(#table_module::id.eq(id))
                        .first::<#struct_name>(&mut conn)
                        .optional()
                        .map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;
                
                    Ok(result)
                }

                pub async fn update(&self, id: i64, new_data: #struct_name) -> shared::errors::ServiceResult<#struct_name> {
                    let mut conn = self.pool.get().map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;
                    let result = diesel::update(#table_module::table.filter(#table_module::id.eq(id)))
                        .set(new_data)
                        .get_result::<#struct_name>(&mut conn)
                        .map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;

                    Ok(result)
                }

                pub async fn delete(&self, id: i64) -> shared::errors::ServiceResult<usize> {
                    let mut conn = self.pool.get().map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;

                    let result = diesel::delete(#table_module::table.filter(#table_module::id.eq(id)))
                        .execute(&mut conn)
                        .map_err(|e| shared::errors::ServiceError::DatabaseError(e.to_string()))?;
                    Ok(result)
                }
            }
        };

    TokenStream::from(generated_impl)
}
