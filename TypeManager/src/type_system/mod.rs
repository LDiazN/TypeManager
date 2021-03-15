/*
    Main internal functions and data structures 
    for our type system simulator 
*/

use std::collections::HashMap;
use crate::utils;


// A type name
pub type Name = String;
// A list of types
pub type TypeList = Vec<Name>;
// A map from names to type data
pub type TypeTable = HashMap<Name, Type>;
// Assumed word size for our machine
pub const WORD_SIZE : usize = 4;

/// Atomic Data type structure
#[derive(Debug)]
pub struct Atomic {
    pub representation: usize,
    pub alignment:      usize
}

/// Struct type structure
#[derive(Debug)]
pub struct Struct {
    pub members: TypeList
}

/// Union type structure
#[derive(Debug)]
pub struct Union {
    pub variants: TypeList
}

/// Every Possible data type
#[derive(Debug)] 
pub enum Type {
    Atomic  (Atomic),
    Struct  (Struct),
    Union   (Union)
}

/// Every possible error 
pub enum TypeError {
    TypeRedefinition,
    NoZeroAlign,
    NoZeroSizedType,
    EmptyCompoundType,
    TypeDoesNotExist(Name)
}

/// Manager object controlling our stored types
pub struct TypeManager {
    types: TypeTable
}

impl TypeManager {

    /// Create a new type manager
    pub fn new() -> TypeManager {
        TypeManager {
            types: TypeTable::new()
        }
    }

    /// Try to add a new type to our type manager
    /// ## Params
    /// * `typename` - name of our new type
    /// * `new_type` - type definition itself
    /// ---
    /// ## Return 
    /// Error describing the issue if could not add, or nothing on success 
    pub fn add(&mut self, typename : Name,  new_type : Type) -> Result<(), TypeError> {

        // if there was some error, return such error. Else, keep going
        if let Err(e) = self.check_new_type(&typename, &new_type) {
            return Err(e)
        }

        // add the new type
        self.types.insert(typename, new_type);
        Ok(())
    }

    /// Try to get data for a type given its name
    /// ## Params
    /// `typename` - name of type whose data is to be retrieved
    /// ## Return
    /// A reference to this type's data if the given name is a valid one,
    /// None otherwhise
    pub fn get(&self, typename: &Name) -> Option<&Type> {
        self.types.get(typename)
    }

    /// Return a human-readable String information about a single type
    /// ## Params
    /// * `typename` - name of type to display
    /// ---
    /// ## Return
    /// String with data about the given type 
    pub fn display(&self, typename: &Name) -> Result<String, TypeError> {
        if !self.types.contains_key(typename) {
            Err(TypeError::TypeDoesNotExist(typename.clone()))
        }
        else {
            Ok(self.types.get(typename).unwrap().display(self))
        }
    }

    /// Checks if the given type could be a valid new type
    fn check_new_type(&self, name: &Name, type_data: &Type) -> Result<(), TypeError> {

        // if name already stored, raise an error
        if self.types.contains_key(name) {
            return Err(TypeError::TypeRedefinition)
        }

        // Check for every kind of type
        match type_data {
            Type::Atomic(a) => {
                // Check for 0-sized types
                if a.representation == 0 {
                    return Err(TypeError::NoZeroSizedType)
                }

                // Check for 0-aligned types
                if a.alignment == 0 {
                    return Err(TypeError::NoZeroAlign)
                }

                Ok(())
            },
            Type::Struct(s) => {        
                
                // Check if some member type is an invalid type
                for sym in &s.members {
                    if !self.types.contains_key(sym) {
                        return Err(TypeError::TypeDoesNotExist(sym.clone()))
                    }
                }

                // no empty type allowed
                if s.members.is_empty() {
                    return Err(TypeError::EmptyCompoundType)
                }

                Ok(())
            },
            Type::Union(u) => {

                // Check if some member type is an invalid type
                for sym in &u.variants {
                    if !self.types.contains_key(sym) {
                        return Err(TypeError::TypeDoesNotExist(sym.clone()))
                    }
                }

                // no empty type allowed
                if u.variants.is_empty() {
                    return Err(TypeError::EmptyCompoundType)
                }
                Ok(())
            }

        }
    }
}

impl Type {

    /// Create an human readable description for this type 
    pub fn display(&self, manager : &TypeManager) -> String {
        match self {
            Type::Atomic(a) => a.display(),
            Type::Struct(s) => s.display(manager),
            Type::Union(u)  => u.display(manager)
        }
    }

    pub fn align(   &self, 
                    manager : &TypeManager, 
                    struct_packing_align : fn(&Struct, &TypeManager) -> usize
                    ) -> usize {
        match self {
            Type::Atomic(a) => a.align(),
            Type::Struct(s) => struct_packing_align(s, manager),
            Type::Union(u)  => u.align(manager, struct_packing_align)
        }
    }

