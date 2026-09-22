pub mod generic;
pub mod store;
pub mod ty;
pub mod type_id;

pub use generic::{GenericParamId, TypeSubstitution};
pub use store::TypeStore;
pub use ty::Type;
pub use type_id::TypeId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types_constants_and_formatting() {
        let store = TypeStore::new();

        assert_eq!(store.get(TypeId::UNIT), &Type::Unit);
        assert_eq!(store.get(TypeId::BOOL), &Type::Bool);
        assert_eq!(store.get(TypeId::INT), &Type::Int);
        assert_eq!(store.get(TypeId::FLOAT), &Type::Float);
        assert_eq!(store.get(TypeId::STRING), &Type::String);

        assert_eq!(store.format_type(TypeId::UNIT), "Unit");
        assert_eq!(store.format_type(TypeId::BOOL), "Bool");
        assert_eq!(store.format_type(TypeId::INT), "Int");
        assert_eq!(store.format_type(TypeId::FLOAT), "Float");
        assert_eq!(store.format_type(TypeId::STRING), "String");
        assert_eq!(store.format_type(TypeId::INT64), "Int64");
        assert_eq!(store.format_type(TypeId::FLOAT64), "Float64");
    }

    #[test]
    fn test_compound_types_interning_and_formatting() {
        let mut store = TypeStore::new();

        // Optional
        let opt_int = store.intern(Type::Optional(TypeId::INT));
        assert_eq!(store.format_type(opt_int), "Option<Int>");

        // References
        let ref_str = store.intern(Type::Reference {
            mutable: false,
            inner: TypeId::STRING,
        });
        let mut_ref_str = store.intern(Type::Reference {
            mutable: true,
            inner: TypeId::STRING,
        });
        assert_eq!(store.format_type(ref_str), "&String");
        assert_eq!(store.format_type(mut_ref_str), "&mut String");

        // Function
        let fn_ty = store.intern(Type::Function {
            params: vec![TypeId::INT, TypeId::INT],
            return_type: TypeId::BOOL,
        });
        assert_eq!(store.format_type(fn_ty), "fn(Int, Int) -> Bool");

        // Tuple
        let tuple_ty = store.intern(Type::Tuple(vec![TypeId::INT, TypeId::STRING]));
        assert_eq!(store.format_type(tuple_ty), "(Int, String)");

        // List and Array
        let list_ty = store.intern(Type::List(TypeId::INT));
        let arr_ty = store.intern(Type::Array {
            element: TypeId::BYTE,
        });
        assert_eq!(store.format_type(list_ty), "List<Int>");
        assert_eq!(store.format_type(arr_ty), "[Byte]");

        // Map
        let map_ty = store.intern(Type::Map {
            key: TypeId::STRING,
            value: TypeId::INT,
        });
        assert_eq!(store.format_type(map_ty), "Map<String, Int>");
    }

    #[test]
    fn test_type_equality_and_assignability() {
        let mut store = TypeStore::new();

        assert!(store.type_equals(TypeId::INT, TypeId::INT));
        assert!(!store.type_equals(TypeId::INT, TypeId::STRING));
        assert!(!store.type_equals(TypeId::INT32, TypeId::INT64));
        assert!(!store.type_equals(TypeId::INT, TypeId::FLOAT));

        // Error type recovery compatibility
        assert!(store.type_equals(TypeId::INT, TypeId::ERROR));
        assert!(store.type_equals(TypeId::ERROR, TypeId::STRING));

        // Deduplication in interner
        let t1 = store.intern(Type::Optional(TypeId::BOOL));
        let t2 = store.intern(Type::Optional(TypeId::BOOL));
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_generic_param_and_applied_types() {
        let mut store = TypeStore::new();

        let param_t = store.alloc_generic_param("T");
        let ty_t = store.intern(Type::GenericParam(param_t));
        assert_eq!(store.format_type(ty_t), "T");

        let param_u = store.alloc_generic_param("U");
        let ty_u = store.intern(Type::GenericParam(param_u));
        assert_eq!(store.format_type(ty_u), "U");
        assert_ne!(ty_t, ty_u);

        // Box<T>
        let box_base = store.intern(Type::Named("Box".to_string()));
        let box_t = store.intern(Type::Applied {
            base: box_base,
            arguments: vec![ty_t],
        });
        assert_eq!(store.format_type(box_t), "Box<T>");

        // Box<Int> and Box<String>
        let box_int = store.intern(Type::Applied {
            base: box_base,
            arguments: vec![TypeId::INT],
        });
        let box_str = store.intern(Type::Applied {
            base: box_base,
            arguments: vec![TypeId::STRING],
        });
        assert_eq!(store.format_type(box_int), "Box<Int>");
        assert_eq!(store.format_type(box_str), "Box<String>");
        assert!(store.type_equals(box_int, box_int));
        assert!(!store.type_equals(box_int, box_str));

        // Nested applied: Box<List<Int>>
        let list_int = store.intern(Type::List(TypeId::INT));
        let box_list_int = store.intern(Type::Applied {
            base: box_base,
            arguments: vec![list_int],
        });
        assert_eq!(store.format_type(box_list_int), "Box<List<Int>>");

        // Map<String, List<User>>
        let user_ty = store.intern(Type::Named("User".to_string()));
        let list_user = store.intern(Type::List(user_ty));
        let map_str_list_user = store.intern(Type::Map {
            key: TypeId::STRING,
            value: list_user,
        });
        assert_eq!(
            store.format_type(map_str_list_user),
            "Map<String, List<User>>"
        );
    }

    #[test]
    fn test_type_substitution() {
        let mut store = TypeStore::new();

        let param_t = store.alloc_generic_param("T");
        let ty_t = store.intern(Type::GenericParam(param_t));

        let mut subst = TypeSubstitution::new();
        subst.insert(param_t, TypeId::INT);

        // 1. T -> Int
        assert_eq!(subst.apply(ty_t, &mut store), TypeId::INT);

        // 2. List<T> -> List<Int>
        let list_t = store.intern(Type::List(ty_t));
        let list_int = subst.apply(list_t, &mut store);
        assert_eq!(store.format_type(list_int), "List<Int>");

        // 3. Map<String, T> -> Map<String, Int>
        let map_str_t = store.intern(Type::Map {
            key: TypeId::STRING,
            value: ty_t,
        });
        let map_str_int = subst.apply(map_str_t, &mut store);
        assert_eq!(store.format_type(map_str_int), "Map<String, Int>");

        // 4. (T, T) -> T substituted to (Int, Int) -> Int
        let fn_t = store.intern(Type::Function {
            params: vec![ty_t, ty_t],
            return_type: ty_t,
        });
        let fn_int = subst.apply(fn_t, &mut store);
        assert_eq!(store.format_type(fn_int), "fn(Int, Int) -> Int");

        // 5. Box<T> -> Box<Int>
        let box_base = store.intern(Type::Named("Box".to_string()));
        let box_t = store.intern(Type::Applied {
            base: box_base,
            arguments: vec![ty_t],
        });
        let box_int = subst.apply(box_t, &mut store);
        assert_eq!(store.format_type(box_int), "Box<Int>");

        // 6. Optional & Reference: &T -> &Int, Option<T> -> Option<Int>
        let ref_t = store.intern(Type::Reference {
            mutable: false,
            inner: ty_t,
        });
        let opt_t = store.intern(Type::Optional(ty_t));
        let res_ref = subst.apply(ref_t, &mut store);
        let res_opt = subst.apply(opt_t, &mut store);
        assert_eq!(store.format_type(res_ref), "&Int");
        assert_eq!(store.format_type(res_opt), "Option<Int>");
    }
}
