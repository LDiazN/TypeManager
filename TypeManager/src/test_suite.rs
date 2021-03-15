#[allow(unused)]
use crate::utils::*;
#[allow(unused)]
use crate::type_system::*;

#[test]
fn test_gcd() {
    // just test that gcd works ok
    assert_eq!(gcd(4, 1), 1);
    assert_eq!(gcd(4,4), 4);
    assert_eq!(gcd(7,9), 1);
    assert_eq!(gcd(3,9), 3);
}

#[test]
fn test_lcm() {
    // test that lcm works ok
    assert_eq!(lcm(4, 4), 4);
    assert_eq!(lcm(7,9), 7*9);
    assert_eq!(lcm(3, lcm(5, 7)), lcm(lcm(3,5), 7));
    assert_eq!(lcm(1,3), 3);
}

#[test]
fn test_permutations_singleton() {
    // singleton should return just one 
    let mut v = vec![1];
    assert_eq!(permutations(&mut v), vec![v]);
}

#[test]
fn test_permutations_empty() {
    // empty should return empty list
    let mut v : Vec<usize>     = vec![];
    let void : Vec<Vec<usize>> = vec![];
    assert_eq!(permutations(&mut v ), void);
}

#[test]
fn test_permutations() {
    // permutations for simple list
    let mut v  = vec![1,2,3];
    let ans = vec![
        vec![1,2,3],
        vec![1,3,2],
        vec![2,1,3],
        vec![2,3,1],
        vec![3,2,1],
        vec![3,1,2]
    ];

    let output = permutations(&mut v);

    // check set equality
    for v in &output{
        assert!(ans.contains(&v));
    }
    for v in &ans{
        assert!(output.contains(&v));
    } 
}

// shortcut to create atomic types in tests
#[allow(unused)]
fn atom(repr : usize, align : usize) -> Type {
    Type::Atomic(Atomic::new(repr, align))
}

#[allow(unused)]
fn strc(members : TypeList) -> Type {
    Type::Struct(Struct::new(members))
}

#[allow(unused)]
fn uni(variants : TypeList) -> Type {
    Type::Union(Union::new(variants))
}


#[test]
fn test_add_atomics() {
    let mut manager = TypeManager::new();
    let name = format!("int");
    // check add ok
    assert!(manager.add(name.clone(), atom(4,4)).is_ok()) ;
    assert_eq!(manager.add(name, atom(4,4)), Err(TypeError::TypeRedefinition));
    
    // check add 0 sized should crash
    assert_eq!(manager.add(format!("zero"), atom(0,4)), Err(TypeError::NoZeroSizedType));

    // check add 0 aligned should crash
    assert_eq!(manager.add(format!("zero"), atom(4,0)), Err(TypeError::NoZeroAlign));
}

#[test]
fn test_add_compound() {
    let mut manager = TypeManager::new();
    let _ = manager.add(format!("int"), atom(4,4));
    let _ = manager.add(format!("char"), atom(2,4));

    // check adding non existent type crashes
    assert_eq!( 
        manager.add(format!("s"), 
        strc( 
            vec!["int".to_string(), "foo".to_string()])
        ), Err(TypeError::TypeDoesNotExist(format!("foo"))) );

    // check cannot add empty compund
    assert_eq!(
        manager.add("s".to_string(), uni(vec![])),
        Err(TypeError::EmptyCompoundType)
    );

    assert_eq!(
        manager.add("s".to_string(), strc(vec![])),
        Err(TypeError::EmptyCompoundType)
    );

    assert_eq!(
        manager.add("s".to_string(), strc(vec!["int".to_string()])),
        Ok(())
    );

    assert_eq!(
        manager.add("u".to_string(), uni(vec!["s".to_string(), "s".to_string(), "int".to_string() ])),
        Ok(())
    );
}


#[test]
fn test_size_struct() {
    let mut manager = TypeManager::new();
    let int = "int".to_string();
    let my_char = "char".to_string();

    let _ = manager.add(int.clone(), atom(4,4));
    let _ = manager.add(my_char.clone(), atom(1,4));
    let _ = manager.add("s1".to_string(), strc(vec![ int.clone(), my_char.clone() ]));
    let _ = manager.add("s2".to_string(), strc(vec![my_char, int]));

    // check sizes 
    let s1 = manager.get(&"s1".to_string()).unwrap();
    let s2 = manager.get(&"s2".to_string()).unwrap();
    
    // s1 it's well-sorted by default, so it has the same size for every packing
    assert_eq!(s1.size(&manager, Struct::unpacked_size), 5);
    assert_eq!(s1.size(&manager, Struct::optimized_size), 5);
    assert_eq!(s1.size(&manager, Struct::packed_size), 5);

    // s2 will lose some space in its unpacked version, but with optimal sorting 
    // becomes s1
    assert_eq!(s2.size(&manager, Struct::unpacked_size), 8);
    assert_eq!(s2.size(&manager, Struct::optimized_size), 5);
    assert_eq!(s2.size(&manager, Struct::packed_size), 5);

}

#[test]
fn test_size_and_align_union() {
    let mut manager = TypeManager::new();
    let int         = "int".to_string();
    let my_char     = "char".to_string();

    let _ = manager.add(int.clone(), atom(4,4));
    let _ = manager.add(my_char.clone(), atom(1,4));
    let _ = manager.add("s1".to_string(), strc(vec![ int.clone(), my_char.clone() ]));
    let _ = manager.add("s2".to_string(), strc(vec![my_char.clone(), int.clone()]));
    let _ = manager.add("u1".to_string(), uni(vec![int.clone(), int.clone()]) );
    let _ = manager.add("u2".to_string(), uni(vec!["s2".to_string(), "s1".to_string()]) );

    // check sizes 
    let u1 = manager.get(&"u1".to_string()).unwrap();
    let u2 = manager.get(&"u2".to_string()).unwrap();
    
    // for every packing type
    assert_eq!(u1.align(&manager, Struct::unpacked_align),  4);
    assert_eq!(u1.align(&manager, Struct::packed_align),    4);
    assert_eq!(u1.align(&manager, Struct::optimized_align), 4);
    
    // size for every packing type
    assert_eq!(u1.size(&manager, Struct::unpacked_size),    4); 
    assert_eq!(u1.size(&manager, Struct::packed_size),      4);
    assert_eq!(u1.size(&manager, Struct::optimized_size),   4);

    // now with a more complex union type
    assert_eq!(u2.align(&manager, Struct::unpacked_align),  4);
    assert_eq!(u2.align(&manager, Struct::packed_align),    4);
    assert_eq!(u2.align(&manager, Struct::optimized_align), 4);

    assert_eq!(u2.size(&manager, Struct::unpacked_size),    8); // 1 bytes for char + 3 bytes align + 4 bytes for int
    assert_eq!(u2.size(&manager, Struct::packed_size),      5);
    assert_eq!(u2.size(&manager, Struct::optimized_size),   5);
}