    /// Get size for this type depending on how structs are packed
    /// ## Params
    /// `manager` - object to retrieve data from
    /// `struct_packing_align` - function to retrieve struct alignment depending on its packing type
    /// ---
    /// ## Return
    /// Total size
    pub fn size(&self,
                    manager : &TypeManager,
                    struct_packing_size : fn(&Struct, &TypeManager) -> usize
                    ) -> usize {

        match self {
            Type::Atomic(a) => a.size(),
            Type::Struct(s) => struct_packing_size(s, manager),
            Type::Union(u)  => u.size(manager, struct_packing_size)
        }
    }

    /// Return alignment for this type depending on how structs are packed
    /// ## Params
    /// `manager` - Type manager to retrieve data from
    /// `struct_packing_align` - 
    pub fn alignment(   &self,
                    manager : &TypeManager,
                    struct_packing_align : fn(&Struct, &TypeManager) -> usize
                    ) -> usize 
    {

        match self {
            Type::Atomic(a) => a.align(),
            Type::Union(u)  => u.align(manager, struct_packing_align),
            Type::Struct(s) => struct_packing_align(s, manager)
        }
    }
}

impl Atomic {

    pub fn new(representation : usize, alignment : usize) -> Atomic {
        Atomic {
            representation,
            alignment
        }
    }

    pub fn display(&self) -> String {
        format!("⚛️  Atómico:\n   * Representación: {}\n   * Alineación: {}", self.representation, self.alignment)
    }

    pub fn size(&self) -> usize {
        self.representation
    }

    pub fn align(&self) -> usize {
        self.alignment
    }
}

impl Struct {

    pub fn new(members: TypeList) -> Struct {
        Struct {
            members,
        }
    }

    pub fn display(&self, manager : &TypeManager) -> String {

        let optimal_size  = self.optimized_size(manager);
        let unpacked_size = self.unpacked_size(manager);
        let packed_size   = self.packed_size(manager);

        let optimized_data = format!(
            "   * Optimizado:\n      + Tamaño: {}\n      + Perdida: {}", 
            optimal_size, 
            optimal_size - packed_size
        );

        let unpacked_data = format!(
            "   * Sin Empaquetar:\n      + Tamaño: {}\n      + Perdida: {}", 
            unpacked_size,
            unpacked_size - packed_size
        );

        let packed_data = format!(
            "   * Empaquetado:\n      + Tamaño: {}\n      + Perdida: {}", 
            packed_size,
            packed_size - packed_size
        );

        format!("📦 Struct:\n{}\n{}\n{}\n", optimized_data, unpacked_data, packed_data)
    }

    fn unpacked_size(&self, manager: &TypeManager) -> usize {
        // We are going to compute the next available position in the struct
        // where the next data should be. When the loop ends, current position
        // will actually be our desired struct size
        // .....................
        // ^ curr pos starts here
        // ------..-----.----...
        //                   ^ curr pos ends here

        let mut curr_pos = 0;
        for member in &self.members {
            let my_type = manager.get(member).unwrap();
            let size = my_type.size(manager, Struct::unpacked_size);
            let align = my_type.align(manager, Struct::unpacked_align);

            if curr_pos % align != 0 {
                curr_pos += align - curr_pos % align
            }

            curr_pos += size
        }

        curr_pos
    }

    fn packed_size(&self, manager: &TypeManager) -> usize {
        
        let mut sum = 0;
        for t in &self.members {
            let my_type = manager.get(&t).unwrap();
            sum += my_type.size(manager, Struct::packed_size);
        }

        sum
    }

    fn optimized_size(&self, manager: &TypeManager) -> usize {
        let (_, size) = self.get_optimal_layout(manager);

        size
    }

    fn unpacked_align(&self, manager: &TypeManager) -> usize {
        manager
            .get(&self.members[0])
            .unwrap()
            .size(manager, Struct::unpacked_align)
    }

    fn packed_align(&self, manager: &TypeManager) -> usize {
        self.unpacked_align(manager)
    }

    fn optimized_align(&self, manager: &TypeManager) -> usize {
        let (layout, _) = self.get_optimal_layout(manager);

        manager
            .get(&layout[0])
            .unwrap()
            .align(manager, Struct::optimized_align)
    }

    fn get_optimal_layout(&self, manager : &TypeManager) -> (TypeList, usize) { // layout, size
        // Compute every permutation
        let permuts = self.members_permutations();

        // Compute size for every permutation and select the best one
        let mut min = usize::MAX;
        let mut layout = vec![];

        for typelist in permuts {
            let mut curr_size = 0;
            for my_type in &typelist {
                curr_size += manager
                                .get(&my_type)
                                .unwrap()
                                .size(manager, Struct::optimized_size);

                
            }
            
            if curr_size < min {
                min = curr_size;
                layout = typelist;
            } 
        }

        (layout,min)
    }
    fn members_permutations(&self) -> Vec<TypeList> {
            // get indices
            let indices = utils::permutations(&mut (0..self.members.len()).collect());
            // Convert permutations of indices into permutations of names
            indices
                .iter()
                .map(
                    |l| l
                        .iter()
                        .map(
                            |i| 
                            self.members[*i].clone()
                        )
                        .collect()
                )
                .collect()
        // Esto se podria implementar mas eficientemente con iteradores, pero no nos pagan lo suficiente
    }
}


