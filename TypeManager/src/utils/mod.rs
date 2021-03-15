
/// Lowest Common Multiple for two numbers
/// ## Params
/// `x` - a number
/// `y` - other number
/// ---
/// ## Return
/// lowest common multiple for x and y
pub fn lcm(x : usize, y : usize) -> usize {
    x * y / gcd(x, y)
}

/// Gratest common divisor for two numbers
/// ## Params
/// `x` - a number
/// `y` - other number
/// ---
/// ## Return 
/// lowest common multiple for x, y
pub fn gcd(x : usize, y : usize) -> usize {
    let mut max = x;
    let mut min = y;
    if min > max {
        let val = max;
        max = min;
        min = val;
    }

    loop {
        let res = max % min;
        if res == 0 {
            return min;
        }

        max = min;
        min = res;
    }
}


/// Compute permutations for a vector of copy-able comparable
/// elements 
/// ## Params
/// * `list` - List of elements to permute
/// ---
/// ## Return
/// A list of permutations
pub fn permutations<T>(list : &mut Vec<T>) -> Vec<Vec<T>>
    where 
        T : Eq,
        T : Copy,
{
 
    let mut ans = Vec::with_capacity(2usize.pow(list.len() as u32));

    permutation_helper(list, 0, list.len()-1, &mut ans);

    ans
}

/// Helper function to co,pute all permutations for a vector
fn permutation_helper<T>(list :&mut Vec<T>, l : usize, r : usize, buff :&mut Vec<Vec<T>>) 
    where  
        T : Eq,
        T : Copy,
{
    if l == r {
        buff.push(list.clone())
    }

    for i in l..r+1 {
        // swap them for now
        let mut temp = list[i].clone();
        list[i] = list[l].clone();
        list[l] = temp;

        

        // permute everything else
        permutation_helper(list, l+1, r, buff);

        // revert swap
        temp = list[i].clone();
        list[i] = list[l].clone();
        list[l] = temp;
    }

} 

#[test]
fn check_permutations() {
    let mut v = vec!['a','b', 'c'];
    for v in permutations(&mut v) {
        println!("{:?}", v);
    }
    println!("Original: {:?}", v);
}