impl Union {

    pub fn new(variants : TypeList) -> Union {
        Union {
            variants
        }
    }

    pub fn display(&self, manager : &TypeManager) -> String {

        // Comput loss & size for every possible packing type
        let unpacked_size   = self.size(manager, Struct::unpacked_size);
        let unpacked_loss   = self.loss(manager, Struct::unpacked_size);
        let packed_size     = self.size(manager, Struct::packed_size);
        let packed_loss     = self.loss(manager, Struct::packed_size);
        let optimal_size    = self.size(manager, Struct::optimized_size);
        let optimal_loss    = self.loss(manager, Struct::optimized_size);

        let optimized_data = format!(
            "* Optimizado:\n      + Tamaño: {}\n      + Perdida: {}", 
            optimal_size, 
            optimal_loss
        );

        let unpacked_data = format!(
            "* Sin Empaquetar:\n      + Tamaño: {}\n      + Perdida: {}", 
            unpacked_size, 
            unpacked_loss
        );

        let packed_data  = format!(
            "* Packed data:\n      + Tamaño: {}\n      + Perdida: {}", 
            packed_size, 
            packed_loss
        );
        

        format!("👺 Variante:\n{}\n{}\n{}\n", optimized_data, unpacked_data, packed_data)
    }

    /// Return loss for this ariant register depending on its packing type for structs
    fn loss(&self, manager : &TypeManager, struct_packing_size : fn (&Struct, &TypeManager) -> usize) -> usize {
        let size = self.size(manager, struct_packing_size);
        let biggest_packed = {
            
            // Compute variant whose loss is the lesser
            let mut packed_size = usize::MIN;
            for typename in &self.variants {
                // Get the type
                let my_type = manager.get(&typename).unwrap();

                // if not the biggest one, continue
                if my_type.size(manager, struct_packing_size) != size {
                    continue
                }

                // check if the loss of this type is less than our current loss
                packed_size = std::cmp::max(
                                    packed_size, 
                                    my_type.size(manager, Struct::packed_size)
                                )
            }

            packed_size
        };

        size - biggest_packed
    }

    /// Return size for this variant type given the struct packing type
    /// and a manager describing this type and every other type
    /// ## Params
    /// `manager` - Manager object to retrieve data for every type
    /// `struct_packing_size` - function to get size for a struct depending on its packing type
    pub fn size(&self, 
                manager: &TypeManager, 
                struct_packing_size : fn (&Struct, &TypeManager) -> usize
                ) -> usize
    {
        // Linear search for max value
        let mut maxi = 0;
        for t in &self.variants {
            // the type is available, our api to add types will ensure it
            let my_type = manager.get(&t).unwrap();

            maxi = std::cmp::max(my_type.size(manager, struct_packing_size), maxi)
        }

        maxi
    }

    /// Get alignment for a variant type, 
    /// is the lowest common multiple between every align for every possible variant
    /// ## Params 
    /// * `manager` - manager to retrieve data from
    /// * `struct_packing_align` - function to compute align depending on the packing type
    /// ---
    /// ### Return
    /// alignment for this variant type
    pub fn align(   &self,
                    manager : &TypeManager,
                    struct_packing_align : fn (&Struct, &TypeManager) -> usize
                ) -> usize
    {
        let mut lcm = 1;
        for i in 0..(self.variants.len() - 1) {

            // Compute lcm for every number
            let size1 = manager.get(&self.variants[i]).unwrap().align(manager, struct_packing_align);
            let size2 = manager.get(&self.variants[i+1]).unwrap().align(manager, struct_packing_align);

            lcm = utils::lcm(size1, size2)
        }

        lcm
    }

}

impl TypeError {
    /// Turns an error into an human-readable string
    /// ## Return
    /// An human-readable string for the given error
    pub fn display(&self) -> String {
        match self {
            TypeError::TypeRedefinition => {
                String::from("No puedes redefinir un tipo ya existente")
            },
            TypeError::TypeDoesNotExist(s) => {
                format!("El símbolo '{}' no existe", s)
            },
            TypeError::EmptyCompoundType => {
                format!("No se permiten datos compuestos vacíos")
            },
            TypeError::NoZeroSizedType => {
                format!("No se permiten tipos de tamaño 0")
            },
            TypeError::NoZeroAlign => {
                format!("No se permite alinear a 0")
            }
        }
    }

}